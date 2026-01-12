# Typing Indicators Specification

## Requirements

### Requirement: Send Typing Indicator
The system SHALL allow users to send typing indicators to indicate active typing.

#### Scenario: Send typing started indicator
- **WHEN** a user starts typing in a conversation
- **THEN** the device sends Typing message with is_typing = true
- **AND** the system forwards indicator to recipient(s)
- **AND** recipient sees typing indicator

#### Scenario: Send typing stopped indicator
- **WHEN** a user stops typing
- **THEN** the device sends Typing message with is_typing = false
- **AND** the system forwards indicator to recipient(s)
- **AND** recipient sees typing stopped

#### Scenario: Typing indicator timeout
- **WHEN** a user stops typing without explicit stop message
- **THEN** the system automatically sends stop indicator after timeout
- **AND** prevents stale typing indicators
- **AND** timeout is configurable (default 3 seconds)

### Requirement: Typing Indicator Forwarding
The system SHALL forward typing indicators to conversation participants.

#### Scenario: Forward to one-on-one recipient
- **WHEN** a user types in one-on-one conversation
- **THEN** the system forwards indicator to the other participant
- **AND** recipient receives real-time typing status
- **AND** indicator includes conversation_id

#### Scenario: Forward to group members
- **WHEN** a user types in group conversation
- **THEN** the system forwards indicator to all other group members
- **AND** members see who is typing
- **AND** indicator includes user identification

#### Scenario: Forward to offline recipients
- **WHEN** typing indicator is sent but recipient is offline
- **THEN** the system does not store indicator
- **AND** typing indicators are real-time only
- **AND** offline users do not receive indicators

### Requirement: Typing Indicator Display
The system SHALL support displaying typing indicators to users.

#### Scenario: Display typing indicator
- **WHEN** a recipient receives typing indicator
- **THEN** the client displays typing animation
- **AND** shows which user is typing
- **AND** updates UI in real-time

#### Scenario: Multiple users typing in group
- **WHEN** multiple users type in group conversation
- **THEN** the system forwards all indicators
- **AND** client displays all typing users
- **AND** shows "X, Y, and Z are typing"

#### Scenario: Typing indicator removal
- **WHEN** typing stops or timeout occurs
- **THEN** the system sends stop indicator
- **AND** client removes typing animation
- **AND** UI updates immediately

### Requirement: Typing Indicator Rate Limiting
The system SHALL limit typing indicator frequency to prevent spam.

#### Scenario: Rate limit typing indicators
- **WHEN** a user sends typing indicators too frequently
- **THEN** the system applies rate limiting
- **AND** prevents indicator spam
- **AND** maintains reasonable update frequency

#### Scenario: Typing indicator throttling
- **WHEN** typing indicators exceed rate limit
- **THEN** the system throttles indicators
- **AND** sends indicators at reduced frequency
- **AND** maintains user experience

### Requirement: Typing Indicator Privacy
The system SHALL respect user privacy preferences for typing indicators.

#### Scenario: Disable typing indicators
- **WHEN** a user disables typing indicators
- **THEN** the system does not send indicators
- **AND** other users do not see typing status
- **AND** privacy is maintained

#### Scenario: Selective typing indicators
- **WHEN** a user enables typing for specific conversations
- **THEN** the system sends indicators only for those conversations
- **AND** respects per-conversation settings
