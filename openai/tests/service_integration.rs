//! Integration tests for service endpoints.

use openai::{
    chat::{ChatCompletionCreateParams, ChatMessageParam, ChatRole},
    completions::CompletionCreateParams,
    embeddings::EmbeddingCreateParams,
    moderations::ModerationCreateParams,
    param::OneOrMany,
    shared::ModelId,
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
async fn chat_create_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(body_partial_json(json!({"model":"gpt-4o-mini"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"chatcmpl_123",
            "object":"chat.completion",
            "model":"gpt-4o-mini",
            "choices":[{
                "index":0,
                "message":{"role":"assistant","content":"Hi"},
                "finish_reason":"stop"
            }]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let response = client
        .chat()
        .completions()
        .create(ChatCompletionCreateParams {
            model: ModelId::from("gpt-4o-mini"),
            messages: vec![ChatMessageParam {
                role: ChatRole::User,
                content: "Hello".to_owned(),
            }],
            stream: None,
            temperature: None,
        })
        .await
        .expect("chat completion");

    assert_eq!(response.id, "chatcmpl_123");
    assert_eq!(response.choices.len(), 1);
}

#[tokio::test]
async fn completions_create_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/completions"))
        .and(body_partial_json(json!({"model":"gpt-3.5-turbo-instruct"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"cmpl_123",
            "object":"text_completion",
            "choices":[{"index":0,"text":"done","finish_reason":"stop"}]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let response = client
        .completions()
        .create(CompletionCreateParams {
            model: ModelId::from("gpt-3.5-turbo-instruct"),
            prompt: "Hello".to_owned(),
            max_tokens: Some(16),
            temperature: None,
        })
        .await
        .expect("completion");

    assert_eq!(response.id, "cmpl_123");
    assert_eq!(response.choices[0].text, "done");
}

#[tokio::test]
async fn embeddings_create_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/embeddings"))
        .and(body_partial_json(json!({"model":"text-embedding-3-small"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{"object":"embedding","index":0,"embedding":[0.1,0.2]}],
            "model":"text-embedding-3-small"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let response = client
        .embeddings()
        .create(EmbeddingCreateParams {
            model: ModelId::from("text-embedding-3-small"),
            input: OneOrMany::One("hello".to_owned()),
            dimensions: None,
        })
        .await
        .expect("embedding response");

    assert_eq!(response.data.len(), 1);
    assert_eq!(
        response.model.as_ref().map(ModelId::as_ref),
        Some("text-embedding-3-small")
    );
}

#[tokio::test]
async fn models_get_and_delete_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models/gpt-4o-mini"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"gpt-4o-mini",
            "object":"model",
            "owned_by":"openai"
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/models/ft%3Agpt-4o%3Acustom"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"ft:gpt-4o:custom",
            "object":"model",
            "deleted":true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);

    let model = client.models().get("gpt-4o-mini").await.expect("model get");
    assert_eq!(model.id.as_ref(), "gpt-4o-mini");

    let deleted = client
        .models()
        .delete("ft:gpt-4o:custom")
        .await
        .expect("model delete");
    assert!(deleted.deleted);
}

#[tokio::test]
async fn moderations_create_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/moderations"))
        .and(body_partial_json(json!({"model":"omni-moderation-latest"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"modr_123",
            "model":"omni-moderation-latest",
            "results":[{"flagged":false}]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let response = client
        .moderations()
        .create(ModerationCreateParams {
            model: Some(ModelId::from("omni-moderation-latest")),
            input: OneOrMany::One("hello".to_owned()),
        })
        .await
        .expect("moderation response");

    assert_eq!(response.id, "modr_123");
    assert_eq!(response.results.len(), 1);
}
