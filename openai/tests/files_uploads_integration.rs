//! Integration tests for files and uploads services.

use openai::{Client, ClientConfig};
use serde_json::json;
use wiremock::{
    matchers::{method, path},
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
async fn files_list_get_delete_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id":"file_1",
                "object":"file",
                "bytes":5,
                "created_at":123,
                "filename":"input.txt",
                "purpose":"assistants"
            }]
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/files/file_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"file_1",
            "object":"file",
            "bytes":5,
            "created_at":123,
            "filename":"input.txt",
            "purpose":"assistants"
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/files/file_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"file_1",
            "object":"file",
            "deleted":true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);

    let files = client.files().list(None).await.expect("files list");
    assert_eq!(files.data.len(), 1);
    assert_eq!(files.data[0].id, "file_1");

    let file = client.files().get("file_1").await.expect("file get");
    assert_eq!(file.filename, "input.txt");

    let deleted = client.files().delete("file_1").await.expect("file delete");
    assert!(deleted.deleted);
}

#[tokio::test]
async fn uploads_get_and_cancel_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/uploads/upload_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"upload_1",
            "object":"upload",
            "bytes":5,
            "filename":"input.bin",
            "purpose":"assistants",
            "status":"pending"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/uploads/upload_1/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"upload_1",
            "object":"upload",
            "bytes":5,
            "filename":"input.bin",
            "purpose":"assistants",
            "status":"cancelled"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);

    let upload = client.uploads().get("upload_1").await.expect("upload get");
    assert_eq!(upload.status, Some(openai::uploads::UploadStatus::Pending));

    let cancelled = client
        .uploads()
        .cancel("upload_1")
        .await
        .expect("upload cancel");
    assert_eq!(
        cancelled.status,
        Some(openai::uploads::UploadStatus::Cancelled)
    );
}
