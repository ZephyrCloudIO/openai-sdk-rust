//! Integration tests for containers service.

use openai::{
    containers::{
        ContainerCreateParams, ContainerExpiresAfter, ContainerFileCreateParams,
        ContainerFileListParams, ContainerListParams,
    },
    Client, ClientConfig,
};
use serde_json::json;
use wiremock::{
    matchers::{body_partial_json, method, path},
    Mock, MockServer, ResponseTemplate,
};

fn test_client(server: &MockServer) -> Client {
    Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri())
            .with_max_retries(0),
    )
    .expect("client init")
}

#[tokio::test]
async fn containers_crud_round_trip() {
    let server = MockServer::start().await;

    // Create
    Mock::given(method("POST"))
        .and(path("/containers"))
        .and(body_partial_json(json!({"name": "my-container"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "ctr_1",
            "object": "container",
            "name": "my-container",
            "status": "active",
            "created_at": 1234567890,
            "memory_limit": "1g"
        })))
        .mount(&server)
        .await;

    // Get
    Mock::given(method("GET"))
        .and(path("/containers/ctr_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "ctr_1",
            "object": "container",
            "name": "my-container",
            "status": "active",
            "created_at": 1234567890,
            "memory_limit": "1g"
        })))
        .mount(&server)
        .await;

    // List
    Mock::given(method("GET"))
        .and(path("/containers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [{
                "id": "ctr_1",
                "object": "container",
                "name": "my-container",
                "status": "active",
                "created_at": 1234567890,
                "memory_limit": "1g"
            }],
            "has_more": false,
            "first_id": "ctr_1",
            "last_id": "ctr_1"
        })))
        .mount(&server)
        .await;

    // Delete
    Mock::given(method("DELETE"))
        .and(path("/containers/ctr_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "ctr_1",
            "object": "container.deleted",
            "deleted": true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let containers = client.containers();

    // Create container
    let created = containers
        .create(ContainerCreateParams {
            name: "my-container".to_owned(),
            expires_after: Some(ContainerExpiresAfter {
                anchor: "last_active_at".to_owned(),
                minutes: 60,
            }),
            file_ids: None,
            memory_limit: Some("1g".to_owned()),
            network_policy: None,
            skills: None,
        })
        .await
        .expect("create container");
    assert_eq!(created.id, "ctr_1");
    assert_eq!(created.name, "my-container");
    assert_eq!(created.status, "active");

    // Get container
    let got = containers.get("ctr_1").await.expect("get container");
    assert_eq!(got.id, "ctr_1");

    // List containers
    let listed = containers
        .list(ContainerListParams::default())
        .await
        .expect("list containers");
    assert_eq!(listed.data.len(), 1);
    assert!(!listed.has_more);

    // Delete container
    let deleted = containers.delete("ctr_1").await.expect("delete container");
    assert!(deleted.deleted);
}

#[tokio::test]
async fn container_files_crud_round_trip() {
    let server = MockServer::start().await;

    // Create file
    Mock::given(method("POST"))
        .and(path("/containers/ctr_1/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "file_1",
            "object": "container.file",
            "bytes": 1024,
            "container_id": "ctr_1",
            "created_at": 1234567890,
            "path": "/data/input.txt",
            "source": "user"
        })))
        .mount(&server)
        .await;

    // Get file
    Mock::given(method("GET"))
        .and(path("/containers/ctr_1/files/file_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "file_1",
            "object": "container.file",
            "bytes": 1024,
            "container_id": "ctr_1",
            "created_at": 1234567890,
            "path": "/data/input.txt",
            "source": "user"
        })))
        .mount(&server)
        .await;

    // List files
    Mock::given(method("GET"))
        .and(path("/containers/ctr_1/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [{
                "id": "file_1",
                "object": "container.file",
                "bytes": 1024,
                "container_id": "ctr_1",
                "created_at": 1234567890,
                "path": "/data/input.txt",
                "source": "user"
            }],
            "has_more": false,
            "first_id": "file_1",
            "last_id": "file_1"
        })))
        .mount(&server)
        .await;

    // Delete file
    Mock::given(method("DELETE"))
        .and(path("/containers/ctr_1/files/file_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "file_1",
            "object": "container.file.deleted",
            "deleted": true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let files = client.containers().files();

    // Create file
    let created = files
        .create(
            "ctr_1",
            ContainerFileCreateParams {
                file_id: Some("file_abc".to_owned()),
            },
        )
        .await
        .expect("create container file");
    assert_eq!(created.id, "file_1");
    assert_eq!(created.container_id, "ctr_1");
    assert_eq!(created.path, "/data/input.txt");

    // Get file
    let got = files
        .get("ctr_1", "file_1")
        .await
        .expect("get container file");
    assert_eq!(got.id, "file_1");
    assert_eq!(got.source, "user");

    // List files
    let listed = files
        .list("ctr_1", ContainerFileListParams::default())
        .await
        .expect("list container files");
    assert_eq!(listed.data.len(), 1);

    // Delete file
    let deleted = files
        .delete("ctr_1", "file_1")
        .await
        .expect("delete container file");
    assert!(deleted.deleted);
}

#[tokio::test]
async fn container_file_content_get() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/containers/ctr_1/files/file_1/content"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"file contents here"))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let content_service = client.containers().files().content();

    let bytes = content_service
        .get("ctr_1", "file_1")
        .await
        .expect("get file content");
    assert_eq!(bytes, b"file contents here");
}
