# Chat/Messaging Specification

## Requirements

### Requirement: Send Message

The system SHALL allow users to send end-to-end encrypted messages through WebSocket.

#### Scenario: Send message to online recipient

- **WHEN** a user sends a SignalMessage through WebSocket
- **AND** the recipient device is online
- **THEN** the system stores the message in database
- **AND** creates message_deliveries record
- **AND** immediately forwards the message to recipient via WebSocket
- **AND** uses deduplication based on client_message_id

#### Scenario: Send message to offline recipient

- **WHEN** a user sends a SignalMessage through WebSocket
- **AND** the recipient device is offline
- **THEN** the system stores the message in database
- **AND** creates message_deliveries record with content
- **AND** marks delivered_at as NULL
- **AND** message will be synced when recipient comes online

#### Scenario: Message deduplication

- **WHEN** a user sends a message with duplicate client_message_id
- **THEN** the system detects the duplicate
- **AND** returns existing message_id without creating new record
- **AND** prevents duplicate delivery

#### Scenario: Invalid conversation

- **WHEN** a user sends a message to non-existent conversation
- **THEN** the system returns error response
- **AND** includes error code INVALID_REQUEST

#### Scenario: Unauthorized message send

- **WHEN** a user attempts to send message without valid JWT
- **THEN** the system rejects WebSocket connection
- **AND** returns 401 Unauthorized

### Requirement: Message Sync

The system SHALL allow devices to sync offline messages when reconnecting.

#### Scenario: Sync all offline messages

- **WHEN** a device sends SyncRequest without last_message_id
- **THEN** the system queries all undelivered messages for that device
- **AND** returns SyncResponse with all messages
- **AND** includes encrypted content for each message

#### Scenario: Incremental sync

- **WHEN** a device sends SyncRequest with last_message_id
- **THEN** the system queries messages after the specified ID
- **AND** returns only new messages
- **AND** reduces bandwidth usage

#### Scenario: Sync empty result

- **WHEN** a device requests sync but has no pending messages
- **THEN** the system returns empty messages array
- **AND** indicates successful sync completion

#### Scenario: Sync with pagination

- **WHEN** a device has many pending messages
- **THEN** the system may paginate results
- **AND** client can request next batch with updated last_message_id

### Requirement: Key Management (Signal Protocol)

The system SHALL support X3DH Key Distribution.

#### Scenario: Upload Keys

- **WHEN** a user registers or refreshes keys
- **THEN** the system stores Identity Key, Signed PreKey, and One-Time PreKeys.
- **AND** validates signature of Signed PreKey.

#### Scenario: Fetch PreKey Bundle

- **WHEN** a user wants to send a message to a recipient
- **THEN** system returns PreKey Bundle:
  - Identity Key (Public)
  - Signed PreKey (Public + Signature)
  - One One-Time PreKey (Public, if available)
  - Registration ID
  - Device ID

### Requirement: Message Storage

The system SHALL store messages using Store & Forward pattern.

#### Scenario: Store message with delivery records

- **WHEN** a message is sent
- **THEN** the system creates message record in messages table
- **AND** creates message_deliveries record for each recipient device
- **AND** stores encrypted content per device in message_deliveries
- **AND** uses database transaction for atomicity

#### Scenario: Store message metadata

- **WHEN** a message is stored
- **THEN** the system records sender_user_id and sender_device_id
- **AND** records conversation_id
- **AND** records sent_at timestamp
- **AND** stores client_message_id for deduplication

### Requirement: Message Encryption

The system SHALL support end-to-end encrypted message content.

#### Scenario: Encrypted message delivery

- **WHEN** a message is sent
- **THEN** the content is encrypted using Signal Protocol
- **AND** each recipient device receives device-specific encrypted content
- **AND** server cannot decrypt message content
- **AND** encryption uses AES-GCM with unique IV per message

#### Scenario: Message content format

- **WHEN** a message is sent
- **THEN** content is stored as Vec<u8> (binary encrypted blob)
- **AND** IV is stored separately for decryption
- **AND** supports variable message sizes

### Requirement: Message Types

The system SHALL support different message types.

#### Scenario: Text message

- **WHEN** a user sends a text message
- **THEN** message_type is set to 1 (Signal Message)
- **AND** content contains encrypted text payload

#### Scenario: Message with attachment

- **WHEN** a user sends message with attachment
- **THEN** attachment_url is stored
- **AND** thumbnail_url is stored if available
- **AND** content may contain encrypted metadata

#### Scenario: Reply message

- **WHEN** a user sends reply to another message
- **THEN** reply_to_message_id is set
- **AND** links to original message

### Requirement: Message Acknowledgment

The system SHALL support message acknowledgment for delivery confirmation.

#### Scenario: Acknowledge message receipt

- **WHEN** a device receives a message
- **THEN** the device can send Ack message
- **AND** server processes the acknowledgment
- **AND** may update delivery status

#### Scenario: Missing acknowledgment

- **WHEN** a message is sent but not acknowledged
- **THEN** the system maintains message in pending state
- **AND** message will be included in next sync
