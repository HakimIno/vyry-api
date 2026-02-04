## ADDED Requirements

### Requirement: User Identification

Users SHALL be able to identify themselves via a unique username (Vyry ID) and phone number.

#### Scenario: Set Username

- **WHEN** a user sets their username
- **THEN** the system verifies uniqueness
- **AND** updates the user profile
- **AND** allows this username to be used for search

#### Scenario: Search by Username

- **WHEN** a user searches for a username string
- **THEN** the system returns the public profile of the matching user
- **AND** returns 404 if not found

#### Scenario: Search by Phone Number

- **WHEN** a user provides a phone number for search
- **THEN** client hashes the phone number
- **AND** system looks up user by phone_number_hash
- **AND** returns public profile if found

### Requirement: Friend Relationships

The system SHALL allow users to manage friend relationships.

#### Scenario: Send Friend Request

- **WHEN** User A sends a friend request to User B
- **THEN** the system creates a relationship record with status PENDING
- **AND** notifies User B (future scope: notification)

#### Scenario: Accept Friend Request

- **WHEN** User B accepts a request from User A
- **THEN** the system updates relationship status to ACCEPTED
- **AND** both users appear in each other's friend lists

#### Scenario: List Friends

- **WHEN** a user requests their friend list
- **THEN** the system returns all users with ACCEPTED status
- **AND** includes their profile summaries

#### Scenario: Block User

- **WHEN** a user blocks another user
- **THEN** relationship status updates to BLOCKED
- **AND** blocked user cannot send messages or requests
