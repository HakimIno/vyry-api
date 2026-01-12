# Authentication Specification

## Requirements

### Requirement: OTP Request
The system SHALL allow users to request a one-time password (OTP) for phone number verification.

#### Scenario: Successful OTP request
- **WHEN** a user provides a valid phone number
- **THEN** the system generates a 6-digit OTP
- **AND** stores it in Redis with 180 seconds expiration
- **AND** returns success response with expiration time
- **AND** logs the OTP for testing purposes

#### Scenario: Rate limited OTP request
- **WHEN** a user requests OTP more than the allowed limit within a time window
- **THEN** the system returns 429 Too Many Requests
- **AND** includes retry_after_seconds in the response
- **AND** prevents OTP generation

#### Scenario: Invalid phone number format
- **WHEN** a user provides an invalid phone number format
- **THEN** the system returns 400 Bad Request
- **AND** includes error code INVALID_REQUEST

### Requirement: OTP Verification
The system SHALL verify OTP codes and issue temporary authentication tokens.

#### Scenario: Successful OTP verification
- **WHEN** a user provides correct OTP within expiration time
- **THEN** the system validates the OTP from Redis
- **AND** issues a temporary token (not full JWT)
- **AND** returns token in response
- **AND** deletes the OTP from Redis

#### Scenario: Invalid OTP
- **WHEN** a user provides incorrect OTP
- **THEN** the system returns 400 Bad Request
- **AND** includes error code INVALID_OTP

#### Scenario: Expired OTP
- **WHEN** a user provides OTP after expiration
- **THEN** the system returns 400 Bad Request
- **AND** includes error code INVALID_OTP

### Requirement: Profile Setup
The system SHALL allow users to set up their profile after OTP verification.

#### Scenario: Successful profile setup
- **WHEN** a user provides valid profile data with JWT token
- **THEN** the system validates display name (2-100 characters)
- **AND** validates username if provided (3-30 characters, alphanumeric with underscore/hyphen)
- **AND** validates bio if provided (max 500 characters)
- **AND** validates profile picture URL if provided (valid HTTP/HTTPS URL, max 2048 characters)
- **AND** checks username uniqueness if provided
- **AND** updates user profile information
- **AND** returns updated profile data

#### Scenario: Profile setup with username
- **WHEN** a user provides valid username
- **THEN** the system validates username format
- **AND** checks username is not already taken
- **AND** stores username if available

#### Scenario: Profile setup without valid token
- **WHEN** a user attempts profile setup without valid JWT token
- **THEN** the system returns 401 Unauthorized
- **AND** includes error code UNAUTHORIZED

#### Scenario: Invalid display name
- **WHEN** a user provides invalid display name (empty, too short, or too long)
- **THEN** the system returns 400 Bad Request
- **AND** includes error code INVALID_REQUEST
- **AND** provides specific validation error message

#### Scenario: Invalid username format
- **WHEN** a user provides username with invalid format
- **THEN** the system returns 400 Bad Request
- **AND** includes error code INVALID_REQUEST
- **AND** indicates allowed characters and length requirements

#### Scenario: Duplicate username
- **WHEN** a user provides username that is already taken
- **THEN** the system returns 400 Bad Request
- **AND** includes error code INVALID_REQUEST
- **AND** indicates username is not available

#### Scenario: Invalid profile picture URL
- **WHEN** a user provides invalid profile picture URL format
- **THEN** the system returns 400 Bad Request
- **AND** includes error code INVALID_REQUEST
- **AND** indicates URL must be valid HTTP/HTTPS URL

### Requirement: PIN Setup
The system SHALL allow users to set up a PIN for additional security.

#### Scenario: Successful PIN setup
- **WHEN** a user provides a valid PIN with authenticated request
- **THEN** the system hashes the PIN using Argon2
- **AND** stores the hash in the user record
- **AND** returns success response

#### Scenario: PIN setup without authentication
- **WHEN** a user attempts PIN setup without valid JWT token
- **THEN** the system returns 401 Unauthorized
- **AND** includes error code UNAUTHORIZED

#### Scenario: Weak PIN
- **WHEN** a user provides a PIN that doesn't meet security requirements
- **THEN** the system returns 400 Bad Request
- **AND** includes error code INVALID_REQUEST

### Requirement: PIN Verification
The system SHALL verify PIN for sensitive operations.

#### Scenario: Successful PIN verification
- **WHEN** a user provides correct PIN
- **THEN** the system verifies against stored Argon2 hash
- **AND** returns success response

#### Scenario: Incorrect PIN
- **WHEN** a user provides incorrect PIN
- **THEN** the system returns 401 Unauthorized
- **AND** includes error code UNAUTHORIZED
- **AND** applies rate limiting to prevent brute force

### Requirement: Token Refresh
The system SHALL allow users to refresh expired access tokens.

#### Scenario: Successful token refresh
- **WHEN** a user provides valid refresh token
- **THEN** the system validates the refresh token
- **AND** issues new access token and refresh token
- **AND** invalidates old refresh token

#### Scenario: Invalid refresh token
- **WHEN** a user provides invalid or expired refresh token
- **THEN** the system returns 401 Unauthorized
- **AND** includes error code INVALID_TOKEN

### Requirement: Device Linking
The system SHALL support linking additional devices to a user account.

#### Scenario: Create linking session
- **WHEN** an authenticated user requests to link a new device
- **THEN** the system creates a linking session
- **AND** generates a one-time linking token
- **AND** returns QR code data or linking token
- **AND** sets session expiration time

#### Scenario: Complete device linking
- **WHEN** a new device provides valid linking token
- **THEN** the system validates the linking session
- **AND** creates device record
- **AND** generates Signal Protocol keys for new device
- **AND** requires approval from existing device

#### Scenario: Approve device linking
- **WHEN** an existing device approves the linking request
- **THEN** the system activates the new device
- **AND** sends notification to new device
- **AND** closes the linking session

#### Scenario: Expired linking session
- **WHEN** a device attempts to complete linking after session expiration
- **THEN** the system returns 400 Bad Request
- **AND** includes error code INVALID_REQUEST

### Requirement: Device Management
The system SHALL allow users to view and manage their devices.

#### Scenario: List devices
- **WHEN** an authenticated user requests device list
- **THEN** the system returns all devices for the user
- **AND** includes device metadata (name, type, last active)
- **AND** excludes sensitive key information

#### Scenario: Unlink device
- **WHEN** an authenticated user requests to unlink a device
- **THEN** the system removes the device record
- **AND** invalidates all sessions for that device
- **AND** prevents further authentication from that device

#### Scenario: Unlink own device
- **WHEN** a user attempts to unlink their only device
- **THEN** the system returns 400 Bad Request
- **AND** includes error code INVALID_REQUEST
- **AND** prevents account lockout
