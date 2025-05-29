use crate::{Global, GlobalRegistry, PromptCollection};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const PACKAGE_SUBPATH: &str = "packages";

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Package {
    pub title: PackageTitle,
    pub description: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptCollection>,
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

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PackageTitle {
    pub name: String,
    pub version: String,
}

impl PartialOrd for PackageTitle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.name.partial_cmp(&other.name) {
            Some(std::cmp::Ordering::Equal) => self.version.partial_cmp(&other.version),
            Some(ord) => Some(ord),
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

impl Default for Source {
    fn default() -> Self {
        Source::Container("scratch".into())
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Networking {
    pub forward_ports: Vec<u16>,
    pub expose_ports: Vec<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Storage {
    pub volumes: Vec<Volume>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Volume {
    pub name: String,
    pub size: u64,
    pub recreate: bool,
    pub private: bool,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct System {
    // --pid host
    pub host_pid: bool,
    // --net host
    pub host_net: bool,
    pub capabilities: Vec<String>,
    pub privileged: bool,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Resources {
    pub cpus: u8,
    pub memory: u32,
    // probably something to bring in PCI devices to appease the crypto folks
}

impl Package {
    pub fn from_file(name: &Path) -> Result<Self> {
        Ok(serde_json::from_reader(
            std::fs::OpenOptions::new().read(true).open(name)?,
        )?)
    }

    pub fn globals(&self, root: &PathBuf) -> Result<Global> {
        let registry = GlobalRegistry { root: root.clone() };

        registry.get(&self.title.name)
    }
}

pub struct Registry {
    root: PathBuf,
}

impl Registry {
    pub fn load(&self, name: &str, version: &str) -> Result<Package> {
        let pb = self.root.join(PACKAGE_SUBPATH).join(name);
        Ok(Package::from_file(&pb.join(&format!("{}.json", version)))?)
    }

    pub fn write(&self, package: &Package) -> Result<()> {
        let pb = self.root.join(PACKAGE_SUBPATH).join(&package.title.name);
        std::fs::create_dir_all(&pb)?;

        let name = pb.join(&format!("{}.json.tmp", package.title.version));
        let f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&name)?;

        serde_json::to_writer_pretty(&f, &package)?;

        Ok(std::fs::rename(
            &name,
            &pb.join(&format!("{}.json", package.title.version)),
        )?)
    }

    pub fn globals(&self, package: &Package) -> Result<Global> {
        package.globals(&self.root)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Global, GlobalRegistry, Package, PackageTitle, Registry, Variables};

    #[test]
    fn io() {
        let dir = tempfile::tempdir().unwrap();
        let table = &[Package {
            title: PackageTitle {
                name: "plex".into(),
                version: "1.2.3".into(),
            },
            ..Default::default()
        }];

        let pr = Registry {
            root: dir.path().to_path_buf(),
        };

        for item in table {
            assert!(pr.write(item).is_ok());
            let cmp = pr.load(&item.title.name, &item.title.version).unwrap();
            assert_eq!(item.clone(), cmp);
        }
    }

    #[test]
    fn globals() {
        let dir = tempfile::tempdir().unwrap();
        let packages = &[Package {
            title: PackageTitle {
                name: "plex".into(),
                version: "1.2.3".into(),
            },
            ..Default::default()
        }];

        let pr = Registry {
            root: dir.path().to_path_buf(),
        };

        for item in packages {
            assert!(pr.write(item).is_ok());
        }

        let mut variables = Variables::default();
        variables.insert("foo".into(), "bar".into());
        variables.insert("baz".into(), "quux".into());

        let globals = &[Global {
            name: "plex".into(),
            variables: variables.clone(),
        }];

        let gr = GlobalRegistry {
            root: dir.path().to_path_buf(),
        };

        for item in globals {
            assert!(gr.set(item).is_ok());
        }

        assert_eq!(pr.globals(&packages[0]).unwrap(), globals[0]);
    }
}
