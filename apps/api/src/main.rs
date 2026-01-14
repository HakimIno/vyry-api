use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub mod config;
pub mod handlers;
mod middleware;
mod websocket;
 
use config::Config;
use handlers::{auth, health, keys};
use middleware::auth::AuthMiddleware;
use middleware::rate_limit::PerIpRateLimitMiddleware;
use websocket::{connection::ConnectionManager, handler::websocket_handler};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging with JSON support
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,api=debug,actix_web=info".into());
    
    // Use compact format for better readability
    let is_json = std::env::var("LOG_FORMAT").unwrap_or_default() == "json";
    
    if is_json {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true)
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(false) // Hide target for cleaner output
                    .with_file(false) // Hide file path for cleaner output
                    .with_line_number(false) // Hide line number for cleaner output
                    .compact()
            )
            .init();
    }

    let config = Config::from_env()?;
    let config_data = web::Data::new(config.clone());
    tracing::info!("Starting vyry API server...");

    // Initialize database connections
    let db_connections = infrastructure::database::DatabaseConnections::new(
        &config.postgres_url,
        &config.redis_url,
    ).await?;
    
    // For backward compatibility, expose individual connections
    let db = db_connections.postgres.clone();
    let redis_conn = db_connections.redis.clone();

    let connection_manager = web::Data::new(ConnectionManager::new());

    let server_addr = format!("{}:{}", config.server_host, config.server_port);
    tracing::info!("Server listening on {}", server_addr);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        // Global rate limiter: 100 requests per minute per IP
        let per_ip_rate_limit = PerIpRateLimitMiddleware::new(100);
        
        // Stricter rate limit for auth endpoints: 10 requests per minute per IP
        let auth_rate_limit = PerIpRateLimitMiddleware::new(10);

        App::new()
            .wrap(cors)
            .wrap(TracingLogger::default())
            .wrap(per_ip_rate_limit) // Global rate limiting
            .wrap(AuthMiddleware)
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(redis_conn.clone()))
            .app_data(config_data.clone())
            .app_data(connection_manager.clone())
            // Health (no rate limit)
            .service(health::health_check)
            // Auth endpoints with stricter rate limiting
            .service(
                web::scope("/api/v1/auth")
                    .wrap(auth_rate_limit)
                    .service(auth::request_otp)
                    .service(auth::verify_otp)
                    .service(auth::get_profile)
                    .service(auth::setup_profile)
                    .service(auth::setup_pin)
                    .service(auth::verify_pin)
                    .service(auth::refresh_token)
            )
            // Device endpoints
            .service(
                web::scope("/api/v1/devices")
                    .service(auth::create_linking_session)
                    .service(auth::complete_linking)
                    .service(auth::approve_linking)
                    .service(auth::list_devices)
                    .service(auth::unlink_device)
            )
            // Keys
            .service(keys::get_prekey_bundle)
            // WebSocket
            .service(websocket_handler)
    })
    .bind(&server_addr)?
    .run()
    .await?;

    Ok(())
}
