use config::Config;

const ENVIRONMENT_PREFIX: &str = "OWARCADEBOT";

#[derive(Debug)]
pub struct ArcadeBotConfig {
    inner: Config,
}

impl ArcadeBotConfig {
    pub fn discord_token(&self) -> Result<String, failure::Error> {
        Ok(self.inner.get_str("DISCORD_TOKEN")?)
    }
    pub fn s3_bucket(&self) -> Result<String, failure::Error> {
        Ok(self.inner.get_str("S3_BUCKET")?)
    }
    pub fn s3_key_config(&self) -> Result<String, failure::Error> {
        Ok(self.inner.get_str("S3_KEY_CONFIG")?)
    }
    pub fn s3_key_gamestate(&self) -> Result<String, failure::Error> {
        Ok(self.inner.get_str("S3_KEY_GAMESTATE")?)
    }
}

pub fn load() -> Result<ArcadeBotConfig, failure::Error> {
    let mut c = config::Config::default();

    let config_env = config::Environment::with_prefix(ENVIRONMENT_PREFIX).separator("__");
    c.merge(config_env)?;

    Ok(ArcadeBotConfig { inner: c })
}
