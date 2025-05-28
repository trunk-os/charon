use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Global {
    pub name: String,
    pub variables: HashMap<String, String>,
}

impl Global {
    pub fn var(&self, name: &str) -> Option<String> {
        self.variables.get(name).cloned()
    }
}

impl Ord for Global {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Global {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

pub struct GlobalRegistry {
    pub root: PathBuf,
}

impl GlobalRegistry {
    pub fn get(&self, name: &str) -> Result<Global> {
        Ok(serde_json::from_reader(
            std::fs::OpenOptions::new()
                .read(true)
                .open(self.root.join("variables").join(&format!("{}.json", name)))?,
        )?)
    }

    pub fn set(&self, global: Global) -> Result<()> {
        let pb = self.root.join("variables");

        std::fs::create_dir_all(&pb)?;

        Ok(serde_json::to_writer_pretty(
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(pb.join(&format!("{}.json", &global.name)))?,
            &global,
        )?)
    }
}
