use crate::grpc::control_client::ControlClient as GRPCControlClient;
use crate::grpc::status_client::StatusClient as GRPCStatusClient;
use crate::ProtoPackageTitleWithRoot;
use anyhow::Result;
use std::path::PathBuf;
use tonic::{transport::Channel, Request};

#[derive(Debug, Clone)]
pub struct Client {
    socket: PathBuf,
}

pub struct StatusClient {
    client: GRPCStatusClient<Channel>,
}

pub struct ControlClient {
    client: GRPCControlClient<Channel>,
}

impl Client {
    pub fn new(socket: PathBuf) -> anyhow::Result<Self> {
        Ok(Self { socket })
    }

    pub async fn status(&self) -> anyhow::Result<StatusClient> {
        let client =
            GRPCStatusClient::connect(format!("unix://{}", self.socket.to_str().unwrap())).await?;
        Ok(StatusClient { client })
    }

    pub async fn control(&self) -> anyhow::Result<ControlClient> {
        let client =
            GRPCControlClient::connect(format!("unix://{}", self.socket.to_str().unwrap())).await?;
        Ok(ControlClient { client })
    }
}

impl StatusClient {
    pub async fn ping(&mut self) -> Result<()> {
        Ok(self.client.ping(Request::new(())).await?.into_inner())
    }
}

impl ControlClient {
    pub async fn write_unit(
        &mut self,
        name: &str,
        version: &str,
        volume_root: PathBuf,
    ) -> Result<()> {
        let out = ProtoPackageTitleWithRoot {
            name: name.into(),
            version: version.into(),
            volume_root: volume_root.to_str().unwrap().to_string(),
        };

        Ok(self
            .client
            .write_unit(Request::new(out))
            .await?
            .into_inner())
    }
}
