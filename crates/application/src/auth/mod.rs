pub mod dtos;
pub mod use_cases;
mod validation;

pub use validation::{PHONE_REGEX, USERNAME_REGEX, validate_phone_number, validate_username};

// Re-export all use cases for easier imports
pub use use_cases::{
    ApproveLinkingUseCase, CheckPinStatusUseCase, CompleteLinkingUseCase, CreateLinkingSessionUseCase,
    GetProfileUseCase, ListDevicesUseCase, RefreshTokenUseCase, RequestOtpUseCase,
    SetupPinUseCase, SetupProfileUseCase, SkipPinSetupUseCase, UnlinkDeviceUseCase, VerifyOtpUseCase,
    VerifyPinUseCase,
};