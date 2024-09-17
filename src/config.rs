use ::config::ConfigError;
use serde::{Deserialize, Serialize};
use deadpool_postgres::PoolConfig;

#[derive(Debug, Deserialize, Serialize)]
pub struct BotConfig {
    pub discord_token: String,
    pub pg: deadpool_postgres::Config,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            discord_token: "".to_string(),
            pg: deadpool_postgres::Config {
                host: Some("127.0.0.1".to_string()),
                port: Some(5438),
                user: Some("postgres".to_string()),
                password: Some("postgres".to_string()),
                dbname: Some("sylliba".to_string()),
                pool: Some(PoolConfig {
                    max_size: 16,
                    ..Default::default()
                }),
                ..Default::default()
            }
        }
    }
}

impl TryFrom<::config::ConfigBuilder<::config::builder::DefaultState>> for BotConfig {
    type Error = ConfigError;

    fn try_from(cfg: ::config::ConfigBuilder<::config::builder::DefaultState>) -> Result<Self, ConfigError> {
        cfg.try_into()
    }
}

impl From<BotConfig> for ::config::ConfigBuilder<::config::builder::DefaultState> {
    fn from(cfg: BotConfig) -> Self {
        ::config::ConfigBuilder::try_from(cfg).unwrap()
    }
}

impl BotConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let builder = ::config::Config::builder()
            .set_default("default", "1")?;

        builder.try_into()
    }
}
