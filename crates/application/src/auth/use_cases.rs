use crate::auth::dtos::*;
use crate::{AppError, AppResult};
use tracing::{info, instrument, warn};
use validator::Validate;

#[cfg(test)]
#[path = "use_cases_test.rs"]
mod tests;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use vyry_core::entities::{device_linking_sessions, devices, one_time_prekeys, users};
use infrastructure::crypto::signal::{
    generate_identity_keypair, generate_prekeys, generate_registration_id, generate_signed_prekey,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

// ============ Config ============

pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub refresh_token_expiration: i64,
}

// ============ Constants ============

const OTP_EXPIRY_SECONDS: u64 = 180;
const OTP_MAX_ATTEMPTS: u32 = 5;
const PIN_MIN_LENGTH: usize = 4;
const PIN_MAX_LENGTH: usize = 32;
const LINKING_SESSION_EXPIRY_MINUTES: i64 = 5;
const DEVICE_TYPE_PRIMARY: i16 = 1;
const DEVICE_TYPE_LINKED: i16 = 2;

// ============ Request OTP Use Case ============

pub struct RequestOtpUseCase;

impl RequestOtpUseCase {
    #[instrument(skip(redis_conn), fields(phone_number = %req.phone_number))]
    pub async fn execute(
        redis_conn: &mut MultiplexedConnection,
        req: RequestOtpRequest,
    ) -> AppResult<String> {
        // Validate input
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        // Check rate limit
        let attempts_key = format!("otp_attempts:{}", req.phone_number);
        let attempts: Option<u32> = redis_conn
            .get(&attempts_key)
            .await
            ?;

        if attempts.unwrap_or(0) >= OTP_MAX_ATTEMPTS {
            warn!("Rate limit exceeded for phone number: {}", req.phone_number);
            return Err(AppError::RateLimitExceeded(
                "Too many OTP requests. Please try again later.".to_string(),
            ));
        }

        // Generate 6-digit OTP
        let otp: String = (0..6)
            .map(|_| rand::thread_rng().gen_range(0..10).to_string())
            .collect();
        let key = format!("otp:{}", req.phone_number);

        // Store in Redis with expiration
        redis_conn
            .set_ex::<_, _, ()>(&key, &otp, OTP_EXPIRY_SECONDS)
            .await
            ?;

        // Increment attempts counter
        redis_conn
            .incr::<_, _, ()>(&attempts_key, 1)
            .await
            ?;
        redis_conn
            .expire::<_, ()>(&attempts_key, 600) // 10 minutes window
            .await
            ?;

        info!("OTP generated for phone number: {}", req.phone_number);
        // TODO: In production, send OTP via SMS provider
        // For now, return it for testing
        Ok(otp)
    }
}

// ============ Verify OTP Use Case ============

pub struct VerifyOtpUseCase;

impl VerifyOtpUseCase {
    #[instrument(skip(db, redis_conn, config), fields(phone_number = %req.phone_number))]
    pub async fn execute(
        db: &DatabaseConnection,
        redis_conn: &mut MultiplexedConnection,
        config: &AuthConfig,
        req: VerifyOtpRequest,
    ) -> AppResult<VerifyOtpResponse> {
        // Validate input
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let key = format!("otp:{}", req.phone_number);
        
        // First check if key exists
        let exists: bool = redis_conn
            .exists(&key)
            .await
            ?;
        
        if !exists {
            warn!("OTP not found for phone number: {}", req.phone_number);
            return Err(AppError::Authentication("Invalid or expired OTP".to_string()));
        }
        
        // Check TTL to verify it hasn't expired
        let ttl: i64 = redis_conn
            .ttl(&key)
            .await
            ?;
        
        // TTL returns -2 if key doesn't exist (shouldn't happen after EXISTS check)
        // -1 if key exists but has no expiration (shouldn't happen for OTP)
        // > 0 if key exists and hasn't expired
        // 0 if key exists but has just expired
        if ttl <= 0 {
            warn!("OTP expired for phone number: {} (TTL: {})", req.phone_number, ttl);
            return Err(AppError::Authentication("Invalid or expired OTP".to_string()));
        }
        
        // Get the stored OTP value
        let stored_otp: Option<String> = redis_conn
            .get(&key)
            .await
            ?;

        // Double-check: if key was deleted between EXISTS and GET (race condition)
        if stored_otp.is_none() {
            warn!("OTP key was deleted between checks for phone number: {}", req.phone_number);
            return Err(AppError::Authentication("Invalid or expired OTP".to_string()));
        }

        // Verify OTP value matches
        if stored_otp.unwrap() != req.otp {
            warn!("Invalid OTP attempt for phone number: {}", req.phone_number);
            return Err(AppError::Authentication("Invalid or expired OTP".to_string()));
        }

        // Delete OTP after successful verification
        redis_conn
            .del::<_, ()>(&key)
            .await
            ?;

        // Start transaction
        let txn = db.begin().await.map_err(AppError::from)?;

        // Check if user exists
        let existing_user = users::Entity::find()
            .filter(users::Column::PhoneNumber.eq(&req.phone_number))
            .one(&txn)
            .await
            .map_err(AppError::from)?;

        let (user, is_new_user) = match existing_user {
            Some(u) => {
                // Existing user - kick old primary device if this is a new primary login
                Self::kick_old_primary_device(&txn, u.user_id).await
                    .map_err(AppError::from)?;
                (u, false)
            }
            None => {
                // Create new user
                // Hash phone number for privacy-preserving lookups
                let phone_hash = Sha256::digest(req.phone_number.as_bytes()).to_vec();
                
                let new_user = users::ActiveModel {
                    user_id: Set(Uuid::new_v4()),
                    phone_number: Set(req.phone_number.clone()),
                    phone_number_hash: Set(phone_hash),
                    username: Set(None),
                    display_name: Set(None),
                    bio: Set(None),
                    profile_picture: Set(None),
                    background_image: Set(None),
                    last_seen_at: Set(None),
                    is_online: Set(false),
                    is_deleted: Set(false),
                    deleted_at: Set(None),
                    created_at: Set(Utc::now().into()),
                    updated_at: Set(Utc::now().into()),
                    pin_hash: Set(None),
                    registration_lock: Set(false),
                    registration_lock_expires_at: Set(None),
                    pin_set_at: Set(None),
                };
                (new_user.insert(&txn).await.map_err(AppError::from)?, true)
            }
        };

        // Check if profile setup is required
        let requires_profile_setup = user.display_name.is_none();
        let requires_pin = user.registration_lock && user.pin_hash.is_some();

        // Generate Signal Keys
        let (identity_key_pair, _) = generate_identity_keypair()
            .map_err(|e| AppError::Cryptographic(e.to_string()))?;
        let registration_id = generate_registration_id();
        let signed_prekey = generate_signed_prekey(&identity_key_pair, 1)
            .map_err(|e| AppError::Cryptographic(e.to_string()))?;
        let one_time_prekeys_list = generate_prekeys(1, 100)
            .map_err(|e| AppError::Cryptographic(e.to_string()))?;

        // Check if device_uuid already exists (could be from previous failed registration)
        // If it exists, delete the old device and its prekeys to allow re-registration
        if let Some(existing_device) = devices::Entity::find()
            .filter(devices::Column::DeviceUuid.eq(req.device_uuid))
            .one(&txn)
            .await
            .map_err(AppError::from)?
        {
            // Delete one-time prekeys first (due to foreign key constraint)
            one_time_prekeys::Entity::delete_many()
                .filter(one_time_prekeys::Column::DeviceId.eq(existing_device.device_id))
                .exec(&txn)
                .await
                .map_err(AppError::from)?;
            
            // Delete the old device
            devices::Entity::delete_by_id(existing_device.device_id)
                .exec(&txn)
                .await
                .map_err(AppError::from)?;
        }

        // Create Device (Primary for OTP login)
        let device = devices::ActiveModel {
            user_id: Set(user.user_id),
            device_uuid: Set(req.device_uuid),
            device_name: Set(req.device_name.clone()),
            platform: Set(req.platform.unwrap_or(1)),
            identity_key_public: Set(identity_key_pair.public_key),
            registration_id: Set(registration_id as i32),
            signed_prekey_id: Set(signed_prekey.id as i32),
            signed_prekey_public: Set(signed_prekey.public_key),
            signed_prekey_signature: Set(signed_prekey.signature),
            last_seen_at: Set(Utc::now().into()),
            created_at: Set(Utc::now().into()),
            device_type: Set(DEVICE_TYPE_PRIMARY),
            is_active: Set(true),
            linked_at: Set(None),
            linked_by_device_id: Set(None),
            ..Default::default()
        };

        let device = device.insert(&txn).await.map_err(AppError::from)?;

        // Insert One Time Prekeys - Batch Insert Optimization
        let prekey_models: Vec<one_time_prekeys::ActiveModel> = one_time_prekeys_list
            .into_iter()
            .map(|prekey| one_time_prekeys::ActiveModel {
                device_id: Set(device.device_id),
                prekey_id: Set(prekey.id as i32),
                public_key: Set(prekey.public_key),
                ..Default::default()
            })
            .collect();

        if !prekey_models.is_empty() {
             one_time_prekeys::Entity::insert_many(prekey_models)
                .exec(&txn)
                .await
                .map_err(AppError::from)?;
        }

        txn.commit().await.map_err(AppError::from)?;

        // Generate JWT tokens
        let (access_token, refresh_token) =
            Self::generate_tokens(config, user.user_id, device.device_id)?;

        Ok(VerifyOtpResponse {
            access_token,
            refresh_token,
            user_id: user.user_id,
            device_id: device.device_id,
            is_new_user,
            requires_profile_setup,
            requires_pin,
        })
    }

    async fn kick_old_primary_device(
        txn: &sea_orm::DatabaseTransaction,
        user_id: Uuid,
    ) -> AppResult<()> {
        // Deactivate all existing primary devices for this user
        let old_devices = devices::Entity::find()
            .filter(devices::Column::UserId.eq(user_id))
            .filter(devices::Column::DeviceType.eq(DEVICE_TYPE_PRIMARY))
            .filter(devices::Column::IsActive.eq(true))
            .all(txn)
            .await
            .map_err(AppError::from)?;

        for old_device in old_devices {
            let mut active_device: devices::ActiveModel = old_device.into();
            active_device.is_active = Set(false);
            active_device
                .update(txn)
                .await
                .map_err(AppError::from)?;
        }

        Ok(())
    }

    fn generate_tokens(
        config: &AuthConfig,
        user_id: Uuid,
        device_id: i64,
    ) -> AppResult<(String, String)> {
        let now = Utc::now();

        let access_claims = Claims {
            sub: user_id.to_string(),
            device_id,
            iat: now.timestamp(),
            exp: (now + Duration::minutes(15)).timestamp(),
            token_type: "access".to_string(),
        };

        let refresh_claims = Claims {
            sub: user_id.to_string(),
            device_id,
            iat: now.timestamp(),
            exp: (now + Duration::days(30)).timestamp(),
            token_type: "refresh".to_string(),
        };

        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let access_token = encode(&Header::default(), &access_claims, &encoding_key)
            .map_err(|e| AppError::Authentication(format!("JWT encoding error: {}", e)))?;
        let refresh_token = encode(&Header::default(), &refresh_claims, &encoding_key)
            .map_err(|e| AppError::Authentication(format!("JWT encoding error: {}", e)))?;

        Ok((access_token, refresh_token))
    }
}

// ============ Setup Profile Use Case ============

pub struct SetupProfileUseCase;

impl SetupProfileUseCase {
    #[instrument(skip(db), fields(user_id = %user_id))]
    pub async fn execute(
        db: &DatabaseConnection,
        user_id: Uuid,
        req: SetupProfileRequest,
    ) -> AppResult<SetupProfileResponse> {
        // Validate input first
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        // Validate display name
        let display_name = req.display_name.trim();
        if display_name.is_empty() {
            return Err(AppError::Validation("Display name cannot be empty".to_string()));
        }

        if display_name.len() < 2 {
            return Err(AppError::Validation("Display name must be at least 2 characters".to_string()));
        }

        if display_name.len() > 100 {
            return Err(AppError::Validation("Display name too long (max 100 characters)".to_string()));
        }

        // Validate username if provided
        if let Some(ref username) = req.username {
            let username = username.trim();
            if !username.is_empty() {
                // Username validation: alphanumeric, underscore, hyphen, 3-30 chars
                if username.len() < 3 {
                    return Err(AppError::Validation("Username must be at least 3 characters".to_string()));
                }

                if username.len() > 30 {
                    return Err(AppError::Validation("Username too long (max 30 characters)".to_string()));
                }

                // Use regex validation
                if !crate::auth::USERNAME_REGEX.is_match(username) {
                    return Err(AppError::Validation("Username can only contain letters, numbers, underscores, and hyphens (3-50 characters)".to_string()));
                }

                // Check if username is already taken
                let existing_user = users::Entity::find()
                    .filter(users::Column::Username.eq(username))
                    .filter(users::Column::UserId.ne(user_id))
                    .one(db)
                    .await?;

                if existing_user.is_some() {
                    return Err(AppError::Validation("Username is already taken".to_string()));
                }
            }
        }

        // Validate bio if provided
        if let Some(ref bio) = req.bio {
            let bio = bio.trim();
            if !bio.is_empty() && bio.len() > 500 {
                return Err(AppError::Validation("Bio too long (max 500 characters)".to_string()));
            }
        }

        // Validate profile picture URL if provided
        if let Some(ref url) = req.profile_picture_url {
            let url = url.trim();
            if !url.is_empty() {
                // Basic URL validation
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    return Err(AppError::Validation("Profile picture URL must be a valid HTTP/HTTPS URL".to_string()));
                }

                if url.len() > 2048 {
                    return Err(AppError::Validation("Profile picture URL too long (max 2048 characters)".to_string()));
                }

                // Additional URL format validation - check for valid URL structure
                if url.len() < 10 || !url.contains("://") {
                    return Err(AppError::Validation("Invalid profile picture URL format".to_string()));
                }

                // Check for valid domain structure (at least one dot after protocol)
                if let Some(domain_part) = url.split("://").nth(1) {
                    if domain_part.is_empty() || !domain_part.contains('.') {
                        return Err(AppError::Validation("Invalid profile picture URL format".to_string()));
                    }
                } else {
                    return Err(AppError::Validation("Invalid profile picture URL format".to_string()));
                }
            }
        }

        // Find and update user
        let user = users::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

        let now = Utc::now();
        let mut active_user: users::ActiveModel = user.into();
        active_user.display_name = Set(Some(display_name.to_string()));
        
        // Update username (set to None if empty string or not provided)
        if let Some(ref username) = req.username {
            let username = username.trim();
            active_user.username = Set(if username.is_empty() { None } else { Some(username.to_string()) });
        }
        // If username not provided in request, keep existing value (for profile updates)

        // Update bio (set to None if empty string or not provided)
        if let Some(ref bio) = req.bio {
            let bio = bio.trim();
            active_user.bio = Set(if bio.is_empty() { None } else { Some(bio.to_string()) });
        }
        // If bio not provided in request, keep existing value (for profile updates)

        // Update profile picture (set to None if empty string or not provided)
        if let Some(ref url) = req.profile_picture_url {
            let url = url.trim();
            active_user.profile_picture = Set(if url.is_empty() { None } else { Some(url.to_string()) });
        }
        // If profile picture not provided in request, keep existing value (for profile updates)
        
        // Update background image (set to None if empty string or not provided)
        if let Some(ref url) = req.background_image_url {
            let url = url.trim();
            active_user.background_image = Set(if url.is_empty() { None } else { Some(url.to_string()) });
        }
        // If background image not provided in request, keep existing value (for profile updates)

        active_user.updated_at = Set(now.into());

        let updated_user = active_user.update(db).await?;

        Ok(SetupProfileResponse {
            user_id: updated_user.user_id,
            display_name: updated_user.display_name.unwrap_or_default(),
            username: updated_user.username,
            bio: updated_user.bio,
            profile_picture_url: updated_user.profile_picture,
            background_image_url: updated_user.background_image,
            updated_at: now,
        })
    }
}

// ============ Get Profile Use Case ============

pub struct GetProfileUseCase;

impl GetProfileUseCase {
    #[instrument(skip(db), fields(user_id = %user_id))]
    pub async fn execute(db: &DatabaseConnection, user_id: Uuid) -> AppResult<GetProfileResponse> {
        let user = users::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

        // Mask phone number (show only last 4 digits)
        let phone = &user.phone_number;
        let masked_phone = if phone.len() > 4 {
            format!("****{}", &phone[phone.len() - 4..])
        } else {
            "****".to_string()
        };

        Ok(GetProfileResponse {
            user_id: user.user_id,
            phone_number: masked_phone,
            display_name: user.display_name,
            username: user.username,
            bio: user.bio,
            profile_picture_url: user.profile_picture,
            background_image_url: user.background_image,
            created_at: user.created_at.into(),
            updated_at: user.updated_at.into(),
        })
    }
}

// ============ Setup PIN Use Case ============

pub struct SetupPinUseCase;

impl SetupPinUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        user_id: Uuid,
        req: SetupPinRequest,
    ) -> AppResult<SetupPinResponse> {
        // Validate input first
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        // Validate PIN
        if req.pin.len() < PIN_MIN_LENGTH || req.pin.len() > PIN_MAX_LENGTH {
            return Err(AppError::Validation(format!(
                "PIN must be between {} and {} characters",
                PIN_MIN_LENGTH,
                PIN_MAX_LENGTH
            )));
        }

        if req.pin != req.confirm_pin {
            return Err(AppError::Validation("PIN confirmation does not match".to_string()));
        }

        // Hash PIN with Argon2
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let pin_hash = argon2
            .hash_password(req.pin.as_bytes(), &salt)
            .map_err(|e| AppError::Cryptographic(format!("Failed to hash PIN: {}", e)))?
            .to_string();

        // Update user
        let user = users::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

        let now = Utc::now();
        let mut active_user: users::ActiveModel = user.into();
        active_user.pin_hash = Set(Some(pin_hash));
        active_user.registration_lock = Set(req.enable_registration_lock);
        active_user.pin_set_at = Set(Some(now.into()));
        active_user.updated_at = Set(now.into());

        if req.enable_registration_lock {
            // Registration lock never expires unless explicitly disabled
            active_user.registration_lock_expires_at = Set(None);
        }

        active_user.update(db).await?;

        Ok(SetupPinResponse {
            registration_lock_enabled: req.enable_registration_lock,
            message: if req.enable_registration_lock {
                "PIN set and Registration Lock enabled".to_string()
            } else {
                "PIN set successfully".to_string()
            },
        })
    }
}

// ============ Verify PIN Use Case ============

pub struct VerifyPinUseCase;

impl VerifyPinUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        redis_conn: &mut MultiplexedConnection,
        user_id: Uuid,
        req: VerifyPinRequest,
    ) -> AppResult<VerifyPinResponse> {
        // Check rate limit for PIN attempts
        let attempts_key = format!("pin_attempts:{}", user_id);
        let attempts: Option<u32> = redis_conn.get(&attempts_key).await?;

        if attempts.unwrap_or(0) >= 5 {
            // Get TTL to show remaining lockout time
            let ttl: i64 = redis_conn.ttl(&attempts_key).await?;
            let lockout_remaining_seconds = if ttl > 0 {
                Some(ttl as u64)
            } else {
                None
            };

            return Ok(VerifyPinResponse {
                verified: false,
                has_pin: true, // User has PIN but is locked out
                attempts_remaining: Some(0),
                lockout_remaining_seconds,
            });
        }

        // Get user
        let user = users::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

        // Check if user has a PIN
        let pin_hash = match user.pin_hash {
            Some(hash) => hash,
            None => {
                return Ok(VerifyPinResponse {
                    verified: false,
                    has_pin: false,
                    attempts_remaining: None,
                    lockout_remaining_seconds: None,
                });
            }
        };

        // Verify PIN
        let parsed_hash = PasswordHash::new(&pin_hash)
            .map_err(|e| AppError::Cryptographic(format!("Invalid PIN hash: {}", e)))?;
        let verified = Argon2::default()
            .verify_password(req.pin.as_bytes(), &parsed_hash)
            .is_ok();

        if verified {
            // Clear attempts on success
            redis_conn.del::<_, ()>(&attempts_key).await?;
        } else {
            // Increment failed attempts
            redis_conn.incr::<_, _, ()>(&attempts_key, 1).await?;
            redis_conn.expire::<_, ()>(&attempts_key, 3600).await?; // 1 hour
        }

        let current_attempts = if verified {
            0
        } else {
            attempts.unwrap_or(0) + 1
        };

        Ok(VerifyPinResponse {
            verified,
            has_pin: true, // We know the user has a PIN if we reached this point
            attempts_remaining: Some(5 - current_attempts),
            lockout_remaining_seconds: None, // Only set when locked out
        })
    }
}

// ============ Refresh Token Use Case ============

pub struct RefreshTokenUseCase;

impl RefreshTokenUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        config: &AuthConfig,
        req: RefreshTokenRequest,
    ) -> AppResult<RefreshTokenResponse> {
        // Decode and validate refresh token
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
        let validation = Validation::default();

        let token_data = decode::<Claims>(&req.refresh_token, &decoding_key, &validation)
            .map_err(|_| AppError::Authentication("Invalid or expired refresh token".to_string()))?;

        let claims = token_data.claims;

        if claims.token_type != "refresh" {
            return Err(AppError::Authentication("Invalid token type".to_string()));
        }

        // Verify device is still active
        let device = devices::Entity::find_by_id(claims.device_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        if !device.is_active {
            return Err(AppError::Authorization("Device is no longer active".to_string()));
        }

        let user_id: Uuid = claims.sub.parse()?;

        // Generate new tokens
        let (access_token, refresh_token) =
            VerifyOtpUseCase::generate_tokens(config, user_id, device.device_id)?;

        Ok(RefreshTokenResponse {
            access_token,
            refresh_token,
        })
    }
}

// ============ Create Linking Session Use Case ============

pub struct CreateLinkingSessionUseCase;

impl CreateLinkingSessionUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        device_id: i64,
    ) -> AppResult<CreateLinkingSessionResponse> {
        // Verify device exists and is primary
        let device = devices::Entity::find_by_id(device_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        if device.device_type != DEVICE_TYPE_PRIMARY {
            return Err(AppError::Authorization("Only primary devices can create linking sessions".to_string()));
        }

        if !device.is_active {
            return Err(AppError::Authorization("Device is not active".to_string()));
        }

        // Generate unique QR token
        let qr_code_token = format!(
            "{}:{}:{}",
            Uuid::new_v4(),
            device_id,
            Utc::now().timestamp()
        );

        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::minutes(LINKING_SESSION_EXPIRY_MINUTES);

        // Create session
        let session = device_linking_sessions::ActiveModel {
            session_id: Set(session_id),
            primary_device_id: Set(device_id),
            qr_code_token: Set(qr_code_token.clone()),
            status: Set(1), // Pending
            new_device_uuid: Set(None),
            new_device_name: Set(None),
            expires_at: Set(expires_at.into()),
            created_at: Set(Utc::now().into()),
            approved_at: Set(None),
        };

        session.insert(db).await?;

        // Generate QR code data (Base64 encoded JSON)
        let qr_data = serde_json::json!({
            "token": qr_code_token,
            "expires_at": expires_at.to_rfc3339(),
        });
        let qr_code_data = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            qr_data.to_string(),
        );

        Ok(CreateLinkingSessionResponse {
            session_id,
            qr_code_data,
            qr_code_token,
            expires_at,
        })
    }
}

// ============ Complete Linking Use Case (New Device) ============

pub struct CompleteLinkingUseCase;

impl CompleteLinkingUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        req: CompleteLinkingRequest,
    ) -> AppResult<CompleteLinkingResponse> {
        // Find session by token
        let session = device_linking_sessions::Entity::find()
            .filter(device_linking_sessions::Column::QrCodeToken.eq(&req.qr_code_token))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Invalid linking session".to_string()))?;

        // Check if session is still valid
        if !session.is_pending() {
            return Err(AppError::Validation("Linking session has expired or is no longer valid".to_string()));
        }

        // Update session with new device info
        let mut active_session: device_linking_sessions::ActiveModel = session.clone().into();
        active_session.new_device_uuid = Set(Some(req.device_uuid));
        active_session.new_device_name = Set(req.device_name.clone());
        active_session.update(db).await?;

        Ok(CompleteLinkingResponse {
            session_id: session.session_id,
            status: "pending_approval".to_string(),
            message: "Waiting for approval from primary device".to_string(),
        })
    }
}

// ============ Approve Linking Use Case (Primary Device) ============

pub struct ApproveLinkingUseCase;

impl ApproveLinkingUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        primary_device_id: i64,
        req: ApproveLinkingRequest,
    ) -> AppResult<ApproveLinkingResponse> {
        let txn = db.begin().await?;

        // Find session
        let session = device_linking_sessions::Entity::find_by_id(req.session_id)
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::NotFound("Session not found".to_string()))?;

        // Verify ownership
        if session.primary_device_id != primary_device_id {
            return Err(AppError::Authorization("Unauthorized".to_string()));
        }

        // Check if session is still pending
        if !session.is_pending() {
            return Err(AppError::Validation("Session has expired or is no longer valid".to_string()));
        }

        let new_device_uuid = session
            .new_device_uuid
            .ok_or_else(|| AppError::NotFound("No device waiting for approval".to_string()))?;

        let mut active_session: device_linking_sessions::ActiveModel = session.clone().into();

        if req.approve {
            // Get primary device to find user
            let primary_device = devices::Entity::find_by_id(primary_device_id)
                .one(&txn)
                .await?
                .ok_or_else(|| AppError::NotFound("Primary device not found".to_string()))?;

            // Generate Signal Keys for new device
            let (identity_key_pair, _) = generate_identity_keypair()
                .map_err(|e| AppError::Cryptographic(e.to_string()))?;
            let registration_id = generate_registration_id();
            let signed_prekey = generate_signed_prekey(&identity_key_pair, 1)
                .map_err(|e| AppError::Cryptographic(e.to_string()))?;
            let one_time_prekeys_list = generate_prekeys(1, 100)
                .map_err(|e| AppError::Cryptographic(e.to_string()))?;

            // Create linked device
            let new_device = devices::ActiveModel {
                user_id: Set(primary_device.user_id),
                device_uuid: Set(new_device_uuid),
                device_name: Set(session.new_device_name.clone()),
                platform: Set(3), // Default to Web for linked devices
                identity_key_public: Set(identity_key_pair.public_key),
                registration_id: Set(registration_id as i32),
                signed_prekey_id: Set(signed_prekey.id as i32),
                signed_prekey_public: Set(signed_prekey.public_key),
                signed_prekey_signature: Set(signed_prekey.signature),
                last_seen_at: Set(Utc::now().into()),
                created_at: Set(Utc::now().into()),
                device_type: Set(DEVICE_TYPE_LINKED),
                is_active: Set(true),
                linked_at: Set(Some(Utc::now().into())),
                linked_by_device_id: Set(Some(primary_device_id)),
                ..Default::default()
            };

            let new_device = new_device.insert(&txn).await?;

            // Insert One Time Prekeys
            for prekey in one_time_prekeys_list {
                let otpk = one_time_prekeys::ActiveModel {
                    device_id: Set(new_device.device_id),
                    prekey_id: Set(prekey.id as i32),
                    public_key: Set(prekey.public_key),
                };
                otpk.insert(&txn).await?;
            }

            // Update session status
            active_session.status = Set(2); // Approved
            active_session.approved_at = Set(Some(Utc::now().into()));
            active_session.update(&txn).await?;

            txn.commit().await?;

            Ok(ApproveLinkingResponse {
                session_id: req.session_id,
                new_device_id: Some(new_device.device_id),
                status: "approved".to_string(),
            })
        } else {
            // Reject
            active_session.status = Set(4); // Rejected
            active_session.update(&txn).await?;
            txn.commit().await?;

            Ok(ApproveLinkingResponse {
                session_id: req.session_id,
                new_device_id: None,
                status: "rejected".to_string(),
            })
        }
    }
}

// ============ List Devices Use Case ============

pub struct ListDevicesUseCase;

impl ListDevicesUseCase {
    #[instrument(skip(db), fields(user_id = %user_id))]
    pub async fn execute(db: &DatabaseConnection, user_id: Uuid) -> AppResult<ListDevicesResponse> {
        let devices_list = devices::Entity::find()
            .filter(devices::Column::UserId.eq(user_id))
            .filter(devices::Column::IsActive.eq(true))
            .order_by_asc(devices::Column::CreatedAt)
            .all(db)
            .await?;

        let devices: Vec<DeviceInfo> = devices_list
            .into_iter()
            .map(|d| DeviceInfo {
                device_id: d.device_id,
                device_uuid: d.device_uuid,
                device_name: d.device_name,
                platform: d.platform,
                device_type: if d.device_type == DEVICE_TYPE_PRIMARY {
                    "primary".to_string()
                } else {
                    "linked".to_string()
                },
                is_active: d.is_active,
                linked_at: d.linked_at.map(|dt| dt.with_timezone(&Utc)),
                last_seen_at: d.last_seen_at.with_timezone(&Utc),
                created_at: d.created_at.with_timezone(&Utc),
            })
            .collect();

        let total = devices.len();

        Ok(ListDevicesResponse { devices, total })
    }
}

// ============ Unlink Device Use Case ============

pub struct UnlinkDeviceUseCase;

impl UnlinkDeviceUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        user_id: Uuid,
        current_device_id: i64,
        target_device_id: i64,
    ) -> AppResult<UnlinkDeviceResponse> {
        // Find target device
        let device = devices::Entity::find_by_id(target_device_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        // Verify ownership
        if device.user_id != user_id {
            return Err(AppError::Authorization("Unauthorized".to_string()));
        }

        // Cannot unlink current device
        if device.device_id == current_device_id {
            return Err(AppError::Validation("Cannot unlink current device".to_string()));
        }

        // Deactivate device
        let mut active_device: devices::ActiveModel = device.into();
        active_device.is_active = Set(false);
        active_device.update(db).await?;

        Ok(UnlinkDeviceResponse {
            unlinked: true,
            message: "Device unlinked successfully".to_string(),
        })
    }
}

// ============ Check PIN Status Use Case ============

pub struct CheckPinStatusUseCase;

impl CheckPinStatusUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> AppResult<PinStatusResponse> {
        // Get user
        let user = users::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

        Ok(PinStatusResponse {
            has_pin: user.pin_hash.is_some(),
        })
    }
}

// ============ Skip PIN Setup Use Case ============

pub struct SkipPinSetupUseCase;

impl SkipPinSetupUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> AppResult<SkipPinSetupResponse> {
        // Get user
        let user = users::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

        // Update user to explicitly mark that PIN setup was skipped
        // We can use pin_set_at field with a special value to indicate skipped
        let mut active_user: users::ActiveModel = user.into();
        active_user.pin_hash = Set(None); // Explicitly no PIN
        active_user.registration_lock = Set(false); // Disable registration lock
        active_user.pin_set_at = Set(Some(Utc::now().into())); // Mark as "processed"
        active_user.updated_at = Set(Utc::now().into());

        active_user.update(db).await?;

        Ok(SkipPinSetupResponse {
            message: "PIN setup skipped successfully".to_string(),
        })
    }
}
