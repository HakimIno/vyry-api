# Delivery Status Specification

## Requirements

### Requirement: Message Delivery Status
The system SHALL track message delivery status per recipient device.

#### Scenario: Mark message as delivered
- **WHEN** a recipient device receives a message
- **THEN** the device sends DeliveryStatus with status Delivered
- **AND** the system updates message_deliveries.delivered_at
- **AND** forwards status update to original sender
- **AND** sender sees delivered indicator

#### Scenario: Mark message as read
- **WHEN** a recipient opens and reads a message
- **THEN** the device sends DeliveryStatus with status Read
- **AND** the system updates message_deliveries.read_at
- **AND** forwards status update to original sender
- **AND** sender sees read indicator

#### Scenario: Delivery status for offline recipient
- **WHEN** a message is sent to offline recipient
- **THEN** delivered_at remains NULL
- **AND** read_at remains NULL
- **AND** status updates when recipient comes online and syncs

#### Scenario: Delivery status for multiple devices
- **WHEN** a user has multiple devices
- **THEN** each device has separate delivery status
- **AND** system tracks status per device
- **AND** sender sees status for each device

### Requirement: Delivery Status Forwarding
The system SHALL forward delivery status updates to message senders.

#### Scenario: Forward delivered status
- **WHEN** a recipient marks message as delivered
- **THEN** the system forwards DeliveryStatus to sender
- **AND** sender receives update via WebSocket if online
- **AND** update includes message_id and status

#### Scenario: Forward read status
- **WHEN** a recipient marks message as read
- **THEN** the system forwards DeliveryStatus to sender
- **AND** sender receives read indicator
- **AND** update includes message_id and status

#### Scenario: Forward to offline sender
- **WHEN** delivery status is updated but sender is offline
- **THEN** the system stores status update
- **AND** sender receives update when reconnecting
- **AND** status is included in sync response

### Requirement: Delivery Status Query
The system SHALL allow users to query delivery status for their sent messages.

#### Scenario: Get delivery status for message
- **WHEN** a user queries delivery status for sent message
- **THEN** the system returns status for all recipient devices
- **AND** includes delivered_at and read_at timestamps
- **AND** indicates which devices have received/read

#### Scenario: Get status for non-existent message
- **WHEN** a user queries status for non-existent message
- **THEN** the system returns 404 Not Found
- **AND** includes error code INVALID_REQUEST

#### Scenario: Get status for other user's message
- **WHEN** a user queries status for message they didn't send
- **THEN** the system returns 403 Forbidden
- **AND** includes error code FORBIDDEN

### Requirement: Read Receipts
The system SHALL support read receipts for message confirmation.

#### Scenario: Enable read receipts
- **WHEN** a user enables read receipts
- **THEN** the system sends read status for all received messages
- **AND** recipients see when messages are read

#### Scenario: Disable read receipts
- **WHEN** a user disables read receipts
- **THEN** the system does not send read status
- **AND** recipients do not see read indicators

#### Scenario: Read receipt privacy
- **WHEN** read receipts are disabled
- **THEN** the system respects privacy setting
- **AND** does not send read status even if message is read

### Requirement: Delivery Status for Group Messages
The system SHALL track delivery status for group messages.

#### Scenario: Track status per group member
- **WHEN** a message is sent to group
- **THEN** the system creates delivery record for each member
- **AND** tracks status per member device
- **AND** sender sees status for all members

#### Scenario: Group read status
- **WHEN** group members read message
- **THEN** the system updates read_at for each member
- **AND** sender sees read count
- **AND** can see which members have read

#### Scenario: Partial group delivery
- **WHEN** some group members are offline
- **THEN** the system tracks delivered status separately
- **AND** sender sees mixed status (some delivered, some pending)
