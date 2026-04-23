//! Integration tests for service endpoints.

use openai::{
    chat::{
        ChatCompletionCreateParams, ChatCompletionMessageParam, ChatCompletionUserMessageContent,
    },
    completions::CompletionCreateParams,
    embeddings::EmbeddingCreateParams,
    moderations::{ModerationCreateParams, ModerationInput},
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
            messages: vec![ChatCompletionMessageParam::User {
                content: ChatCompletionUserMessageContent::Text("Hello".to_owned()),
                name: None,
            }],
            stream: None,
            temperature: None,
            frequency_penalty: None,
            presence_penalty: None,
            max_completion_tokens: None,
            max_tokens: None,
            n: None,
            top_p: None,
            seed: None,
            store: None,
            logprobs: None,
            top_logprobs: None,
            stop: None,
            response_format: None,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            user: None,
            service_tier: None,
            metadata: None,
            reasoning_effort: None,
            stream_options: None,
            audio: None,
            logit_bias: None,
            modalities: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            safety_identifier: None,
            verbosity: None,
            prediction: None,
            web_search_options: None,
            function_call: None,
            functions: None,
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
            prompt: openai::completions::CompletionPrompt::Single("Hello".to_owned()),
            max_tokens: Some(16),
            temperature: None,
            ..Default::default()
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
            user: None,
            encoding_format: None,
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
            "results":[{
                "flagged":false,
                "categories":{
                    "harassment":false,"harassment/threatening":false,
                    "hate":false,"hate/threatening":false,
                    "illicit":false,"illicit/violent":false,
                    "self-harm":false,"self-harm/instructions":false,"self-harm/intent":false,
                    "sexual":false,"sexual/minors":false,
                    "violence":false,"violence/graphic":false
                },
                "category_scores":{
                    "harassment":0.0,"harassment/threatening":0.0,
                    "hate":0.0,"hate/threatening":0.0,
                    "illicit":0.0,"illicit/violent":0.0,
                    "self-harm":0.0,"self-harm/instructions":0.0,"self-harm/intent":0.0,
                    "sexual":0.0,"sexual/minors":0.0,
                    "violence":0.0,"violence/graphic":0.0
                }
            }]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let response = client
        .moderations()
        .create(ModerationCreateParams {
            model: Some(ModelId::from("omni-moderation-latest")),
            input: ModerationInput::from("hello"),
        })
        .await
        .expect("moderation response");

    assert_eq!(response.id, "modr_123");
    assert_eq!(response.results.len(), 1);
}
