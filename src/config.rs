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
}

impl Config {
    pub fn from_env() -> Self {
        let axum_server_port = std::option_env!("AXUM_SERVER_PORT")
            .and_then(|port| port.parse::<u16>().ok())
            .unwrap_or(8080);

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let parsed_url = Url::parse(&database_url).expect("Invalid DATABASE_URL format");
        if parsed_url.scheme() != "postgres" {
            panic!("DATABASE_URL must use the 'postgres' scheme");
        }

        let shortlink_length = std::option_env!("SHORTLINK_LENGTH")
            .and_then(|length| length.parse::<usize>().ok())
            .unwrap_or(8);
        if shortlink_length > 16 {
            panic!("SHORTLINK_LENGTH support max to 16");
        }

        Self {
            axum_server_port,
            database_url: database_url,
            shortlink_base_url: std::option_env!("SHORTLINK_BASE_URL")
                .unwrap_or(&format!("http://localhost:{}", axum_server_port))
                .to_string(),
            shortlink_length,
            shortlink_expire_days: std::option_env!("SHORTLINK_EXPIRE_DAYS")
                .and_then(|days| days.parse::<u16>().ok())
                .unwrap_or(10),
            shortlink_max_hash_retries: std::option_env!("SHORTLINK_MAX_HASH_RETRIES")
                .and_then(|retries| retries.parse::<u16>().ok())
                .unwrap_or(4),
        }
    }
}

// impl axum::extract::FromRef<Config> for Config {
//     fn from_ref(&self) -> Self {
//         Config {
//             axum_server_port: self.axum_server_port,
//             database_url: self.database_url.clone(),
//             shortlink_base_url: self.shortlink_base_url.clone(),
//             shortlink_length: self.shortlink_length,
//             shortlink_expire_days: self.shortlink_expire_days,
//             shortlink_max_hash_retries: self.shortlink_max_hash_retries,
//         }
//     }
// }
