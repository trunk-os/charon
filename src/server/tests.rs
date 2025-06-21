use crate::{Client, Config, Server};
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_ping() {
    let tf = NamedTempFile::new().unwrap();
    let pb = tf.path().to_path_buf();
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

    let client = Client::new(pb).unwrap();
    client.status().await.unwrap().ping().await.unwrap();
}
