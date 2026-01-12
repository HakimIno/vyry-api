#[derive(Clone)]
pub struct Config {
    // Database URLs
    pub postgres_url: String,
    pub redis_url: String,
    // ScyllaDB (for future use)
    #[allow(dead_code)]
    pub scylladb_url: Option<String>,
    
    // JWT Configuration
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub refresh_token_expiration: i64,
    
    // Server Configuration
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            // Support both DATABASE_URL (legacy) and POSTGRES_URL
            postgres_url: std::env::var("POSTGRES_URL")
                .or_else(|_| std::env::var("DATABASE_URL"))
                .unwrap_or_else(|_| "postgresql://vyryuser:vyrypass@localhost:5432/vyrydb".to_string()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            scylladb_url: std::env::var("SCYLLADB_URL").ok(),
            
            jwt_secret: std::env::var("JWT_SECRET")?,
            jwt_expiration: std::env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()?,
            refresh_token_expiration: std::env::var("REFRESH_TOKEN_EXPIRATION")
                .unwrap_or_else(|_| "604800".to_string())
                .parse()?,
            server_host: std::env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()?,
        })
    }
    
    /// Get database URL (backward compatibility)
    pub fn database_url(&self) -> &str {
        &self.postgres_url
    }
}
