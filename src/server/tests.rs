use crate::{Client, Config, Server};
use std::path::PathBuf;
use tempfile::NamedTempFile;

async fn start_server() -> PathBuf {
    let tf = NamedTempFile::new().unwrap();
    let (_, path) = tf.keep().unwrap();
    let pb = path.to_path_buf();
    let pb2 = pb.clone();
    tokio::spawn(async move {
        Server::new(Config {
            socket: pb2,
            log_level: None,
            debug: Some(true),
            registry: "testdata/registry".into(),
        })
        .start()
        .unwrap()
        .await
        .unwrap()
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    path
}

#[tokio::test]
async fn test_ping() {
    let client = Client::new(start_server().await.to_path_buf()).unwrap();
    client.status().await.unwrap().ping().await.unwrap();
}

#[tokio::test]
async fn test_write_unit() {
    let client = Client::new(start_server().await.to_path_buf()).unwrap();
    client
        .control()
        .await
        .unwrap()
        .write_unit("podman-test", "0.0.2", "/tmp/volroot".into())
        .await
        .unwrap()
}
