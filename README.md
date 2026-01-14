# Vyry-API - Production Signal+Line Chat System

โปรเจกต์แชทแบบ Signal + Line เต็มระบบ สร้างด้วย Rust 2025 standard

## Tech Stack

- **actix-web 4.12** - Web framework พร้อม WebSocket support
- **tokio 1.48** - Async runtime
- **sea-orm 1.1** - ORM พร้อม migrations, PostgreSQL support
- **ed25519-dalek 2.1 + x25519-dalek 2.0** - Custom Signal Protocol implementation
- **redis 0.27** - Pub/Sub และ presence tracking
- **jsonwebtoken 9.3** - JWT authentication
- **PostgreSQL 16** - Main database
- **Redis 7** - Cache และ real-time messaging

## โครงสร้างโปรเจกต์

```
vyry-api/
├── Cargo.toml                  # Workspace root
├── apps/api/                   # Main API service
├── crates/
│   ├── core/                   # Entities + Signal wrapper
│   ├── domain/                 # Business logic
│   ├── application/            # Use cases
│   ├── infrastructure/         # Database + Redis
│   └── interfaces/             # DTOs
├── migrations/                 # 9 SeaORM migrations
├── docker-compose.yml          # PostgreSQL + Redis + Adminer
└── .env.example

```

## Database Schema

9 ตารางตามสเปค 100%:

1. `users` - ข้อมูลผู้ใช้พร้อม phone_number_hash
2. `devices` - อุปกรณ์แต่ละเครื่องพร้อม Signal keys
3. `one_time_prekeys` - PreKeys 100 อันต่อ device
4. `signal_sessions` - Session records
5. `conversations` - ห้องแชท (personal/group)
6. `conv_members` - สมาชิกในห้อง
7. `messages` - ข้อความเข้ารหัส AES-GCM
8. `message_deliveries` - สถานะการส่ง/อ่าน
9. `push_tokens` - FCM/APNS tokens

## Setup

### Option 1: Docker/Podman (แนะนำสำหรับ Production)

#### 1. สร้าง .env file

```bash
cp .env.example .env
# แก้ไข JWT_SECRET ให้เป็นค่าที่ปลอดภัย (อย่างน้อย 32 ตัวอักษร)
```

#### 2. Build และ Run ทั้งหมด (PostgreSQL + Redis + API)

**Production Build (ใช้เวลานาน แต่ optimized):**
```bash
podman compose up -d --build
# หรือ
docker compose up -d --build
```

**Fast Build (สำหรับเครื่อง RAM น้อย หรือ development):**
```bash
DOCKERFILE=Dockerfile.fast podman compose -f docker-compose.yml -f docker-compose.fast.yml up --build
```

API จะรันที่ `http://localhost:8000`

#### 3. Run Migrations

```bash
podman compose exec api sea-orm-cli migrate up
```

#### 4. ดู Logs

```bash
podman compose logs -f api
```

#### 5. Development Mode (Hot Reload)

```bash
podman compose -f docker-compose.yml -f docker-compose.dev.yml up
```

#### ⚠️ Tips สำหรับ Build ที่ช้า:

- **ใช้ Fast Build**: `Dockerfile.fast` ใช้ debug build และ single job (ใช้ RAM น้อยกว่า)
- **เพิ่ม RAM**: Rust release build ต้องการ RAM อย่างน้อย 4GB
- **ใช้ BuildKit cache**: `BUILDKIT_INLINE_CACHE=1` ช่วย cache dependencies
- **Build แยก**: Build dependencies ก่อน แล้วค่อย build application

### Option 2: Local Development

#### 1. เริ่มต้น Database

```bash
docker-compose up -d postgres redis adminer
```

### 2. สร้าง .env

```bash
cp .env.example .env
```

### 3. Run Migrations

```bash
cargo install sea-orm-cli
sea-orm-cli migrate up
```

### 4. Build & Run

**⚠️ IMPORTANT: Zerocopy Workaround**

ปัจจุบันมีปัญหา zerocopy 0.8.31 ต้องการ Rust nightly features ให้แก้ไขดังนี้:

```bash
# Option 1: ใช้ Rust nightly
rustup default nightly
cargo build --workspace

# Option 2: Pin zerocopy version (แนะนำ)
# เพิ่มใน Cargo.toml ที่ workspace root:
# [patch.crates-io]
# zerocopy = { version = "=0.7.35" }

cargo build --workspace
cargo run -p api
```

Server จะรันที่ `http://0.0.0.0:8000`

## API Endpoints

### Health Check

```bash
curl http://localhost:8000/health
```

### Register

```bash
curl -X POST http://localhost:8000/api/v1/register \
  -H "Content-Type: application/json" \
  -d '{
    "phone_number": "+66812345678",
    "device_name": "iPhone 15 Pro",
    "platform": 1
  }'
```

### Login

```bash
curl -X POST http://localhost:8000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "phone_number": "+66812345678",
    "device_uuid": "uuid-from-register-response"
  }'
```

### WebSocket

```
ws://localhost:8000/ws/
```

## Signal Protocol Implementation

Custom implementation ใช้:

- **ed25519-dalek** สำหรับ Identity Keys และ Signatures
- **x25519-dalek** สำหรับ PreKeys และ Key Exchange
- **AES-GCM** สำหรับ Message Encryption
- **HKDF** สำหรับ Key Derivation

### Key Generation

```rust
use core::signal::wrapper::create_signal_keys;

let keys = create_signal_keys()?;
// keys.identity_key_pair
// keys.registration_id
// keys.signed_prekey
// keys.one_time_prekeys (100 keys)
```

## Features

✅ User registration พร้อม Signal key generation  
✅ Device management (multi-device support)  
✅ 100 one-time prekeys ต่อ device  
✅ WebSocket connection manager  
✅ Redis Pub/Sub สำหรับ real-time messaging  
✅ JWT authentication  
✅ CORS enabled  
✅ Health check endpoint  
✅ Docker Compose setup  
✅ SeaORM migrations

## Database Tools

### Adminer

เข้าถึงที่ `http://localhost:8080`

- System: PostgreSQL
- Server: postgres
- Username: vyryuser
- Password: vyrypass
- Database: vyrydb

### Redis CLI

```bash
docker exec -it vyry-redis redis-cli
```

## Production Deployment

1. Update `.env` with production credentials
2. Enable SSL/TLS for PostgreSQL
3. Configure Redis password
4. Set strong JWT_SECRET
5. Enable rate limiting
6. Add monitoring (Prometheus/Grafana)
7. Setup backup strategy

## License

MIT
# vyry-api
