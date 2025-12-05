#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub refresh_token_expiration: i64,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            redis_url: std::env::var("REDIS_URL")?,
            jwt_secret: std::env::var("JWT_SECRET")?,
            jwt_expiration: std::env::var("JWT_EXPIRATION")?.parse()?,
            refresh_token_expiration: std::env::var("REFRESH_TOKEN_EXPIRATION")?.parse()?,
            server_host: std::env::var("SERVER_HOST")?,
            server_port: std::env::var("SERVER_PORT")?.parse()?,
        })
    }
}
