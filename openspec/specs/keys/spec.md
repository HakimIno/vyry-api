# Signal Protocol Keys Specification

## Requirements

### Requirement: Generate Signal Protocol Keys
The system SHALL generate Signal Protocol keys for each device during registration.

#### Scenario: Generate keys for new device
- **WHEN** a new device is registered
- **THEN** the system generates identity key pair (ed25519)
- **AND** generates signed prekey (x25519)
- **AND** generates 100 one-time prekeys (x25519)
- **AND** generates registration ID
- **AND** stores all keys securely in database

#### Scenario: Key generation uniqueness
- **WHEN** keys are generated for multiple devices
- **THEN** each device has unique identity key pair
- **AND** each device has unique prekeys
- **AND** keys are cryptographically secure

### Requirement: Retrieve Prekey Bundle
The system SHALL provide prekey bundles for establishing encrypted sessions.

#### Scenario: Get prekey bundle for user device
- **WHEN** a user requests prekey bundle for recipient device
- **THEN** the system retrieves identity key, signed prekey, and one prekey
- **AND** returns prekey bundle
- **AND** marks the one-time prekey as used
- **AND** removes used prekey from available pool

#### Scenario: Get prekey bundle for non-existent device
- **WHEN** a user requests prekey bundle for non-existent device
- **THEN** the system returns 404 Not Found
- **AND** includes error code INVALID_REQUEST

#### Scenario: Get prekey bundle without authentication
- **WHEN** an unauthenticated user requests prekey bundle
- **THEN** the system returns 401 Unauthorized
- **AND** includes error code UNAUTHORIZED

### Requirement: Prekey Management
The system SHALL manage one-time prekeys to ensure availability.

#### Scenario: Replenish prekeys
- **WHEN** a device's prekey count drops below threshold
- **THEN** the system generates new prekeys
- **AND** maintains pool of 100 prekeys
- **AND** ensures keys are available for new sessions

#### Scenario: Prekey exhaustion
- **WHEN** a device runs out of prekeys
- **THEN** the system generates new prekeys immediately
- **AND** notifies device to upload new prekeys
- **AND** prevents session establishment failure

### Requirement: Signed Prekey Rotation
The system SHALL support signed prekey rotation for security.

#### Scenario: Rotate signed prekey
- **WHEN** a device rotates signed prekey
- **THEN** the system generates new signed prekey
- **AND** signs with identity key
- **AND** updates device record
- **AND** invalidates old signed prekey

#### Scenario: Signed prekey expiration
- **WHEN** a signed prekey expires
- **THEN** the system requires new signed prekey
- **AND** prevents use of expired prekey

### Requirement: Identity Key Management
The system SHALL securely manage identity keys.

#### Scenario: Store identity key
- **WHEN** identity key is generated
- **THEN** the system stores public key in database
- **AND** private key remains on device only
- **AND** public key is used for verification

#### Scenario: Identity key verification
- **WHEN** a message is received
- **THEN** the system verifies signature using identity key
- **AND** ensures message authenticity

### Requirement: Session Key Establishment
The system SHALL support Signal Protocol session establishment.

#### Scenario: Create session from prekey bundle
- **WHEN** a user receives prekey bundle
- **THEN** the client establishes session using Signal Protocol
- **AND** session is stored in signal_sessions table
- **AND** enables encrypted message exchange

#### Scenario: Session key derivation
- **WHEN** a session is established
- **THEN** session keys are derived using HKDF
- **AND** keys are unique per session
- **AND** keys enable AES-GCM encryption

### Requirement: Key Distribution Message
The system SHALL support sender key distribution for group messages.

#### Scenario: Distribute sender key
- **WHEN** a user sends first message to group
- **THEN** the system includes sender_key_distribution in message
- **AND** recipients can establish group session
- **AND** enables efficient group encryption

#### Scenario: Use existing sender key
- **WHEN** a user sends subsequent group message
- **THEN** the system uses existing sender key
- **AND** does not include distribution data
- **AND** reduces message size
