use crate::{
    Client, Config, Input, InputType, Prompt, PromptCollection, PromptResponse, PromptResponses,
    Server,
};
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

#[tokio::test]
async fn test_get_prompts() {
    let client = Client::new(start_server().await.to_path_buf()).unwrap();
    let prompts = client
        .query()
        .await
        .unwrap()
        .get_prompts("podman-test", "0.0.2")
        .await
        .unwrap();

    assert!(prompts.0.is_empty());

    let prompts = client
        .query()
        .await
        .unwrap()
        .get_prompts("with-prompts", "0.0.1")
        .await
        .unwrap();

    assert!(!prompts.0.is_empty());

    let equal = PromptCollection(vec![
        Prompt {
            template: "private_path".into(),
            question: "Where do you want this mounted?".into(),
            input_type: InputType::Name,
        },
        Prompt {
            template: "private_size".into(),
            question: "How big should it be?".into(),
            input_type: InputType::Integer,
        },
        Prompt {
            template: "private_recreate".into(),
            question: "Should we recreate this volume if it already exists?".into(),
            input_type: InputType::Boolean,
        },
    ]);

    assert_eq!(prompts, equal);
}

#[tokio::test]
async fn test_set_responses() {
    let responses = PromptResponses(vec![
        PromptResponse {
            input: Input::String("/tmp/volroot".into()),
            template: "private_path".into(),
        },
        PromptResponse {
            input: Input::Integer(8675309),
            template: "private_size".into(),
        },
        PromptResponse {
            input: Input::Boolean(false),
            template: "private_recreate".into(),
        },
    ]);

    let client = Client::new(start_server().await.to_path_buf()).unwrap();
    client
        .query()
        .await
        .unwrap()
        .set_responses("with-prompts", responses)
        .await
        .unwrap();
}
