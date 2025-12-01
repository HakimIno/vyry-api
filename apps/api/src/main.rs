use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_cors::Cors;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod handlers;
mod middleware;
mod websocket;

use config::Config;
use handlers::{auth, health};
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
    tracing::info!("Starting chat API server...");

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
            .app_data(config_data.clone())
            .app_data(connection_manager.clone())
            .service(health::health_check)
            .service(auth::register)
            .service(auth::login)
            .service(auth::get_me)
            .service(websocket_handler)
    })
    .bind(&server_addr)?
    .run()
    .await?;

    Ok(())
}
