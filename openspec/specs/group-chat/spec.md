# Group Chat Specification

## Requirements

### Requirement: Create Group Conversation
The system SHALL allow users to create group conversations.

#### Scenario: Create new group
- **WHEN** an authenticated user creates a group conversation
- **THEN** the system creates conversation with conv_type = 2 (group)
- **AND** sets creator_id to the user
- **AND** adds creator as conv_member with role = 1 (admin)
- **AND** allows adding initial members
- **AND** returns conversation_id

#### Scenario: Create group with name
- **WHEN** a user creates group with name
- **THEN** the system stores the group name
- **AND** name is visible to all members

#### Scenario: Create group with avatar
- **WHEN** a user creates group with avatar
- **THEN** the system stores avatar URL
- **AND** avatar is visible to all members

#### Scenario: Create group without authentication
- **WHEN** an unauthenticated user attempts to create group
- **THEN** the system returns 401 Unauthorized
- **AND** includes error code UNAUTHORIZED

### Requirement: Add Group Members
The system SHALL allow group admins to add members.

#### Scenario: Admin adds member
- **WHEN** a group admin adds a user to group
- **THEN** the system verifies admin permissions
- **AND** checks if user is already a member
- **AND** if not, adds user as conv_member
- **AND** sets role = 0 (member) by default
- **AND** sets joined_at timestamp
- **AND** notifies new member

#### Scenario: Add multiple members
- **WHEN** an admin adds multiple users at once
- **THEN** the system adds all users in single transaction
- **AND** creates conv_member record for each user
- **AND** notifies all new members

#### Scenario: Add existing member
- **WHEN** an admin attempts to add user who is already a member
- **THEN** the system returns 400 Bad Request
- **AND** includes error code INVALID_REQUEST

#### Scenario: Non-admin adds member
- **WHEN** a non-admin user attempts to add member
- **THEN** the system returns 403 Forbidden
- **AND** includes error code FORBIDDEN

### Requirement: Remove Group Members
The system SHALL allow group admins to remove members.

#### Scenario: Admin removes member
- **WHEN** a group admin removes a member
- **THEN** the system verifies admin permissions
- **AND** removes user from conv_members
- **AND** sets left_at timestamp
- **AND** user no longer receives group messages
- **AND** notifies remaining members

#### Scenario: Remove self
- **WHEN** a member removes themselves from group
- **THEN** the system removes user from conv_members
- **AND** sets left_at timestamp
- **AND** user no longer receives group messages

#### Scenario: Remove non-member
- **WHEN** an admin attempts to remove user who is not a member
- **THEN** the system returns 400 Bad Request
- **AND** includes error code INVALID_REQUEST

#### Scenario: Remove last admin
- **WHEN** an admin attempts to remove the last admin
- **THEN** the system prevents removal
- **AND** returns 400 Bad Request
- **AND** includes error code INVALID_REQUEST

### Requirement: Update Member Roles
The system SHALL allow group admins to update member roles.

#### Scenario: Promote member to admin
- **WHEN** a group admin promotes a member to admin
- **THEN** the system updates member role to 1 (admin)
- **AND** member gains admin permissions
- **AND** can add/remove members

#### Scenario: Demote admin to member
- **WHEN** a group admin demotes another admin
- **THEN** the system updates admin role to 0 (member)
- **AND** admin loses admin permissions
- **AND** cannot add/remove members

#### Scenario: Update role without permission
- **WHEN** a non-admin attempts to update roles
- **THEN** the system returns 403 Forbidden
- **AND** includes error code FORBIDDEN

### Requirement: Broadcast Messages to Group
The system SHALL support sending messages to all group members.

#### Scenario: Send message to group
- **WHEN** a group member sends a message
- **THEN** the system creates message record
- **AND** creates message_deliveries for all active members
- **AND** forwards message to all online members via WebSocket
- **AND** stores message for offline members

#### Scenario: Send to large group
- **WHEN** a message is sent to group with many members
- **THEN** the system efficiently creates delivery records
- **AND** uses batch operations for performance
- **AND** handles offline members gracefully

#### Scenario: Send after leaving group
- **WHEN** a user sends message after leaving group
- **THEN** the system returns 403 Forbidden
- **AND** includes error code FORBIDDEN

### Requirement: Group Metadata Management
The system SHALL allow admins to update group metadata.

#### Scenario: Update group name
- **WHEN** a group admin updates group name
- **THEN** the system updates the name
- **AND** all members see updated name
- **AND** change is logged in metadata

#### Scenario: Update group avatar
- **WHEN** a group admin updates group avatar
- **THEN** the system updates avatar URL
- **AND** all members see updated avatar

#### Scenario: Update group metadata
- **WHEN** a group admin updates metadata JSON
- **THEN** the system stores custom metadata
- **AND** metadata is available to all members

#### Scenario: Non-admin updates metadata
- **WHEN** a non-admin attempts to update group metadata
- **THEN** the system returns 403 Forbidden
- **AND** includes error code FORBIDDEN

### Requirement: Group Member List
The system SHALL allow members to view group member list.

#### Scenario: Get member list
- **WHEN** a group member requests member list
- **THEN** the system returns all active members
- **AND** includes member roles
- **AND** includes joined_at timestamps
- **AND** excludes left members

#### Scenario: Get member list for non-member
- **WHEN** a non-member requests member list
- **THEN** the system returns 403 Forbidden
- **AND** includes error code FORBIDDEN
