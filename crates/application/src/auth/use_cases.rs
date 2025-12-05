use crate::auth::dtos::{Claims, RequestOtpRequest, VerifyOtpRequest, VerifyOtpResponse};
use anyhow::Result;
use chrono::{Duration, Utc};
use core::entities::{devices, one_time_prekeys, users};
use infrastructure::crypto::signal::{
    generate_identity_keypair, generate_prekeys, generate_registration_id, generate_signed_prekey,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::Rng;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use uuid::Uuid;

pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub refresh_token_expiration: i64,
}

pub struct RequestOtpUseCase;

impl RequestOtpUseCase {
    pub async fn execute(
        redis_conn: &mut MultiplexedConnection,
        req: RequestOtpRequest,
    ) -> Result<String> {
        let otp: String = (0..6)
            .map(|_| rand::thread_rng().gen_range(0..10).to_string())
            .collect();
        let key = format!("otp:{}", req.phone_number);

        // Store in Redis with 180s expiration
        redis_conn.set_ex::<_, _, ()>(&key, &otp, 180).await?;

        // In a real app, we would send the OTP via SMS here.
        // For now, we return it so the API can log it or return it for testing.
        Ok(otp)
    }
}

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
            return Err(anyhow::anyhow!("Invalid or expired OTP"));
        }

        // Start transaction
        let txn = db.begin().await?;

        // Check if user exists
        let (user, is_new_user) = match users::Entity::find()
            .filter(users::Column::PhoneNumber.eq(&req.phone_number))
            .one(&txn)
            .await?
        {
            Some(u) => (u, false),
            None => {
                // Create new user
                let new_user = users::ActiveModel {
                    user_id: Set(Uuid::new_v4()),
                    phone_number: Set(req.phone_number.clone()),
                    phone_number_hash: Set(Vec::new()), // Placeholder
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
                };
                (new_user.insert(&txn).await?, true)
            }
        };

        // Generate Signal Keys
        let (identity_key_pair, _) = generate_identity_keypair()?;
        let registration_id = generate_registration_id();
        let signed_prekey = generate_signed_prekey(&identity_key_pair, 1)?;
        let one_time_prekeys_list = generate_prekeys(1, 100)?;

        // Create Device
        let device = devices::ActiveModel {
            user_id: Set(user.user_id),
            device_uuid: Set(req.device_uuid),
            device_name: Set(req.device_name.clone()),
            platform: Set(1), // Default platform
            identity_key_public: Set(identity_key_pair.public_key),
            registration_id: Set(registration_id as i32),
            signed_prekey_id: Set(signed_prekey.id as i32),
            signed_prekey_public: Set(signed_prekey.public_key),
            signed_prekey_signature: Set(signed_prekey.signature),
            last_seen_at: Set(Utc::now().into()),
            created_at: Set(Utc::now().into()),
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

        // Generate JWT
        let now = Utc::now();
        let access_claims = Claims {
            sub: user.user_id.to_string(),
            device_id: device.device_id,
            iat: now.timestamp(),
            exp: (now + Duration::minutes(15)).timestamp(),
            token_type: "access".to_string(),
        };

        let refresh_claims = Claims {
            sub: user.user_id.to_string(),
            device_id: device.device_id,
            iat: now.timestamp(),
            exp: (now + Duration::days(30)).timestamp(),
            token_type: "refresh".to_string(),
        };

        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let access_token = encode(&Header::default(), &access_claims, &encoding_key)?;
        let refresh_token = encode(&Header::default(), &refresh_claims, &encoding_key)?;

        Ok(VerifyOtpResponse {
            access_token,
            refresh_token,
            user_id: user.user_id,
            device_id: device.device_id,
            is_new_user,
        })
    }
}
