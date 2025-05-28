use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Package {
    pub title: PackageTitle,
    pub description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<PackageTitle>,
    pub source: Source,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networking: Option<Networking>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<Storage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<System>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Resources>,
}

impl PartialOrd for Package {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.title.partial_cmp(&other.title)
    }
}

impl Ord for Package {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.title.cmp(&other.title)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PackageTitle {
    pub name: String,
    pub version: String,
}

impl PartialOrd for PackageTitle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.name.partial_cmp(&other.name) {
            Some(ord) => {
                if ord == std::cmp::Ordering::Equal {
                    self.version.partial_cmp(&other.version)
                } else {
                    Some(ord)
                }
            }
            None => self.version.partial_cmp(&other.version),
        }
    }
}

impl Ord for PackageTitle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.name.cmp(&other.name) {
            std::cmp::Ordering::Equal => self.version.cmp(&other.version),
            x => x,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Source {
    #[serde(rename = "http")]
    HTTP(String),
    #[serde(rename = "container")]
    Container(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Networking {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub forward_ports: Vec<u16>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub expose_ports: Vec<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Storage {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<Volume>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Volume {
    pub name: String,
    pub size: u64,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub recreate: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub private: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct System {
    // --pid host
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub host_pid: bool,
    // --net host
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub host_net: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub privileged: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Resources {
    pub cpus: u8,
    pub memory: u32,
    // probably something to bring in PCI devices to appease the crypto folks
}

impl Package {
    pub fn from_file(name: &Path) -> Result<Self> {
        Ok(serde_json::from_reader(
            std::fs::OpenOptions::new().open(name)?,
        )?)
    }

    pub fn load(root: &Path, name: String, version: String) -> Result<Self> {
        let pb = PathBuf::from_str(root.as_os_str().try_into()?)?.join("packages");
        let name = pb.join(name).join(&format!("{}.json", version));
        Ok(serde_json::from_reader(
            std::fs::OpenOptions::new().open(name)?,
        )?)
    }

    pub fn write(&self, root: &Path) -> Result<()> {
        let pb = PathBuf::from_str(root.as_os_str().try_into()?)?.join("packages");
        std::fs::create_dir_all(&pb)?;

        let name = pb
            .join(&self.title.name)
            .join(&format!("{}.json", self.title.version));

        Ok(serde_json::to_writer(
            std::fs::OpenOptions::new()
                .create_new(true)
                .truncate(true)
                .write(true)
                .open(name)?,
            self,
        )?)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_io() {}
}
