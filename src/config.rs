use std::env;

pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self, BridgeError> {
        let url = env::var("DATABASE_URL")
            .map_err(|_| BridgeError::Config("DATABASE_URL not set".into()))?;
        
        // Parse URL or use default values
        Ok(DatabaseConfig {
            host: env::var("DB_HOST").unwrap_or_else(|_| "localhost".into()),
            port: env::var("DB_PORT").unwrap_or_else(|_| "5432".into())
                .parse()
                .map_err(|_| BridgeError::Config("Invalid DB_PORT".into()))?,
            user: env::var("DB_USER").unwrap_or_else(|_| "postgres".into()),
            password: env::var("DB_PASSWORD")
                .map_err(|_| BridgeError::Config("DB_PASSWORD required".into()))?,
            database: env::var("DB_NAME").unwrap_or_else(|_| "tor_metrics".into()),
        })
    }
}