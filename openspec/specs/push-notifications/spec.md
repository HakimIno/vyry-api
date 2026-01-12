# Push Notifications Specification

## Requirements

### Requirement: Register Push Token
The system SHALL allow devices to register push notification tokens.

#### Scenario: Register FCM token
- **WHEN** a device registers FCM push token
- **THEN** the system stores token in push_tokens table
- **AND** associates token with user_id and device_id
- **AND** stores platform type (Android)
- **AND** enables push notifications for device

#### Scenario: Register APNS token
- **WHEN** an iOS device registers APNS push token
- **THEN** the system stores token in push_tokens table
- **AND** associates token with user_id and device_id
- **AND** stores platform type (iOS)
- **AND** enables push notifications for device

#### Scenario: Update existing token
- **WHEN** a device registers new token for existing device
- **THEN** the system updates existing token record
- **AND** invalidates old token
- **AND** uses new token for notifications

#### Scenario: Register token without authentication
- **WHEN** an unauthenticated device attempts to register token
- **THEN** the system returns 401 Unauthorized
- **AND** includes error code UNAUTHORIZED

### Requirement: Send Push Notification
The system SHALL send push notifications for offline message delivery.

#### Scenario: Send notification for new message
- **WHEN** a message is sent to offline recipient
- **THEN** the system queries push_tokens for recipient devices
- **AND** sends push notification to each device
- **AND** notification includes message metadata (not content)
- **AND** notification triggers app wake-up

#### Scenario: Send notification for group message
- **WHEN** a group message is sent
- **THEN** the system sends notifications to all offline members
- **AND** notification indicates group and sender
- **AND** does not include message content

#### Scenario: Skip notification for online device
- **WHEN** a message is sent and recipient is online
- **THEN** the system does not send push notification
- **AND** message is delivered via WebSocket
- **AND** reduces unnecessary notifications

### Requirement: Push Notification Content
The system SHALL format push notifications appropriately.

#### Scenario: Notification without content
- **WHEN** push notification is sent
- **THEN** notification does not include message content
- **AND** maintains end-to-end encryption
- **AND** shows generic "New message" text

#### Scenario: Notification with sender info
- **WHEN** push notification is sent
- **THEN** notification includes sender name or phone number
- **AND** includes conversation name (for groups)
- **AND** enables user to identify message source

#### Scenario: Notification metadata
- **WHEN** push notification is sent
- **THEN** notification includes conversation_id
- **AND** includes message_id for app routing
- **AND** enables app to fetch message on open

### Requirement: Push Notification Delivery
The system SHALL ensure reliable push notification delivery.

#### Scenario: Handle invalid token
- **WHEN** push notification fails due to invalid token
- **THEN** the system marks token as invalid
- **AND** removes token from active list
- **AND** device must re-register token

#### Scenario: Handle delivery failure
- **WHEN** push notification delivery fails
- **THEN** the system may retry delivery
- **AND** logs failure for monitoring
- **AND** maintains message in database for sync

#### Scenario: Batch notification delivery
- **WHEN** multiple messages arrive for offline user
- **THEN** the system may batch notifications
- **AND** sends single notification with count
- **AND** reduces notification spam

### Requirement: Push Notification Preferences
The system SHALL support user preferences for push notifications.

#### Scenario: Disable push notifications
- **WHEN** a user disables push notifications
- **THEN** the system does not send notifications
- **AND** respects user preference
- **AND** messages still stored for sync

#### Scenario: Quiet hours
- **WHEN** quiet hours are configured
- **THEN** the system does not send notifications during quiet hours
- **AND** messages are stored for later
- **AND** notifications resume after quiet hours

#### Scenario: Per-conversation preferences
- **WHEN** user mutes specific conversation
- **THEN** the system does not send notifications for that conversation
- **AND** messages are still stored
- **AND** user can check messages manually

### Requirement: Push Notification Integration
The system SHALL integrate with FCM and APNS services.

#### Scenario: FCM integration
- **WHEN** sending notification to Android device
- **THEN** the system uses FCM API
- **AND** includes FCM server key
- **AND** formats notification for FCM

#### Scenario: APNS integration
- **WHEN** sending notification to iOS device
- **THEN** the system uses APNS API
- **AND** includes APNS certificate or key
- **AND** formats notification for APNS

#### Scenario: Handle service errors
- **WHEN** FCM or APNS service returns error
- **THEN** the system handles error gracefully
- **AND** logs error for monitoring
- **AND** may retry or mark token invalid
