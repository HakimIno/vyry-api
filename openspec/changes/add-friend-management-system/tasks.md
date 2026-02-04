## 1. Database Schema

- [ ] 1.1 Create `friends` table migration (user_id, friend_id, status, created_at, updated_at).
- [ ] 1.2 Add indexes for fast lookup (by user_id, by status).

## 2. Core Logic (Domain/Application)

- [ ] 2.1 Implement `AddFriendUseCase` (Send Request).
- [ ] 2.2 Implement `AcceptFriendUseCase` (Update Status).
- [ ] 2.3 Implement `ListFriendsUseCase` (Query with Pagination).
- [ ] 2.4 Implement `SearchUserUseCase` (By Username or Phone Hash).
- [ ] 2.5 Implement `SetUsernameUseCase` (Unique constraint check).

## 3. API Handlers

- [ ] 3.1 `POST /friends/request`
- [ ] 3.2 `POST /friends/accept`
- [ ] 3.3 `GET /friends`
- [ ] 3.4 `GET /users/search`
- [ ] 3.5 `PUT /users/username`

## 4. Testing

- [ ] 4.1 Unit tests for friend logic (state transitions).
- [ ] 4.2 Integration tests for API endpoints.
