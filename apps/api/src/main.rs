use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod config;
pub mod handlers;
mod middleware;
mod websocket;

use config::Config;
use handlers::{auth, health, keys};
use middleware::auth::AuthMiddleware;
use websocket::{connection::ConnectionManager, handler::websocket_handler};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,api=debug,actix_web=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let config_data = web::Data::new(config.clone());
    tracing::info!("Starting vyry API server...");

    let db = infrastructure::database::init_database(&config.database_url).await?;
    let redis_conn = infrastructure::database::init_redis(&config.redis_url).await?;

    let connection_manager = web::Data::new(ConnectionManager::new());

    let server_addr = format!("{}:{}", config.server_host, config.server_port);
    tracing::info!("Server listening on {}", server_addr);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(AuthMiddleware)
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(redis_conn.clone()))
            .app_data(config_data.clone())
            .app_data(connection_manager.clone())
            // Health
            .service(health::health_check)
            // Auth - OTP
            .service(auth::request_otp)
            .service(auth::verify_otp)
            // Auth - Profile & PIN
            .service(auth::setup_profile)
            .service(auth::setup_pin)
            .service(auth::verify_pin)
            // Auth - Token
            .service(auth::refresh_token)
            // Device Linking
            .service(auth::create_linking_session)
            .service(auth::complete_linking)
            .service(auth::approve_linking)
            // Device Management
            .service(auth::list_devices)
            .service(auth::unlink_device)
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
