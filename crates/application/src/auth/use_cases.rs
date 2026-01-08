use crate::auth::dtos::*;
use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use core::entities::{device_linking_sessions, devices, one_time_prekeys, users};
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
    pub async fn execute(
        redis_conn: &mut MultiplexedConnection,
        req: RequestOtpRequest,
    ) -> Result<String> {
        // Check rate limit
        let attempts_key = format!("otp_attempts:{}", req.phone_number);
        let attempts: Option<u32> = redis_conn.get(&attempts_key).await?;

        if attempts.unwrap_or(0) >= OTP_MAX_ATTEMPTS {
            return Err(anyhow!("Too many OTP requests. Please try again later."));
        }

        // Generate 6-digit OTP
        let otp: String = (0..6)
            .map(|_| rand::thread_rng().gen_range(0..10).to_string())
            .collect();
        let key = format!("otp:{}", req.phone_number);

        // Store in Redis with expiration
        redis_conn
            .set_ex::<_, _, ()>(&key, &otp, OTP_EXPIRY_SECONDS)
            .await?;

        // Increment attempts counter
        redis_conn
            .incr::<_, _, ()>(&attempts_key, 1)
            .await?;
        redis_conn
            .expire::<_, ()>(&attempts_key, 600) // 10 minutes window
            .await?;

        // TODO: In production, send OTP via SMS provider
        // For now, return it for testing
        Ok(otp)
    }
}

// ============ Verify OTP Use Case ============

pub struct VerifyOtpUseCase;

impl VerifyOtpUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        redis_conn: &mut MultiplexedConnection,
        config: &AuthConfig,
        req: VerifyOtpRequest,
    ) -> Result<VerifyOtpResponse> {
        let key = format!("otp:{}", req.phone_number);
        let stored_otp: Option<String> = redis_conn.get(&key).await?;

        if stored_otp.is_none() || stored_otp.unwrap() != req.otp {
            return Err(anyhow!("Invalid or expired OTP"));
        }

        // Delete OTP after successful verification
        redis_conn.del::<_, ()>(&key).await?;

        // Start transaction
        let txn = db.begin().await?;

        // Check if user exists
        let existing_user = users::Entity::find()
            .filter(users::Column::PhoneNumber.eq(&req.phone_number))
            .one(&txn)
            .await?;

        let (user, is_new_user) = match existing_user {
            Some(u) => {
                // Existing user - kick old primary device if this is a new primary login
                Self::kick_old_primary_device(&txn, u.user_id).await?;
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
                (new_user.insert(&txn).await?, true)
            }
        };

        // Check if profile setup is required
        let requires_profile_setup = user.display_name.is_none();
        let requires_pin = user.registration_lock && user.pin_hash.is_some();

        // Generate Signal Keys
        let (identity_key_pair, _) = generate_identity_keypair()?;
        let registration_id = generate_registration_id();
        let signed_prekey = generate_signed_prekey(&identity_key_pair, 1)?;
        let one_time_prekeys_list = generate_prekeys(1, 100)?;

        // Check if device_uuid already exists (could be from previous failed registration)
        // If it exists, delete the old device and its prekeys to allow re-registration
        if let Some(existing_device) = devices::Entity::find()
            .filter(devices::Column::DeviceUuid.eq(req.device_uuid))
            .one(&txn)
            .await?
        {
            // Delete one-time prekeys first (due to foreign key constraint)
            one_time_prekeys::Entity::delete_many()
                .filter(one_time_prekeys::Column::DeviceId.eq(existing_device.device_id))
                .exec(&txn)
                .await?;
            
            // Delete the old device
            devices::Entity::delete_by_id(existing_device.device_id)
                .exec(&txn)
                .await?;
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

        let device = device.insert(&txn).await?;

        // Insert One Time Prekeys
        for prekey in one_time_prekeys_list {
            let otpk = one_time_prekeys::ActiveModel {
                device_id: Set(device.device_id),
                prekey_id: Set(prekey.id as i32),
                public_key: Set(prekey.public_key),
            };
            otpk.insert(&txn).await?;
        }

        txn.commit().await?;

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
    ) -> Result<()> {
        // Deactivate all existing primary devices for this user
        let old_devices = devices::Entity::find()
            .filter(devices::Column::UserId.eq(user_id))
            .filter(devices::Column::DeviceType.eq(DEVICE_TYPE_PRIMARY))
            .filter(devices::Column::IsActive.eq(true))
            .all(txn)
            .await?;

        for old_device in old_devices {
            let mut active_device: devices::ActiveModel = old_device.into();
            active_device.is_active = Set(false);
            active_device.update(txn).await?;
        }

        Ok(())
    }

    fn generate_tokens(
        config: &AuthConfig,
        user_id: Uuid,
        device_id: i64,
    ) -> Result<(String, String)> {
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
        let access_token = encode(&Header::default(), &access_claims, &encoding_key)?;
        let refresh_token = encode(&Header::default(), &refresh_claims, &encoding_key)?;

        Ok((access_token, refresh_token))
    }
}

// ============ Setup Profile Use Case ============

pub struct SetupProfileUseCase;

impl SetupProfileUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        user_id: Uuid,
        req: SetupProfileRequest,
    ) -> Result<SetupProfileResponse> {
        // Validate display name
        if req.display_name.trim().is_empty() {
            return Err(anyhow!("Display name cannot be empty"));
        }

        if req.display_name.len() > 100 {
            return Err(anyhow!("Display name too long (max 100 characters)"));
        }

        // Find and update user
        let user = users::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        let now = Utc::now();
        let mut active_user: users::ActiveModel = user.into();
        active_user.display_name = Set(Some(req.display_name.clone()));
        active_user.profile_picture = Set(req.profile_picture_url.clone());
        active_user.updated_at = Set(now.into());

        let updated_user = active_user.update(db).await?;

        Ok(SetupProfileResponse {
            user_id: updated_user.user_id,
            display_name: updated_user.display_name.unwrap_or_default(),
            profile_picture_url: updated_user.profile_picture,
            updated_at: now,
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
    ) -> Result<SetupPinResponse> {
        // Validate PIN
        if req.pin.len() < PIN_MIN_LENGTH || req.pin.len() > PIN_MAX_LENGTH {
            return Err(anyhow!(
                "PIN must be between {} and {} characters",
                PIN_MIN_LENGTH,
                PIN_MAX_LENGTH
            ));
        }

        if req.pin != req.confirm_pin {
            return Err(anyhow!("PIN confirmation does not match"));
        }

        // Hash PIN with Argon2
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let pin_hash = argon2
            .hash_password(req.pin.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash PIN: {}", e))?
            .to_string();

        // Update user
        let user = users::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

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
    ) -> Result<VerifyPinResponse> {
        // Check rate limit for PIN attempts
        let attempts_key = format!("pin_attempts:{}", user_id);
        let attempts: Option<u32> = redis_conn.get(&attempts_key).await?;

        if attempts.unwrap_or(0) >= 5 {
            return Ok(VerifyPinResponse {
                verified: false,
                attempts_remaining: Some(0),
            });
        }

        // Get user
        let user = users::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        let pin_hash = user.pin_hash.ok_or_else(|| anyhow!("No PIN set"))?;

        // Verify PIN
        let parsed_hash = PasswordHash::new(&pin_hash)
            .map_err(|e| anyhow!("Invalid PIN hash: {}", e))?;
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
            attempts_remaining: Some(5 - current_attempts),
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
    ) -> Result<RefreshTokenResponse> {
        // Decode and validate refresh token
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
        let validation = Validation::default();

        let token_data = decode::<Claims>(&req.refresh_token, &decoding_key, &validation)
            .map_err(|_| anyhow!("Invalid or expired refresh token"))?;

        let claims = token_data.claims;

        if claims.token_type != "refresh" {
            return Err(anyhow!("Invalid token type"));
        }

        // Verify device is still active
        let device = devices::Entity::find_by_id(claims.device_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Device not found"))?;

        if !device.is_active {
            return Err(anyhow!("Device is no longer active"));
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
    ) -> Result<CreateLinkingSessionResponse> {
        // Verify device exists and is primary
        let device = devices::Entity::find_by_id(device_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Device not found"))?;

        if device.device_type != DEVICE_TYPE_PRIMARY {
            return Err(anyhow!("Only primary devices can create linking sessions"));
        }

        if !device.is_active {
            return Err(anyhow!("Device is not active"));
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
    ) -> Result<CompleteLinkingResponse> {
        // Find session by token
        let session = device_linking_sessions::Entity::find()
            .filter(device_linking_sessions::Column::QrCodeToken.eq(&req.qr_code_token))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Invalid linking session"))?;

        // Check if session is still valid
        if !session.is_pending() {
            return Err(anyhow!("Linking session has expired or is no longer valid"));
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
    ) -> Result<ApproveLinkingResponse> {
        let txn = db.begin().await?;

        // Find session
        let session = device_linking_sessions::Entity::find_by_id(req.session_id)
            .one(&txn)
            .await?
            .ok_or_else(|| anyhow!("Session not found"))?;

        // Verify ownership
        if session.primary_device_id != primary_device_id {
            return Err(anyhow!("Unauthorized"));
        }

        // Check if session is still pending
        if !session.is_pending() {
            return Err(anyhow!("Session has expired or is no longer valid"));
        }

        let new_device_uuid = session
            .new_device_uuid
            .ok_or_else(|| anyhow!("No device waiting for approval"))?;

        let mut active_session: device_linking_sessions::ActiveModel = session.clone().into();

        if req.approve {
            // Get primary device to find user
            let primary_device = devices::Entity::find_by_id(primary_device_id)
                .one(&txn)
                .await?
                .ok_or_else(|| anyhow!("Primary device not found"))?;

            // Generate Signal Keys for new device
            let (identity_key_pair, _) = generate_identity_keypair()?;
            let registration_id = generate_registration_id();
            let signed_prekey = generate_signed_prekey(&identity_key_pair, 1)?;
            let one_time_prekeys_list = generate_prekeys(1, 100)?;

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
    pub async fn execute(db: &DatabaseConnection, user_id: Uuid) -> Result<ListDevicesResponse> {
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
    ) -> Result<UnlinkDeviceResponse> {
        // Find target device
        let device = devices::Entity::find_by_id(target_device_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Device not found"))?;

        // Verify ownership
        if device.user_id != user_id {
            return Err(anyhow!("Unauthorized"));
        }

        // Cannot unlink current device
        if device.device_id == current_device_id {
            return Err(anyhow!("Cannot unlink current device"));
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
