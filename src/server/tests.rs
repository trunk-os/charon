use crate::{
    Client, Config, Input, InputType, Prompt, PromptCollection, PromptResponse, PromptResponses,
    Server,
};
use std::path::PathBuf;
use tempfile::{tempdir, NamedTempFile};

async fn start_server(debug: bool) -> (PathBuf, Option<PathBuf>) {
    let tf = NamedTempFile::new().unwrap();
    let (_, path) = tf.keep().unwrap();
    let pb = path.to_path_buf();
    let pb2 = pb.clone();

    let systemd_root = if debug {
        None
    } else {
        let tf = tempdir().unwrap();
        Some(tf.keep())
    };

    let inner = systemd_root.clone();
    tokio::spawn(async move {
        Server::new(Config {
            socket: pb2,
            log_level: None,
            debug: Some(debug),
            registry: "testdata/registry".into(),
            systemd_root: inner,
        })
        .start()
        .unwrap()
        .await
        .unwrap()
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    (path, systemd_root)
}

#[tokio::test]
async fn test_ping() {
    let client = Client::new(start_server(true).await.0.to_path_buf()).unwrap();
    client.status().await.unwrap().ping().await.unwrap();
}

#[cfg(feature = "livetests")]
#[tokio::test]
async fn test_write_unit_real() {
    // real mode. validate written. this test also reloads systemd (which doesn't pick up anything
    // new because of the temporary path to write to) so it needs to be run as root.
    let (socket, systemd_path) = start_server(false).await;
    let client = Client::new(socket).unwrap();

    assert!(client
        .control()
        .await
        .unwrap()
        .remove_unit("podman-test", "0.0.2")
        .await
        .is_err());

    client
        .control()
        .await
        .unwrap()
        .write_unit("podman-test", "0.0.2", "/tmp/volroot".into())
        .await
        .unwrap();

    let systemd_path = systemd_path.unwrap();

    let content = std::fs::read_to_string(systemd_path.join("podman-test-0.0.2.service")).unwrap();
    assert_eq!(
        content,
        format!(
            r#"
[Unit]
Description=Charon launcher for podman-test, version 0.0.2
After= # FIXME: add dependencies
After= # FIXME: this needs to follow the trunk microservices boot

[Service]
ExecStart=/usr/bin/charon -r testdata/registry launch podman-test 0.0.2 /tmp/volroot
ExecStop=/usr/bin/charon -r testdata/registry stop podman-test 0.0.2 /tmp/volroot
Restart=always

[Install]
Alias=podman-test-0.0.2.service
"#
        )
    );

    assert!(client
        .control()
        .await
        .unwrap()
        .remove_unit("podman-test", "0.0.2")
        .await
        .is_ok());

    assert!(!std::fs::exists(systemd_path.join("podman-test-0.0.2.service")).unwrap())
}

#[tokio::test]
async fn test_write_unit_debug() {
    // debug mode
    let client = Client::new(start_server(true).await.0.to_path_buf()).unwrap();
    client
        .control()
        .await
        .unwrap()
        .write_unit("podman-test", "0.0.2", "/tmp/volroot".into())
        .await
        .unwrap();
}

#[tokio::test]
async fn test_get_prompts() {
    let client = Client::new(start_server(true).await.0.to_path_buf()).unwrap();
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

    let client = Client::new(start_server(true).await.0.to_path_buf()).unwrap();
    client
        .query()
        .await
        .unwrap()
        .set_responses("with-prompts", responses)
        .await
        .unwrap();
}
