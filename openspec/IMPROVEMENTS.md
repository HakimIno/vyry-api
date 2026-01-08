# OpenSpec Improvements for Vyry-API

## สรุปสถานะปัจจุบัน

### ✅ มีอยู่แล้ว
- **AGENTS.md** - คู่มือ OpenSpec ครบถ้วนดี
- **project.md** - Project context พื้นฐานดี แต่ยังขาดรายละเอียดบางส่วน

### ❌ ขาดหายไป

#### 1. Specs สำหรับ Capabilities ที่มีอยู่
โปรเจกต์มี features หลายอย่างแต่ยังไม่มี specs:

- **Authentication** (`specs/auth/spec.md`)
  - OTP request/verification
  - Profile setup
  - PIN setup/verification
  - Device linking (create, complete, approve)
  - Device management (list, unlink)
  - Token refresh

- **Chat/Messaging** (`specs/chat/spec.md`)
  - Send messages
  - Sync messages
  - Update delivery status (delivered/read)
  - WebSocket connection management

- **Keys** (`specs/keys/spec.md`)
  - Prekey bundle retrieval
  - Signal protocol key management

- **Health** (`specs/health/spec.md`)
  - Health check endpoint

#### 2. project.md ยังขาดรายละเอียด

**API Conventions:**
- API versioning strategy (`/api/v1/`)
- Error response format
- Request/response patterns
- Status code conventions

**Security:**
- Rate limiting strategy
- JWT token expiration/refresh
- PIN security requirements
- OTP expiration and retry limits
- Device linking security

**Error Handling:**
- Standard error response format
- Error codes convention
- Retry logic

**Deployment:**
- Environment variables
- Database migration process
- Health check requirements
- Monitoring/logging setup

**Performance:**
- Redis caching strategy
- Database query optimization
- WebSocket connection limits

## แนะนำการปรับปรุง

### Phase 1: เพิ่ม Specs สำหรับ Capabilities หลัก (Priority: High)

สร้าง specs สำหรับ capabilities ที่มีอยู่แล้ว:

1. `specs/auth/spec.md` - Authentication requirements
2. `specs/chat/spec.md` - Messaging requirements  
3. `specs/keys/spec.md` - Key management requirements
4. `specs/health/spec.md` - Health check requirements

### Phase 2: ปรับปรุง project.md (Priority: Medium)

เพิ่ม sections:
- API Conventions
- Security Best Practices
- Error Handling Standards
- Deployment Procedures
- Performance Guidelines

### Phase 3: สร้าง Example Specs (Priority: Low)

สร้างตัวอย่าง specs ใน `specs/` เพื่อเป็น template สำหรับ future changes

## ตัวอย่าง Structure ที่ควรมี

```
openspec/
├── AGENTS.md              ✅ มีแล้ว
├── project.md             ⚠️ ต้องเพิ่มรายละเอียด
├── specs/                 ❌ ว่างเปล่า - ต้องสร้าง
│   ├── auth/
│   │   └── spec.md
│   ├── chat/
│   │   └── spec.md
│   ├── keys/
│   │   └── spec.md
│   └── health/
│       └── spec.md
└── changes/
    └── archive/
```

## Next Steps

1. **สร้าง specs สำหรับ capabilities หลัก** - เริ่มจาก auth, chat, keys, health
2. **อัปเดต project.md** - เพิ่ม API conventions, security, error handling
3. **Review และ validate** - ใช้ `openspec validate --strict` เพื่อตรวจสอบ

