use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub repository: PathBuf,
}

impl Config {
    pub fn from_file(filename: PathBuf) -> Result<Self> {
        let f = std::fs::OpenOptions::new().read(true).open(&filename)?;
        Ok(serde_yaml_ng::from_reader(&f)?)
    }
}
