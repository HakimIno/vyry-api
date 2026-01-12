# Multi-Database Architecture Design

## Overview

Architecture สำหรับรองรับ multiple databases:
- **PostgreSQL**: Users, devices, conversations, metadata (relational data)
- **ScyllaDB**: Messages, message_deliveries (time-series, high-volume data)
- **Redis**: Cache, pub/sub, rate limiting

## Design Principles

1. **Database Abstraction**: Use trait-based abstraction for easy database switching
2. **Repository Pattern**: Separate data access from business logic
3. **Connection Pooling**: Independent connection pools per database
4. **Configuration-Driven**: Easy to change database URLs via environment variables
5. **Migration Strategy**: Separate migrations per database type

## Architecture Layers

```
┌─────────────────────────────────────────┐
│         Application Layer               │
│      (Use Cases / Business Logic)       │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│         Repository Layer                │
│    (Trait-based abstractions)           │
│  - UserRepository                       │
│  - MessageRepository                    │
│  - ConversationRepository               │
└─────────────────────────────────────────┘
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
┌───────────┐ ┌──────────┐ ┌──────────┐
│PostgreSQL │ │ ScyllaDB │ │  Redis   │
│Repository │ │Repository│ │Repository│
└───────────┘ └──────────┘ └──────────┘
        │           │           │
        ▼           ▼           ▼
┌───────────┐ ┌──────────┐ ┌──────────┐
│PostgreSQL │ │ ScyllaDB │ │  Redis   │
│Connection │ │Connection│ │Connection│
└───────────┘ └──────────┘ └──────────┘
```

## Database Responsibilities

### PostgreSQL
- Users (authentication, profiles)
- Devices (device management)
- Conversations (metadata)
- Conv_members (membership)
- One-time prekeys
- Signal sessions
- Device linking sessions
- Push tokens

### ScyllaDB
- Messages (high-volume, time-series)
- Message_deliveries (delivery status per device)

### Redis
- OTP storage (temporary)
- Rate limiting counters
- Pub/sub for real-time messaging
- Session cache
- Presence tracking

## Implementation Strategy

### Phase 1: Abstraction Layer
1. Create trait-based repository interfaces
2. Implement PostgreSQL repositories
3. Update use cases to use repositories

### Phase 2: ScyllaDB Integration
1. Add ScyllaDB driver
2. Implement ScyllaDB message repository
3. Migrate message operations to ScyllaDB

### Phase 3: Configuration & Migration
1. Update config for multiple databases
2. Separate migration strategies
3. Data migration tools

## Benefits

1. **Scalability**: Scale each database independently
2. **Performance**: Right database for right workload
3. **Flexibility**: Easy to switch databases
4. **Maintainability**: Clear separation of concerns
