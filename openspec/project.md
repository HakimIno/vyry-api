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
