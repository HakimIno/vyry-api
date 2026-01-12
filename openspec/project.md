# Project Context

## Purpose
Vyry-API is a production-ready chat system implementing Signal + Line messaging functionality. It provides end-to-end encrypted messaging with multi-device support, built entirely in Rust following 2025 standards. The system includes user authentication, real-time messaging, and comprehensive security features.

## Tech Stack
- **Rust 2021** - Systems programming language with memory safety
- **Actix-web 4.9** - Web framework with WebSocket support
- **Tokio 1.40** - Asynchronous runtime
- **Sea-ORM 1.1** - Object-relational mapping with PostgreSQL support
- **ed25519-dalek 2.1 + x25519-dalek 2.0** - Custom Signal Protocol implementation
- **Redis 0.27** - Pub/sub and presence tracking
- **JSON Web Token 9.3** - Authentication tokens
- **PostgreSQL 16** - Primary database
- **Redis 7** - Caching and real-time messaging
- **Docker Compose** - Container orchestration for development

## Project Conventions

### Code Style
- Rust 2021 edition standards
- Async/await patterns throughout
- Domain-driven design with clear separation of concerns
- Workspace structure with multiple crates
- Comprehensive error handling with anyhow
- Comprehensive logging with tracing

### Architecture Patterns
- **Hexagonal Architecture**: Clear separation between domain, application, and infrastructure
- **CQRS-style**: Separate read/write models in application layer
- **Workspace Structure**: Multiple crates for different concerns
  - `core/`: Entities and Signal protocol wrapper
  - `domain/`: Business logic
  - `application/`: Use cases and application logic
  - `infrastructure/`: Database and Redis implementation
  - `interfaces/`: Data transfer objects
  - `apps/api/`: Web API and WebSocket handlers
- **Clean Architecture**: Dependencies point inward toward domain
- **Custom Signal Protocol**: Implementation using ed25519/x25519 for E2E encryption

### Testing Strategy
- Unit tests for individual functions
- Integration tests for API endpoints
- Custom test cases for cryptographic operations
- No external testing framework dependencies

### Git Workflow
- No specific branching strategy documented yet
- Workspace structure suggests feature branches

## API Conventions

### Versioning
- All API endpoints use `/api/v1/` prefix
- Version is included in URL path, not headers
- Breaking changes require new version (e.g., `/api/v2/`)
- Non-breaking changes can be added to existing version

### Request/Response Format
- **Content-Type**: `application/json` for all requests and responses
- **Request Body**: JSON format with camelCase field names
- **Response Body**: JSON format with camelCase field names
- **Status Codes**: Standard HTTP status codes (200, 201, 400, 401, 403, 404, 429, 500)

### Error Response Format
All error responses follow this structure:
```json
{
  "error": "Human-readable error message",
  "error_code": "MACHINE_READABLE_CODE",
  "retry_after_seconds": 600  // Optional, for rate limiting
}
```

**Standard Error Codes:**
- `RATE_LIMITED` - Too many requests (429)
- `UNAUTHORIZED` - Authentication required (401)
- `INVALID_TOKEN` - Token expired or invalid (401)
- `FORBIDDEN` - Insufficient permissions (403)
- `INVALID_REQUEST` - Bad request format (400)
- `INVALID_OTP` - OTP verification failed (400)
- `INTERNAL_ERROR` - Server error (500)

**Status Code Mapping:**
- `400 Bad Request` - Invalid request format or parameters
- `401 Unauthorized` - Missing or invalid authentication
- `403 Forbidden` - Authenticated but insufficient permissions
- `404 Not Found` - Resource not found
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Server-side error

### Authentication
- JWT tokens in `Authorization: Bearer <token>` header
- Token expiration configured via `JWT_EXPIRATION` environment variable
- Refresh tokens available via `/api/v1/auth/refresh-token`
- All authenticated endpoints require valid JWT token

## Security Best Practices

### Rate Limiting
- **OTP Requests**: Limited per phone number to prevent abuse
- **Rate Limit Response**: Returns `429 Too Many Requests` with `retry_after_seconds`
- **Implementation**: Redis-based rate limiting with sliding window
- **Default Retry**: 600 seconds (10 minutes) for OTP requests

### Token Management
- **JWT Secret**: Must be strong, randomly generated (minimum 32 characters)
- **Token Expiration**: Configurable via `JWT_EXPIRATION` (default: 1 hour)
- **Refresh Token Expiration**: Configurable via `REFRESH_TOKEN_EXPIRATION` (default: 7 days)
- **Token Storage**: Clients must store tokens securely (not in localStorage for web)
- **Token Rotation**: Refresh tokens should be rotated on use

### PIN Security
- **PIN Storage**: Must be hashed using Argon2
- **PIN Verification**: Rate-limited to prevent brute force
- **PIN Requirements**: Minimum length and complexity (enforced by client)
- **PIN Reset**: Requires OTP verification

### Device Linking Security
- **Linking Sessions**: Time-limited (expire after set duration)
- **Approval Required**: Device linking requires approval from existing device
- **Session Tokens**: One-time use tokens for device linking
- **Device Limits**: Maximum devices per user (configurable)

### OTP Security
- **OTP Expiration**: 180 seconds (3 minutes)
- **OTP Retry Limits**: Maximum attempts per phone number
- **OTP Rate Limiting**: Prevents SMS bombing attacks
- **OTP Storage**: Stored in Redis with expiration

### Data Encryption
- **End-to-End Encryption**: All messages encrypted with AES-GCM
- **Signal Protocol**: Custom implementation using ed25519/x25519
- **Key Management**: Identity keys, prekeys, and session keys stored securely
- **Database Encryption**: Sensitive fields (phone_number_hash) stored as hashes

### CORS Configuration
- **Development**: Allow any origin (for testing)
- **Production**: Restrict to specific domains
- **Headers**: Allow necessary headers for authentication
- **Methods**: Allow required HTTP methods

## Error Handling Standards

### Error Propagation
- Use `anyhow::Result<T>` for error handling in application layer
- Convert to HTTP responses in handler layer
- Log errors with appropriate log levels (error, warn, info)

### Error Logging
- **Error Level**: Use for unexpected errors requiring attention
- **Warn Level**: Use for recoverable errors or validation failures
- **Info Level**: Use for normal flow events (OTP sent, device linked)
- **Debug Level**: Use for detailed debugging information

### Error Response Guidelines
- **User-Facing Errors**: Provide clear, actionable error messages
- **Security Errors**: Generic messages to prevent information leakage
- **Rate Limiting**: Include `retry_after_seconds` for client retry logic
- **Validation Errors**: Include field-level error details when appropriate

### Error Recovery
- **Retry Logic**: Clients should implement exponential backoff
- **Circuit Breaker**: Consider for external service calls
- **Graceful Degradation**: Fallback mechanisms for non-critical features

## Deployment Procedures

### Environment Variables
Required environment variables:
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `JWT_SECRET` - Secret key for JWT signing (minimum 32 characters)
- `JWT_EXPIRATION` - JWT token expiration in seconds (default: 3600)
- `REFRESH_TOKEN_EXPIRATION` - Refresh token expiration in seconds (default: 604800)
- `SERVER_HOST` - Server bind address (default: 0.0.0.0)
- `SERVER_PORT` - Server port (default: 8000)

### Database Migrations
1. **Run Migrations**: Use Sea-ORM CLI to apply migrations
   ```bash
   sea-orm-cli migrate up
   ```
2. **Migration Order**: Migrations run in chronological order
3. **Rollback**: Use `sea-orm-cli migrate down` for rollback
4. **Production**: Always backup database before migrations

### Docker Deployment
1. **Build Image**: `docker build -t vyry-api .`
2. **Run Container**: Use docker-compose for development
3. **Environment**: Set all required environment variables
4. **Volumes**: Mount database and Redis volumes for persistence
5. **Health Check**: Configure health check endpoint

### Production Checklist
- [ ] Set strong `JWT_SECRET` (32+ characters, random)
- [ ] Configure production database with SSL
- [ ] Set up Redis with password authentication
- [ ] Enable rate limiting with appropriate limits
- [ ] Configure CORS for production domains
- [ ] Set up monitoring and logging
- [ ] Configure backup strategy for database
- [ ] Set up SSL/TLS certificates
- [ ] Configure firewall rules
- [ ] Set up health check monitoring

### Health Check
- **Endpoint**: `GET /health`
- **Response**: `{"status": "ok", "service": "vyry-api", "version": "1.0.0"}`
- **Use Case**: Load balancer health checks, monitoring
- **Dependencies**: Should not check database/Redis (lightweight check)

## Performance Guidelines

### Database Optimization
- **Connection Pooling**: Use Sea-ORM connection pooling
- **Query Optimization**: Use indexes on frequently queried fields
- **Batch Operations**: Batch database operations when possible
- **Lazy Loading**: Avoid N+1 queries with proper joins

### Redis Caching Strategy
- **OTP Storage**: Store OTPs in Redis with TTL
- **Rate Limiting**: Use Redis for distributed rate limiting
- **Session Storage**: Consider Redis for session data
- **Pub/Sub**: Use Redis pub/sub for real-time messaging

### WebSocket Connection Management
- **Connection Limits**: Monitor and limit concurrent WebSocket connections
- **Heartbeat**: Implement ping/pong for connection health
- **Reconnection**: Clients should implement exponential backoff reconnection
- **Message Queuing**: Queue messages for offline clients

### Query Performance
- **Indexes**: Ensure indexes on foreign keys and frequently queried fields
- **Pagination**: Implement pagination for list endpoints
- **Selective Fields**: Return only required fields in responses
- **Eager Loading**: Use eager loading to avoid N+1 queries

### Resource Limits
- **Request Timeout**: Configure appropriate request timeouts
- **Connection Limits**: Limit concurrent database connections
- **Memory Usage**: Monitor memory usage, especially for WebSocket connections
- **CPU Usage**: Profile CPU-intensive operations

### Monitoring and Logging
- **Structured Logging**: Use structured logging with tracing
- **Metrics**: Track request rates, error rates, latency
- **Alerting**: Set up alerts for error rates and latency spikes
- **Performance Profiling**: Regular performance profiling in production

## Domain Context
- **Signal Protocol Implementation**: Custom implementation using ed25519-dalek for identity keys and signatures, x25519-dalek for prekeys and key exchange
- **Multi-device Support**: Users can register multiple devices, each with unique Signal keys
- **One-time Prekeys**: 100 prekeys per device for initial session establishment
- **End-to-end Encryption**: All messages encrypted using AES-GCM
- **Real-time Messaging**: WebSocket connections with Redis pub/sub for real-time message delivery
- **JWT Authentication**: Secure token-based authentication
- **Push Notifications**: Support for FCM/APNS tokens

## Important Constraints
- **Rust nightly compatibility**: Current zerocopy dependency requires special handling
- **Database migrations**: Must use Sea-ORM migrations for schema changes
- **Security**: All messages must be encrypted end-to-end
- **Scalability**: Redis pub/sub for horizontal scaling
- **Docker-first**: Development and deployment via Docker containers

## External Dependencies
- **PostgreSQL**: Primary data storage for users, conversations, messages
- **Redis**: Pub/sub for real-time messaging and presence tracking
- **Adminer**: Database administration UI (via Docker)
- **Signal Protocol libraries**: ed25519-dalek, x25519-dalek for cryptographic operations
