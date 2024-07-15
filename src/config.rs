use config::{ConfigError, Environment, File};
use dirs::home_dir;
use serde::Deserialize;
use tracing::Level;

#[derive(Deserialize, Debug, Default)]
pub(crate) struct Config {
    pub(crate) logging: Logging,
}

impl Config {
    pub(crate) fn new() -> Result<Config, ConfigError> {
        let settings = config::Config::builder()
            .add_source(File::with_name("/etc/swayboard/config").required(false));
        let settings = if let Some(home_dir) = home_dir() {
            settings.add_source(
                File::from(home_dir.join(".config/swayboard/config.toml")).required(false),
            )
        } else {
            settings
        };
        let settings = settings
            .add_source(Environment::with_prefix("swayboard").separator("_"))
            .build()?;
        settings.try_deserialize::<Config>()
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct Logging {
    pub(crate) level: LoggingLevel,
}

#[derive(Deserialize, Debug, Default)]
pub(crate) enum LoggingLevel {
    Error,
    #[default]
    Warn,
    Info,
    Debug,
    Trace,
}

impl LoggingLevel {
    pub(crate) fn to_tracing_level(&self) -> Level {
        match self {
            LoggingLevel::Error => Level::ERROR,
            LoggingLevel::Warn => Level::WARN,
            LoggingLevel::Info => Level::INFO,
            LoggingLevel::Debug => Level::DEBUG,
            LoggingLevel::Trace => Level::TRACE,
        }
    }
}
