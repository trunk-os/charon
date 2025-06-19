use crate::status_server::{Status, StatusServer};
use crate::Config;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use tonic::{body::Body, transport::Server as TransportServer, Result};
use tonic_middleware::{Middleware, MiddlewareLayer, ServiceBound};
use tracing::{error, info};

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
            .serve_with_incoming(uds_stream))
    }
}

#[tonic::async_trait]
impl Status for Server {
    async fn ping(&self, _: tonic::Request<()>) -> Result<tonic::Response<()>> {
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
