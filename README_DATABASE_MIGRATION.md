# Database Migration & Scaling Guide

## Quick Start

### ย้าย Database ได้ง่าย

ระบบรองรับการย้าย database ผ่าน environment variables:

```bash
# PostgreSQL
export POSTGRES_URL=postgresql://new-host:5432/new-db

# Redis  
export REDIS_URL=redis://new-host:6379

# Restart application
cargo run -p api
```

## Architecture

```
┌─────────────────────────────────────┐
│      Application Layer              │
│   (Use Cases / Business Logic)      │
└─────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────┐
│    Database Connections              │
│  - PostgreSQL (users/metadata)      │
│  - Redis (cache/pubsub)             │
│  - ScyllaDB (messages) [Future]     │
└─────────────────────────────────────┘
```

## Database Responsibilities

### PostgreSQL
- Users, Devices, Conversations
- Metadata และ relational data
- **Migration**: ใช้ `sea-orm-cli migrate up`

### Redis
- OTP storage (temporary)
- Rate limiting
- Pub/sub สำหรับ real-time messaging
- **Migration**: ไม่จำเป็น (temporary data)

### ScyllaDB (Future)
- Messages (high-volume)
- Message deliveries
- **Migration**: จะ implement ในอนาคต

## Configuration

### Environment Variables

```bash
# .env file
POSTGRES_URL=postgresql://user:pass@host:5432/db
REDIS_URL=redis://host:6379
# SCYLLADB_URL=scylladb://host:9042  # Future
```

### Multiple Environments

**Development**:
```bash
POSTGRES_URL=postgresql://localhost:5432/vyrydb_dev
REDIS_URL=redis://localhost:6379
```

**Production**:
```bash
POSTGRES_URL=postgresql://prod-db.example.com:5432/vyrydb
REDIS_URL=redis://prod-redis.example.com:6379
```

## Migration Steps

### 1. ย้าย PostgreSQL

```bash
# 1. Backup old database
pg_dump -h old-host -U user -d database > backup.sql

# 2. Update environment variable
export POSTGRES_URL=postgresql://new-host:5432/new-db

# 3. Run migrations
sea-orm-cli migrate up

# 4. Import data (if needed)
psql $POSTGRES_URL < backup.sql

# 5. Restart application
cargo run -p api
```

### 2. ย้าย Redis

```bash
# 1. Update environment variable
export REDIS_URL=redis://new-host:6379

# 2. Restart application
# Note: Redis data is temporary, no migration needed
```

## Benefits

✅ **Easy Migration**: เปลี่ยน database URL ได้ทันที  
✅ **Scalability**: Scale แต่ละ database แยกกัน  
✅ **Flexibility**: ใช้ database ที่เหมาะสมกับ workload  
✅ **Backward Compatible**: ยังรองรับ `DATABASE_URL` (legacy)

## Next Steps

1. ✅ PostgreSQL abstraction - Done
2. ✅ Redis abstraction - Done  
3. ⏳ ScyllaDB integration - Future work
4. ⏳ Repository pattern migration - Gradually migrate use cases
