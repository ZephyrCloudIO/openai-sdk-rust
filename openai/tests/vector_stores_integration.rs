//! Integration tests for vector store service.

use openai::{
    vector_stores::{VectorStoreCreateParams, VectorStoreUpdateParams},
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

    let listed = vector_stores.list().await.expect("list vector stores");
    assert_eq!(listed.data.len(), 1);

    let deleted = vector_stores
        .delete("vs_1")
        .await
        .expect("delete vector store");
    assert!(deleted.deleted);
}
