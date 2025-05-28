use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Package {
    pub title: PackageTitle,
    pub description: String,
    pub dependencies: Vec<PackageTitle>,
    pub source: Source,
    pub networking: Option<Networking>,
    pub storage: Option<Storage>,
    pub system: Option<System>,
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
    pub forward_ports: Vec<u16>,
    pub expose_ports: Vec<u16>,
    pub internal_network: Option<String>,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Storage {
    pub volumes: Vec<Volume>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Volume {
    name: String,
    size: u64,
    recreate: bool,
    private: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct System {
    // --pid host
    host_pid: bool,
    // --net host
    host_net: bool,
    capabilities: Vec<String>,
    privileged: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Resources {
    cpus: u8,
    memory: u32,
    // probably something to bring in PCI devices to appease the crypto folks
}
