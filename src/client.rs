use crate::grpc::query_client::QueryClient as GRPCQueryClient;
use crate::grpc::status_client::StatusClient as GRPCStatusClient;
use crate::{grpc::control_client::ControlClient as GRPCControlClient, ProtoPackageTitle};
use crate::{
    InputType, Prompt, PromptCollection, PromptResponses, ProtoPackageTitleWithRoot,
    ProtoPromptResponse, ProtoPromptResponses, ProtoType,
};
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

pub struct QueryClient {
    client: GRPCQueryClient<Channel>,
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

    pub async fn query(&self) -> anyhow::Result<QueryClient> {
        let client =
            GRPCQueryClient::connect(format!("unix://{}", self.socket.to_str().unwrap())).await?;
        Ok(QueryClient { client })
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

impl QueryClient {
    pub async fn get_prompts(&mut self, name: &str, version: &str) -> Result<PromptCollection> {
        let title = ProtoPackageTitle {
            name: name.into(),
            version: version.into(),
        };

        let prompts = self
            .client
            .get_prompts(Request::new(title))
            .await?
            .into_inner();

        let mut out = Vec::new();

        for prompt in &prompts.prompts {
            out.push(Prompt {
                template: prompt.template.clone(),
                question: prompt.question.clone(),
                input_type: match prompt.input_type() {
                    ProtoType::String => InputType::Name,
                    ProtoType::Integer => InputType::Integer,
                    ProtoType::SignedInteger => InputType::SignedInteger,
                    ProtoType::Boolean => InputType::Boolean,
                },
            });
        }

        Ok(PromptCollection(out))
    }

    pub async fn set_responses(&mut self, name: &str, responses: PromptResponses) -> Result<()> {
        let mut out = ProtoPromptResponses {
            name: name.to_string(),
            responses: Default::default(),
        };

        for response in &responses.0 {
            out.responses.push(ProtoPromptResponse {
                template: response.template.clone(),
                response: response.input.to_string(),
            });
        }

        self.client.set_responses(Request::new(out)).await?;
        Ok(())
    }
}
