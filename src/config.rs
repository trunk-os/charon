use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

use crate::Registry;

#[derive(Debug, Clone, Deserialize, Default)]
pub enum LogLevel {
    #[serde(rename = "warn")]
    Warn,
    #[default]
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "trace")]
    Trace,
}

impl From<LogLevel> for tracing::Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }
}

impl From<tracing::Level> for LogLevel {
    fn from(value: tracing::Level) -> Self {
        match value {
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::ERROR => LogLevel::Error,
            tracing::Level::DEBUG => LogLevel::Debug,
            tracing::Level::TRACE => LogLevel::Trace,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub registry: PathBuf,
    pub socket: PathBuf,
    pub log_level: Option<LogLevel>,
    pub debug: Option<bool>,
}

impl Config {
    pub fn from_file(filename: PathBuf) -> Result<Self> {
        let f = std::fs::OpenOptions::new().read(true).open(&filename)?;
        let this: Self = serde_yaml_ng::from_reader(&f)?;
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Into::<tracing::Level>::into(
                this.log_level.clone().unwrap_or_default(),
            ))
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
        info!("Configuration parsed successfully.");
        Ok(this)
    }

    pub fn registry(&self) -> Registry {
        Registry::new(self.registry.clone())
    }

    pub fn debug(&self) -> bool {
        self.debug.unwrap_or_default()
    }
}
