use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    // axum
    pub axum_server_port: u16,

    // database
    pub database_url: String,

    // shortlink
    pub shortlink_base_url: String,
    pub shortlink_length: usize,
    pub shortlink_expire_days: u16,
    pub shortlink_max_hash_retries: u16,

    // log
    pub logging_file_location: String,
    pub logging_file_name: String,
    pub logging_level: String,
}

impl Config {
    pub fn from_env() -> Self {
        let axum_server_port = std::env::var("AXUM_SERVER_PORT")
            .ok()
            .and_then(|port| port.parse::<u16>().ok())
            .unwrap_or(8080);

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let parsed_url = Url::parse(&database_url).expect("Invalid DATABASE_URL format");
        if parsed_url.scheme() != "postgres" {
            panic!("DATABASE_URL must use the 'postgres' scheme");
        }

        let shortlink_length = std::env::var("SHORTLINK_LENGTH")
            .ok()
            .and_then(|length| length.parse::<usize>().ok())
            .unwrap_or(8);
        if shortlink_length > 16 {
            panic!("SHORTLINK_LENGTH support max to 16");
        }

        let logging_file_location =
            std::env::var("LOGGING_FILE_LOCATION").expect("LOGGING_FILE_LOCATION must be set");
        let logging_file_name =
            std::env::var("LOGGING_FILE_NAME").expect("LOGGING_FILE_NAME must be set");

        let logging_level = std::env::var("LOGGING_LEVEL")
            .ok()
            .unwrap_or_else(|| "info".to_string())
            .to_lowercase();
        match logging_level.as_str() {
            "error" | "warn" | "info" | "debug" | "trace" => {}
            _ => panic!("LOGGING_LEVEL must be one of: error, warn, info, debug, trace"),
        }

        Self {
            axum_server_port,
            database_url,
            shortlink_base_url: std::env::var("SHORTLINK_BASE_URL")
                .ok()
                .unwrap_or_else(|| format!("http://localhost:{}", axum_server_port)),
            shortlink_length,
            shortlink_expire_days: std::env::var("SHORTLINK_EXPIRE_DAYS")
                .ok()
                .and_then(|days| days.parse::<u16>().ok())
                .unwrap_or(10),
            shortlink_max_hash_retries: std::env::var("SHORTLINK_MAX_HASH_RETRIES")
                .ok()
                .and_then(|retries| retries.parse::<u16>().ok())
                .unwrap_or(4),
            logging_file_location,
            logging_file_name,
            logging_level,
        }
    }
}
