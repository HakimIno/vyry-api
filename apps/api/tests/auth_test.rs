use actix_web::{test, web, App};
use api::handlers::auth::{
    request_otp, verify_otp, RequestOtpRequest, VerifyOtpRequest, VerifyOtpResponse,
};
use infrastructure::database;
use redis::AsyncCommands;
use sea_orm::{ColumnTrait, DeleteResult, EntityTrait, QueryFilter};
use uuid::Uuid;

// Import core crate entities
// Note: Using explicit module path to avoid linter confusion with std::core
use core::entities::devices;
use core::entities::one_time_prekeys;
use core::entities::users;

#[actix_web::test]
async fn test_otp_flow_and_signal_keys() {
    // 1. Setup Test Environment
    dotenvy::from_filename(".env").ok();
    let config = api::config::Config::from_env().expect("Failed to load config");

    let db = database::init_database(&config.database_url)
        .await
        .expect("Failed to connect DB");
    let redis_client =
        redis::Client::open(config.redis_url.clone()).expect("Failed to create Redis client");
    let redis_conn = redis_client
        .get_multiplexed_tokio_connection()
        .await
        .expect("Failed to connect Redis");

    // Clean up previous test data
    let phone_number = "+66899999999";
    let _: DeleteResult = users::Entity::delete_many()
        .filter(users::Column::PhoneNumber.eq(phone_number))
        .exec(&db)
        .await
        .expect("Failed to clean up test data");

    // 2. Init App
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(redis_conn.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(request_otp)
            .service(verify_otp),
    )
    .await;

    // 3. Test Request OTP
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/request-otp")
        .set_json(RequestOtpRequest {
            phone_number: phone_number.to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // 4. Get OTP from Redis (Simulate user receiving SMS)
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to get async conn");
    let otp: String = conn
        .get(format!("otp:{}", phone_number))
        .await
        .expect("Failed to get OTP");
    assert_eq!(otp.len(), 6);

    // 5. Test Verify OTP
    let device_uuid = Uuid::new_v4();
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/verify-otp")
        .set_json(VerifyOtpRequest {
            phone_number: phone_number.to_string(),
            otp: otp.clone(),
            device_uuid,
            device_name: Some("Test Device".to_string()),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: VerifyOtpResponse = test::read_body_json(resp).await;
    assert!(!body.access_token.is_empty());
    assert!(!body.refresh_token.is_empty());
    assert!(body.is_new_user);

    // 6. Verify Database State (Signal Keys)

    // Check User
    let user = users::Entity::find_by_id(body.user_id)
        .one(&db)
        .await
        .expect("DB Error")
        .expect("User not found");
    assert_eq!(user.phone_number, phone_number);

    // Check Device & Signal Keys
    let device = devices::Entity::find_by_id(body.device_id)
        .one(&db)
        .await
        .expect("DB Error")
        .expect("Device not found");
    assert_eq!(device.user_id, user.user_id);
    assert!(!device.identity_key_public.is_empty());
    assert!(!device.signed_prekey_public.is_empty());
    assert!(!device.signed_prekey_signature.is_empty());

    // Check One-Time Prekeys (Should be 100)
    let prekeys = one_time_prekeys::Entity::find()
        .filter(one_time_prekeys::Column::DeviceId.eq(device.device_id))
        .all(&db)
        .await
        .expect("DB Error");

    assert_eq!(
        prekeys.len(),
        100,
        "Should generate exactly 100 one-time prekeys"
    );

    println!("✅ Test Passed: OTP Flow + Signal Key Generation Complete!");
}
