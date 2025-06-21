use crate::{
    control_server::{Control, ControlServer},
    query_server::{Query, QueryServer},
    reload_systemd,
    status_server::{Status, StatusServer},
    Config, Input, InputType, PromptResponse, PromptResponses, ProtoPackageTitle,
    ProtoPackageTitleWithRoot, ProtoPrompt, ProtoPromptResponses, ProtoPrompts, ProtoType,
    SystemdUnit,
};
use std::os::unix::fs::PermissionsExt;
use std::{fs::Permissions, io::Write};
use tonic::{body::Body, transport::Server as TransportServer, Result};
use tonic_middleware::{Middleware, MiddlewareLayer, ServiceBound};
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct Server {
    config: Config,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn start(
        &self,
    ) -> anyhow::Result<impl std::future::Future<Output = Result<(), tonic::transport::Error>>>
    {
        info!("Starting service.");

        if let Some(parent) = self.config.socket.to_path_buf().parent() {
            std::fs::create_dir_all(&parent)?;
        }

        if std::fs::exists(&self.config.socket)? {
            std::fs::remove_file(&self.config.socket)?;
        }

        let uds = tokio::net::UnixListener::bind(&self.config.socket)?;
        let uds_stream = tokio_stream::wrappers::UnixListenerStream::new(uds);

        std::fs::set_permissions(&self.config.socket, Permissions::from_mode(0o600))?;

        Ok(TransportServer::builder()
            .layer(MiddlewareLayer::new(LogMiddleware))
            .add_service(StatusServer::new(self.clone()))
            .add_service(ControlServer::new(self.clone()))
            .add_service(QueryServer::new(self.clone()))
            .serve_with_incoming(uds_stream))
    }
}

#[tonic::async_trait]
impl Status for Server {
    async fn ping(&self, _: tonic::Request<()>) -> Result<tonic::Response<()>> {
        Ok(tonic::Response::new(()))
    }
}

#[tonic::async_trait]
impl Control for Server {
    async fn write_unit(
        &self,
        title: tonic::Request<ProtoPackageTitleWithRoot>,
    ) -> Result<tonic::Response<()>> {
        let r = self.config.registry();
        let title = title.into_inner();

        let pkg = r
            .load(&title.name, &title.version)
            .map_err(|e| tonic::Status::new(tonic::Code::Internal, e.to_string()))?
            .compile()
            .map_err(|e| tonic::Status::new(tonic::Code::Internal, e.to_string()))?;

        let unit = SystemdUnit::new(pkg);
        if !self.config.debug() {
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&unit.filename())
                .map_err(|e| tonic::Status::new(tonic::Code::Internal, e.to_string()))?;
            f.write_all(
                unit.unit(r.path(), title.volume_root.into())
                    .map_err(|e| tonic::Status::new(tonic::Code::Internal, e.to_string()))?
                    .as_bytes(),
            )
            .map_err(|e| tonic::Status::new(tonic::Code::Internal, e.to_string()))?;

            reload_systemd()
                .await
                .map_err(|e| tonic::Status::new(tonic::Code::Internal, e.to_string()))?;

            info!("Wrote unit to {}", unit.filename().display());
        } else {
            warn!("debug mode in effect; not writing unit file");
        }

        Ok(tonic::Response::new(()))
    }
}

#[tonic::async_trait]
impl Query for Server {
    async fn get_prompts(
        &self,
        title: tonic::Request<ProtoPackageTitle>,
    ) -> Result<tonic::Response<ProtoPrompts>> {
        let r = self.config.registry();
        let title = title.into_inner();
        let pkg = r
            .load(&title.name, &title.version)
            .map_err(|e| tonic::Status::new(tonic::Code::Internal, e.to_string()))?;
        let prompts = pkg.prompts.unwrap_or_default();

        let mut out = ProtoPrompts::default();

        for prompt in &prompts.to_vec() {
            // FIXME: do a From trait
            out.prompts.push(ProtoPrompt {
                template: prompt.template.clone(),
                question: prompt.question.clone(),
                input_type: match prompt.input_type {
                    InputType::Name | InputType::Path => ProtoType::String,
                    InputType::Integer => ProtoType::Integer,
                    InputType::SignedInteger => ProtoType::SignedInteger,
                    InputType::Boolean => ProtoType::Boolean,
                    _ => {
                        return Err(tonic::Status::new(
                            tonic::Code::Internal,
                            "Unsupported input type in prompts".to_string(),
                        ))
                    }
                }
                .into(),
            })
        }

        Ok(tonic::Response::new(out))
    }

    async fn set_responses(
        &self,
        responses: tonic::Request<ProtoPromptResponses>,
    ) -> Result<tonic::Response<()>> {
        let r = self.config.registry();
        let responses = responses.into_inner();

        let mut pr = Vec::new();
        for response in &responses.responses {
            pr.push(PromptResponse {
                template: response.template.clone(),
                input: Input::String(response.response.clone()),
            });
        }

        r.response_registry()
            .set(&responses.name, &PromptResponses(pr))
            .map_err(|e| tonic::Status::new(tonic::Code::Internal, e.to_string()))?;

        Ok(tonic::Response::new(()))
    }
}

#[derive(Default, Clone)]
pub struct LogMiddleware;

#[tonic::async_trait]
impl<S> Middleware<S> for LogMiddleware
where
    S: ServiceBound,
    S::Future: Send,
    S::Error: ToString,
{
    async fn call(
        &self,
        req: http::Request<Body>,
        mut service: S,
    ) -> Result<http::Response<Body>, S::Error> {
        let uri = req.uri().clone();
        info!("GRPC Request to {}", uri.path());

        match service.call(req).await {
            Ok(x) => Ok(x),
            Err(e) => {
                error!("Error during request to {}: {}", uri.path(), e.to_string());
                Err(e)
            }
        }
    }
}
