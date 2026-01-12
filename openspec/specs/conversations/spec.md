# Conversations Specification

## Requirements

### Requirement: Create One-on-One Conversation
The system SHALL allow users to create one-on-one conversations.

#### Scenario: Create new conversation
- **WHEN** a user requests to create conversation with another user
- **THEN** the system checks if conversation already exists
- **AND** if not exists, creates new conversation with conv_type = 1 (one-on-one)
- **AND** adds both users as conv_members
- **AND** returns conversation_id
- **AND** sets created_at timestamp

#### Scenario: Return existing conversation
- **WHEN** a user requests to create conversation
- **AND** conversation already exists between the two users
- **THEN** the system returns existing conversation_id
- **AND** does not create duplicate conversation

#### Scenario: Create conversation with invalid user
- **WHEN** a user attempts to create conversation with non-existent user
- **THEN** the system returns 404 Not Found
- **AND** includes error code INVALID_REQUEST

#### Scenario: Create conversation without authentication
- **WHEN** an unauthenticated user attempts to create conversation
- **THEN** the system returns 401 Unauthorized
- **AND** includes error code UNAUTHORIZED

### Requirement: List Conversations
The system SHALL allow users to retrieve their conversation list.

#### Scenario: Get user conversations
- **WHEN** an authenticated user requests conversation list
- **THEN** the system queries all conversations where user is a member
- **AND** returns conversations sorted by last message timestamp
- **AND** includes conversation metadata (name, avatar, type)
- **AND** includes unread message count per conversation

#### Scenario: Paginated conversation list
- **WHEN** a user has many conversations
- **THEN** the system supports pagination
- **AND** returns limited results per page
- **AND** includes pagination metadata (has_more, next_cursor)

#### Scenario: Empty conversation list
- **WHEN** a new user has no conversations
- **THEN** the system returns empty array
- **AND** indicates successful query

### Requirement: Get Conversation Details
The system SHALL allow users to retrieve conversation details.

#### Scenario: Get conversation information
- **WHEN** an authenticated user requests conversation details
- **AND** user is a member of the conversation
- **THEN** the system returns conversation metadata
- **AND** includes conversation type, name, avatar
- **AND** includes member list
- **AND** includes creation timestamp

#### Scenario: Get non-member conversation
- **WHEN** a user attempts to access conversation they're not a member of
- **THEN** the system returns 403 Forbidden
- **AND** includes error code FORBIDDEN

#### Scenario: Get non-existent conversation
- **WHEN** a user requests non-existent conversation
- **THEN** the system returns 404 Not Found
- **AND** includes error code INVALID_REQUEST

### Requirement: Get Message History
The system SHALL allow users to retrieve message history for a conversation.

#### Scenario: Get recent messages
- **WHEN** an authenticated user requests message history
- **THEN** the system returns messages from the conversation
- **AND** messages are sorted by sent_at descending (newest first)
- **AND** includes message metadata (sender, timestamp, type)
- **AND** includes encrypted content for user's device

#### Scenario: Paginated message history
- **WHEN** a conversation has many messages
- **THEN** the system supports pagination
- **AND** returns limited messages per request
- **AND** supports cursor-based pagination with message_id

#### Scenario: Get messages before timestamp
- **WHEN** a user requests messages before a specific timestamp
- **THEN** the system returns messages older than the timestamp
- **AND** enables infinite scroll functionality

#### Scenario: Get empty message history
- **WHEN** a new conversation has no messages
- **THEN** the system returns empty array
- **AND** indicates successful query

### Requirement: Update Conversation Metadata
The system SHALL allow users to update conversation metadata.

#### Scenario: Update conversation name
- **WHEN** an authenticated user updates conversation name
- **AND** user has permission (creator or admin)
- **THEN** the system updates the name
- **AND** returns updated conversation data

#### Scenario: Update conversation avatar
- **WHEN** an authenticated user updates conversation avatar
- **AND** user has permission
- **THEN** the system updates avatar URL
- **AND** returns updated conversation data

#### Scenario: Update without permission
- **WHEN** a user attempts to update conversation without permission
- **THEN** the system returns 403 Forbidden
- **AND** includes error code FORBIDDEN

### Requirement: Leave Conversation
The system SHALL allow users to leave conversations.

#### Scenario: Leave one-on-one conversation
- **WHEN** a user leaves a one-on-one conversation
- **THEN** the system marks user as left (sets left_at timestamp)
- **AND** user no longer receives new messages
- **AND** conversation remains for other user

#### Scenario: Leave group conversation
- **WHEN** a user leaves a group conversation
- **THEN** the system removes user from conv_members
- **AND** user no longer receives group messages
- **AND** group continues with remaining members

#### Scenario: Leave as last member
- **WHEN** the last member leaves a conversation
- **THEN** the system may archive the conversation
- **AND** conversation becomes inactive
