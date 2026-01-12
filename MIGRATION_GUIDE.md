# Database Migration Guide

## Overview

คู่มือสำหรับย้าย database เมื่อต้องการ scale หรือเปลี่ยน database provider

## Architecture

```
PostgreSQL (Users/Metadata) + ScyllaDB (Messages) + Redis (Cache/PubSub)
```

## การย้าย Database

### 1. ย้าย PostgreSQL

#### เปลี่ยน PostgreSQL Provider

**Step 1**: อัปเดต environment variable
```bash
# Old
POSTGRES_URL=postgresql://user:pass@localhost:5432/db

# New (e.g., AWS RDS)
POSTGRES_URL=postgresql://user:pass@rds-instance.region.rds.amazonaws.com:5432/db
```

**Step 2**: Run migrations
```bash
sea-orm-cli migrate up
```

**Step 3**: Restart application
```bash
cargo run -p api
```

#### Migrate ข้อมูลจาก Database เก่า

```bash
# Export data
pg_dump -h old-host -U user -d database > backup.sql

# Import data
psql -h new-host -U user -d database < backup.sql
```

### 2. ย้ายไป ScyllaDB (สำหรับ Messages)

**Status**: ยังไม่ implement (future work)

เมื่อ implement แล้ว:
1. Messages จะย้ายไป ScyllaDB
2. Message_deliveries จะย้ายไป ScyllaDB
3. PostgreSQL จะเก็บแค่ metadata

### 3. ย้าย Redis

#### เปลี่ยน Redis Provider

**Step 1**: อัปเดต environment variable
```bash
# Old
REDIS_URL=redis://localhost:6379

# New (e.g., AWS ElastiCache)
REDIS_URL=redis://elasticache-cluster.region.cache.amazonaws.com:6379
```

**Step 2**: Restart application

**Note**: Redis data ไม่จำเป็นต้อง migrate เพราะเป็น temporary data (OTP, cache)

## Configuration

### Environment Variables

```bash
# PostgreSQL (required)
POSTGRES_URL=postgresql://user:pass@host:5432/db

# Redis (required)
REDIS_URL=redis://host:6379

# ScyllaDB (optional, for future)
SCYLLADB_URL=scylladb://host:9042
```

### Multiple Environments

**Development**:
```bash
POSTGRES_URL=postgresql://localhost:5432/vyrydb_dev
REDIS_URL=redis://localhost:6379
```

**Staging**:
```bash
POSTGRES_URL=postgresql://staging-db.example.com:5432/vyrydb_staging
REDIS_URL=redis://staging-redis.example.com:6379
```

**Production**:
```bash
POSTGRES_URL=postgresql://prod-db.example.com:5432/vyrydb_prod
REDIS_URL=redis://prod-redis.example.com:6379
```

## Database Responsibilities

### PostgreSQL
- Users
- Devices
- Conversations
- Conv_members
- One-time prekeys
- Signal sessions
- Device linking sessions
- Push tokens

### ScyllaDB (Future)
- Messages
- Message_deliveries

### Redis
- OTP storage (temporary)
- Rate limiting
- Pub/sub
- Cache

## Best Practices

1. **Connection Pooling**: แต่ละ database มี connection pool แยก
2. **Environment Variables**: ใช้ environment variables สำหรับ configuration
3. **Migration Strategy**: แยก migrations ตาม database type
4. **Backup Strategy**: Backup แต่ละ database แยกกัน
5. **Monitoring**: Monitor แต่ละ database แยกกัน

## Troubleshooting

### Connection Issues

```bash
# Test PostgreSQL connection
psql $POSTGRES_URL

# Test Redis connection
redis-cli -u $REDIS_URL ping
```

### Migration Issues

```bash
# Check migration status
sea-orm-cli migrate status

# Rollback if needed
sea-orm-cli migrate down
```
