#[cfg(test)]
mod tests {
    use crate::auth::dtos::*;
    use crate::AppError;
    use validator::Validate;
    use uuid::Uuid;

    #[test]
    fn test_request_otp_validation() {
        // Valid phone number
        let valid_req = RequestOtpRequest {
            phone_number: "+66812345678".to_string(),
        };
        assert!(valid_req.validate().is_ok());

        // Invalid phone number (no +)
        let invalid_req = RequestOtpRequest {
            phone_number: "66812345678".to_string(),
        };
        assert!(invalid_req.validate().is_err());

        // Invalid phone number (too short)
        let invalid_req2 = RequestOtpRequest {
            phone_number: "+66".to_string(),
        };
        assert!(invalid_req2.validate().is_err());
    }

    #[test]
    fn test_verify_otp_validation() {
        // Valid request
        let valid_req = VerifyOtpRequest {
            phone_number: "+66812345678".to_string(),
            otp: "123456".to_string(),
            device_uuid: Uuid::new_v4(),
            device_name: Some("Test Device".to_string()),
            platform: Some(1),
        };
        assert!(valid_req.validate().is_ok());

        // Invalid OTP length
        let invalid_req = VerifyOtpRequest {
            phone_number: "+66812345678".to_string(),
            otp: "12345".to_string(), // Too short
            device_uuid: Uuid::new_v4(),
            device_name: None,
            platform: None,
        };
        assert!(invalid_req.validate().is_err());

        // Invalid platform
        let invalid_req2 = VerifyOtpRequest {
            phone_number: "+66812345678".to_string(),
            otp: "123456".to_string(),
            device_uuid: Uuid::new_v4(),
            device_name: None,
            platform: Some(5), // Invalid platform
        };
        assert!(invalid_req2.validate().is_err());
    }

    #[test]
    fn test_setup_profile_validation() {
        // Valid request
        let valid_req = SetupProfileRequest {
            display_name: "John Doe".to_string(),
            username: Some("johndoe".to_string()),
            bio: Some("Test bio".to_string()),
            profile_picture_url: Some("https://example.com/pic.jpg".to_string()),
            background_image_url: None,
        };
        assert!(valid_req.validate().is_ok());

        // Display name too short
        let invalid_req = SetupProfileRequest {
            display_name: "J".to_string(), // Too short
            username: None,
            bio: None,
            profile_picture_url: None,
            background_image_url: None,
        };
        assert!(invalid_req.validate().is_err());

        // Invalid username format
        let invalid_req2 = SetupProfileRequest {
            display_name: "John Doe".to_string(),
            username: Some("jo".to_string()), // Too short
            bio: None,
            profile_picture_url: None,
            background_image_url: None,
        };
        assert!(invalid_req2.validate().is_err());

        // Invalid URL
        let invalid_req3 = SetupProfileRequest {
            display_name: "John Doe".to_string(),
            username: None,
            bio: None,
            profile_picture_url: Some("not-a-url".to_string()),
            background_image_url: None,
        };
        assert!(invalid_req3.validate().is_err());
    }

    #[test]
    fn test_setup_pin_validation() {
        // Valid request
        let valid_req = SetupPinRequest {
            pin: "1234".to_string(),
            confirm_pin: "1234".to_string(),
            enable_registration_lock: false,
        };
        assert!(valid_req.validate().is_ok());

        // PINs don't match
        let invalid_req = SetupPinRequest {
            pin: "1234".to_string(),
            confirm_pin: "5678".to_string(),
            enable_registration_lock: false,
        };
        assert!(invalid_req.validate().is_err());

        // PIN too short
        let invalid_req2 = SetupPinRequest {
            pin: "123".to_string(),
            confirm_pin: "123".to_string(),
            enable_registration_lock: false,
        };
        assert!(invalid_req2.validate().is_err());
    }

    #[test]
    fn test_app_error_status_codes() {
        let auth_error = AppError::Authentication("test".to_string());
        assert_eq!(auth_error.status_code(), 401);
        assert_eq!(auth_error.error_code(), "AUTHENTICATION_FAILED");

        let validation_error = AppError::Validation("test".to_string());
        assert_eq!(validation_error.status_code(), 400);
        assert_eq!(validation_error.error_code(), "VALIDATION_ERROR");

        let rate_limit_error = AppError::RateLimitExceeded("test".to_string());
        assert_eq!(rate_limit_error.status_code(), 429);
        assert_eq!(rate_limit_error.error_code(), "RATE_LIMITED");
        assert_eq!(rate_limit_error.retry_after_seconds(), Some(60));
    }
}
