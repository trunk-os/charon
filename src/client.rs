use crate::grpc::status_client::StatusClient as GRPCStatusClient;
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

impl Client {
    pub fn new(socket: PathBuf) -> anyhow::Result<Self> {
        Ok(Self { socket })
    }

    pub async fn status(&self) -> anyhow::Result<StatusClient> {
        let client =
            GRPCStatusClient::connect(format!("unix://{}", self.socket.to_str().unwrap())).await?;
        Ok(StatusClient { client })
    }
}

impl StatusClient {
    pub async fn ping(&mut self) -> Result<()> {
        Ok(self.client.ping(Request::new(())).await?.into_inner())
    }
}
