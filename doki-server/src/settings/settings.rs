use config::{Case, Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Server {
    pub host: String,
    pub port: i32,
}

#[derive(Debug, Deserialize, Default)]
pub struct Logging {
    pub log_level: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct Settings {
    pub server: Server,
    pub logging: Logging,
}

impl Settings {
    pub fn new(location: &str, env_prefix: &str) -> anyhow::Result<Self> {
        let s = Config::builder()
            .add_source(File::with_name(location))
            .add_source(Environment::with_prefix(env_prefix).convert_case(Case::Snake).separator("__").prefix_separator("__"))
            .build()?;
        let settings = s.try_deserialize()?;
        Ok(settings)
    }
}
