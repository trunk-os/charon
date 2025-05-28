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

pub struct PackageTitle {
    pub name: String,
    pub version: String,
}

pub struct Source {
    pub http: Option<String>,
    pub container: Option<String>,
}

pub struct Networking {
    pub forward_ports: Vec<u16>,
    pub expose_ports: Vec<u16>,
    pub internal_network: Option<String>,
    pub hostname: Option<String>,
}

pub struct Storage {
    pub volumes: Vec<Volume>,
}

pub struct Volume {
    name: String,
    size: u64,
    recreate: bool,
    private: bool,
}

pub struct System {
    // --pid host
    host_pid: bool,
    // --net host
    host_net: bool,
    capabilities: Vec<String>,
    privileged: bool,
}

pub struct Resources {
    cpus: u8,
    memory: u32,
    // probably something to bring in PCI devices to appease the crypto folks
}
