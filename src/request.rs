use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Port(u16),
    Volume(PathBuf),
    Network(SocketAddr),
}
