use config::ConfigError;
use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize)]
/// App config
pub struct Configuration {
    /// Level of Rust logging
    pub rust_log: String,

    /// Database URL for Postgres
    pub database_url: String,

    /// JWT secret used for key creation
    pub jwt_secret: String,

    /// Host URL
    pub allowed_host: String,
}

impl Configuration {
    /// A new configuration read from the env
    pub fn new() -> Result<Self, ConfigError> {
        let builder = config::Config::builder().add_source(config::Environment::default());

        builder.build()?.try_deserialize()
    }
}

lazy_static! {
    pub static ref CONFIG: Configuration = Configuration::new().expect("Config can be loaded");
}
