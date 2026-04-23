//! Integration tests for vector store service.

use std::time::Duration;

use openai::{
    files::{FileCreateParams, FilePurpose, FileUploadPart},
    vector_stores::{
        VectorStoreCreateParams, VectorStoreFileBatchCreateParams, VectorStoreFileCreateParams,
        VectorStoreUpdateParams,
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

fn vector_store_file_json(id: &str, status: &str) -> serde_json::Value {
    json!({
        "id": id,
        "object": "vector_store.file",
        "created_at": 1700000000,
        "status": status,
        "vector_store_id": "vs_1"
    })
}

fn vector_store_file_batch_json(id: &str, status: &str) -> serde_json::Value {
    json!({
        "id": id,
        "object": "vector_store.file_batch",
        "created_at": 1700000000,
        "status": status,
        "vector_store_id": "vs_1"
    })
}

fn file_object_json(id: &str) -> serde_json::Value {
    json!({
        "id": id,
        "object": "file",
        "bytes": 10,
        "created_at": 1700000000,
        "filename": "doc.txt",
        "purpose": "assistants",
        "status": "processed"
    })
}

#[tokio::test]
async fn vector_stores_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/vector_stores"))
        .and(body_partial_json(json!({"name":"docs-store"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"vs_1",
            "object":"vector_store",
            "name":"docs-store",
            "status":"in_progress"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/vector_stores/vs_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"vs_1",
            "object":"vector_store",
            "name":"docs-store",
            "status":"completed"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/vector_stores/vs_1"))
        .and(body_partial_json(json!({"name":"docs-store-v2"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"vs_1",
            "object":"vector_store",
            "name":"docs-store-v2",
            "status":"completed"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/vector_stores"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id":"vs_1",
                "object":"vector_store",
                "name":"docs-store-v2",
                "status":"completed"
            }],
            "has_more":false,
            "first_id":"vs_1",
            "last_id":"vs_1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/vector_stores/vs_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"vs_1",
            "object":"vector_store.deleted",
            "deleted":true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let vector_stores = client.vector_stores();

    let created = vector_stores
        .create(VectorStoreCreateParams {
            name: Some("docs-store".to_owned()),
            file_ids: None,
            metadata: None,
        })
        .await
        .expect("create vector store");
    assert_eq!(created.id, "vs_1");

    let got = vector_stores.get("vs_1").await.expect("get vector store");
    assert_eq!(got.status.as_deref(), Some("completed"));

    let updated = vector_stores
        .update(
            "vs_1",
            VectorStoreUpdateParams {
                name: Some("docs-store-v2".to_owned()),
                metadata: None,
            },
        )
        .await
        .expect("update vector store");
    assert_eq!(updated.name.as_deref(), Some("docs-store-v2"));

    let listed = vector_stores.list(None).await.expect("list vector stores");
    assert_eq!(listed.data.len(), 1);

    let deleted = vector_stores
        .delete("vs_1")
        .await
        .expect("delete vector store");
    assert!(deleted.deleted);
}

#[tokio::test]
async fn vector_store_file_create_and_poll_returns_terminal_file() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/vector_stores/vs_1/files"))
        .and(body_partial_json(json!({"file_id":"file_1"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(vector_store_file_json("file_1", "in_progress")),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/vector_stores/vs_1/files/file_1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(vector_store_file_json("file_1", "completed")),
        )
        .mount(&server)
        .await;

    let client = test_client(&server);
    let file = client
        .vector_stores()
        .files("vs_1")
        .create_and_poll(
            VectorStoreFileCreateParams {
                file_id: "file_1".to_owned(),
                attributes: None,
                chunking_strategy: None,
            },
            Duration::from_millis(0),
        )
        .await
        .expect("create and poll vector store file");

    assert_eq!(file.id, "file_1");
    assert_eq!(file.status, "completed");
}

#[tokio::test]
async fn vector_store_file_upload_and_poll_uploads_and_attaches_file() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(file_object_json("file_uploaded")))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/vector_stores/vs_1/files"))
        .and(body_partial_json(json!({"file_id":"file_uploaded"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(vector_store_file_json("file_uploaded", "in_progress")),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/vector_stores/vs_1/files/file_uploaded"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(vector_store_file_json("file_uploaded", "completed")),
        )
        .mount(&server)
        .await;

    let client = test_client(&server);
    let file = client
        .vector_stores()
        .files("vs_1")
        .upload_and_poll(
            FileCreateParams {
                purpose: FilePurpose::Assistants,
                file: FileUploadPart::from_bytes(b"hello".to_vec(), "doc.txt"),
                expires_after: None,
            },
            Duration::from_millis(0),
        )
        .await
        .expect("upload and poll vector store file");

    assert_eq!(file.id, "file_uploaded");
    assert_eq!(file.status, "completed");
}

#[tokio::test]
async fn vector_store_file_batch_create_and_poll_returns_terminal_batch() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/vector_stores/vs_1/file_batches"))
        .and(body_partial_json(json!({"file_ids":["file_1"]})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(vector_store_file_batch_json("batch_1", "in_progress")),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/vector_stores/vs_1/file_batches/batch_1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(vector_store_file_batch_json("batch_1", "completed")),
        )
        .mount(&server)
        .await;

    let client = test_client(&server);
    let batch = client
        .vector_stores()
        .file_batches("vs_1")
        .create_and_poll(
            VectorStoreFileBatchCreateParams {
                file_ids: Some(vec!["file_1".to_owned()]),
                attributes: None,
                chunking_strategy: None,
                files: None,
            },
            Duration::from_millis(0),
        )
        .await
        .expect("create and poll vector store file batch");

    assert_eq!(batch.id, "batch_1");
    assert_eq!(batch.status, "completed");
}

#[tokio::test]
async fn vector_store_file_batch_upload_and_poll_uploads_files_and_creates_batch() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(file_object_json("file_uploaded")))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/vector_stores/vs_1/file_batches"))
        .and(body_partial_json(
            json!({"file_ids":["file_existing","file_uploaded"]}),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(vector_store_file_batch_json(
                "batch_uploaded",
                "in_progress",
            )),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/vector_stores/vs_1/file_batches/batch_uploaded"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(vector_store_file_batch_json("batch_uploaded", "completed")),
        )
        .mount(&server)
        .await;

    let client = test_client(&server);
    let batch = client
        .vector_stores()
        .file_batches("vs_1")
        .upload_and_poll(
            vec![FileCreateParams {
                purpose: FilePurpose::Assistants,
                file: FileUploadPart::from_bytes(b"hello".to_vec(), "doc.txt"),
                expires_after: None,
            }],
            vec!["file_existing".to_owned()],
            Duration::from_millis(0),
        )
        .await
        .expect("upload and poll vector store file batch");

    assert_eq!(batch.id, "batch_uploaded");
    assert_eq!(batch.status, "completed");
}
