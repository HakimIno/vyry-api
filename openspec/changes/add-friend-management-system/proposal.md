# Change: Add Friend Management System

## Why

Currently, the system lacks any social graph capabilities. Users cannot add contacts, search for friends by ID/phone, or maintain a friend list. This is a core requirement for a chat application to function beyond ephemeral messaging.

## What Changes

- **New Tables**: `friends` (stores relationships and status).
- **New API Endpoints**:
  - `POST /api/v1/friends/request`: Send friend request.
  - `POST /api/v1/friends/accept`: Accept friend request.
  - `GET /api/v1/friends`: List all friends.
  - `GET /api/v1/users/search`: Search by Username or Phone Number.
  - `PUT /api/v1/users/username`: Set/Update global Vyry ID.
- **Privacy Privacy**: Friend requests require approval (Linear logic, not just auto-add).

## Impact

- **Database**: New migrations for `friends` table.
- **API**: New `User` and `Friend` controllers.
- **Specs**: New `friend-management` spec.
