use dotenvy::dotenv;
use serde::Deserialize;
use tracing::level_filters::LevelFilter;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Config {
    pub rust_log: LevelFilter,
}

#[derive(Deserialize, Debug, Clone)]
struct ConfigEnv {
    #[serde(alias = "RUST_LOG", default = "default_log_level_str")]
    pub rust_log_str: String,
}

fn default_log_level_str() -> String {
    "Info".to_string()
}

impl Config {
    pub fn init() -> Result<Self, String> {
        if let Err(e) = dotenv() {
            eprintln!("Info: Failed to load .env file: {}", e);
        }

        let config_env = envy::from_env::<ConfigEnv>()
            .map_err(|e| format!("Failed to load configuration: {e}"))?;

        let rust_log_level = LevelFilter::from_str(&config_env.rust_log_str).map_err(|_| {
            format!(
                "Invalid log level \'{}!\'. Supported levels are: trace, debug, info, warn, error, off",
                config_env.rust_log_str
            )
        })?;

        Ok(Config {
            rust_log: rust_log_level,
        })
    }
}
