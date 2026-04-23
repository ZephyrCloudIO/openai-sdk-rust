//! Integration tests for the Responses API service.

use openai::{
    responses::{
        InputContent, InputItemListOrder, InputItemListParams, InputMessageRole,
        InputTokenCountParams, ResponseCompactParams, ResponseCompactPromptCacheRetention,
        ResponseCreateParams, ResponseInput, ResponseInputItem, ResponseOutputItem, ResponseStatus,
        ServiceTier, TextConfig, TextVerbosity, Tool, ToolChoice, ToolChoiceMode, Truncation,
    },
    shared::ModelId,
    Client, ClientConfig,
};
use serde_json::json;
use wiremock::{
    matchers::{body_partial_json, method, path, path_regex},
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

fn sample_response_json() -> serde_json::Value {
    json!({
        "id": "resp_abc123",
        "object": "response",
        "created_at": 1700000000.0,
        "status": "completed",
        "model": "gpt-4o",
        "output": [
            {
                "type": "message",
                "id": "msg_abc123",
                "role": "assistant",
                "content": [
                    {
                        "type": "output_text",
                        "text": "Hello! I can help with that.",
                        "annotations": []
                    }
                ],
                "status": "completed"
            }
        ],
        "usage": {
            "input_tokens": 10,
            "output_tokens": 8,
            "total_tokens": 18
        }
    })
}

#[tokio::test]
async fn responses_create_text_input() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(body_partial_json(json!({
            "model": "gpt-4o",
            "input": "Hello, how are you?"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_response_json()))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let response = client
        .responses()
        .create(ResponseCreateParams {
            model: ModelId::from("gpt-4o"),
            input: ResponseInput::Text("Hello, how are you?".to_owned()),
            background: None,
            context_management: None,
            conversation: None,
            include: None,
            instructions: None,
            max_output_tokens: None,
            max_tool_calls: None,
            metadata: None,
            parallel_tool_calls: None,
            previous_response_id: None,
            prompt: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            reasoning: None,
            safety_identifier: None,
            service_tier: None,
            store: None,
            stream: None,
            stream_options: None,
            temperature: None,
            text: None,
            tool_choice: None,
            tools: None,
            top_logprobs: None,
            top_p: None,
            truncation: None,
            user: None,
            container: None,
        })
        .await
        .expect("create response");

    assert_eq!(response.id, "resp_abc123");
    assert_eq!(response.status, ResponseStatus::Completed);
    assert_eq!(response.output_text(), "Hello! I can help with that.");
}

#[tokio::test]
async fn responses_create_with_items_input() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(body_partial_json(json!({"model": "gpt-4o"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_response_json()))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let response = client
        .responses()
        .create(ResponseCreateParams {
            model: ModelId::from("gpt-4o"),
            input: ResponseInput::Items(vec![ResponseInputItem::Message {
                role: InputMessageRole::User,
                content: InputContent::Text("What is Rust?".to_owned()),
                status: None,
            }]),
            background: None,
            context_management: None,
            conversation: None,
            include: None,
            instructions: Some("Be concise".to_owned()),
            max_output_tokens: None,
            max_tool_calls: None,
            metadata: None,
            parallel_tool_calls: None,
            previous_response_id: None,
            prompt: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            reasoning: None,
            safety_identifier: None,
            service_tier: None,
            store: None,
            stream: None,
            stream_options: None,
            temperature: Some(0.5),
            text: None,
            tool_choice: None,
            tools: None,
            top_logprobs: None,
            top_p: None,
            truncation: None,
            user: None,
            container: None,
        })
        .await
        .expect("create response");

    assert_eq!(response.id, "resp_abc123");
    assert_eq!(response.output.len(), 1);
}

#[tokio::test]
async fn responses_create_with_tools() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(body_partial_json(json!({
            "model": "gpt-4o",
            "tools": [{"type": "function", "name": "get_weather"}]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "resp_tool",
            "object": "response",
            "created_at": 1700000000.0,
            "status": "completed",
            "model": "gpt-4o",
            "output": [
                {
                    "type": "function_call",
                    "id": "fc_1",
                    "call_id": "call_abc",
                    "name": "get_weather",
                    "arguments": "{\"location\":\"London\"}",
                    "status": "completed"
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let response = client
        .responses()
        .create(ResponseCreateParams {
            model: ModelId::from("gpt-4o"),
            input: ResponseInput::Text("What's the weather in London?".to_owned()),
            background: None,
            context_management: None,
            conversation: None,
            include: None,
            instructions: None,
            max_output_tokens: None,
            max_tool_calls: None,
            metadata: None,
            parallel_tool_calls: None,
            previous_response_id: None,
            prompt: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            reasoning: None,
            safety_identifier: None,
            service_tier: None,
            store: None,
            stream: None,
            stream_options: None,
            temperature: None,
            text: None,
            tool_choice: None,
            tools: Some(vec![Tool::Function {
                name: "get_weather".to_owned(),
                description: Some("Get weather for a location".to_owned()),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    },
                    "required": ["location"]
                })),
                strict: Some(true),
            }]),
            top_logprobs: None,
            top_p: None,
            truncation: None,
            user: None,
            container: None,
        })
        .await
        .expect("create response");

    assert_eq!(response.id, "resp_tool");
    match &response.output[0] {
        ResponseOutputItem::FunctionCall {
            name, arguments, ..
        } => {
            assert_eq!(name, "get_weather");
            assert!(arguments.contains("London"));
        }
        other => panic!("expected FunctionCall output, got {:?}", other),
    }
}

#[tokio::test]
async fn responses_get() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/responses/resp_abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_response_json()))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let response = client
        .responses()
        .get("resp_abc123", None)
        .await
        .expect("get response");

    assert_eq!(response.id, "resp_abc123");
    assert_eq!(response.status, ResponseStatus::Completed);
}

#[tokio::test]
async fn responses_delete() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/responses/resp_abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "resp_abc123",
            "object": "response",
            "deleted": true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let deleted = client
        .responses()
        .delete("resp_abc123")
        .await
        .expect("delete response");

    assert_eq!(deleted.id, "resp_abc123");
    assert!(deleted.deleted);
}

#[tokio::test]
async fn responses_cancel() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/responses/resp_bg123/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "resp_bg123",
            "object": "response",
            "created_at": 1700000000.0,
            "status": "cancelled",
            "model": "gpt-4o",
            "output": []
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let response = client
        .responses()
        .cancel("resp_bg123")
        .await
        .expect("cancel response");

    assert_eq!(response.id, "resp_bg123");
    assert_eq!(response.status, ResponseStatus::Cancelled);
}

#[tokio::test]
async fn responses_list_input_items() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex(r"/responses/resp_abc123/input_items.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [
                {
                    "type": "message",
                    "role": "user",
                    "content": "Hello"
                }
            ],
            "has_more": false,
            "first_id": "item_1",
            "last_id": "item_1"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let page = client
        .responses()
        .list_input_items("resp_abc123", None)
        .await
        .expect("list input items");

    assert!(!page.data.is_empty());
    assert!(!page.has_more);
}

#[tokio::test]
async fn responses_list_input_items_with_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex(r"/responses/resp_xyz/input_items.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [],
            "has_more": false,
            "first_id": null,
            "last_id": null
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let page = client
        .responses()
        .list_input_items(
            "resp_xyz",
            Some(InputItemListParams {
                after: Some("item_1".to_owned()),
                limit: Some(10),
                include: None,
                order: Some(InputItemListOrder::Asc),
            }),
        )
        .await
        .expect("list input items with params");

    assert!(page.data.is_empty());
}

#[tokio::test]
async fn responses_input_items_service() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex(r"/responses/resp_svc/input_items.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [
                {
                    "type": "message",
                    "role": "user",
                    "content": "test message"
                }
            ],
            "has_more": true,
            "first_id": "item_a",
            "last_id": "item_b"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let page = client
        .responses()
        .input_items()
        .list("resp_svc", None)
        .await
        .expect("input items service list");

    assert_eq!(page.data.len(), 1);
    assert!(page.has_more);
}

#[tokio::test]
async fn responses_input_tokens_count() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/responses/input_tokens"))
        .and(body_partial_json(json!({
            "model": "gpt-4o",
            "input": "Count me"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "response.input_tokens",
            "input_tokens": 3
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let count = client
        .responses()
        .input_tokens()
        .count(InputTokenCountParams {
            model: Some(ModelId::from("gpt-4o")),
            input: Some(ResponseInput::Text("Count me".to_owned())),
            instructions: None,
            parallel_tool_calls: None,
            previous_response_id: None,
            conversation: None,
            text: None,
            tool_choice: None,
            tools: None,
            reasoning: None,
            truncation: None,
        })
        .await
        .expect("count input tokens");

    assert_eq!(count.input_tokens, 3);
    assert_eq!(count.object, "response.input_tokens");
}

#[tokio::test]
async fn responses_compact_sends_prompt_cache_retention() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/responses/compact"))
        .and(body_partial_json(json!({
            "model": "gpt-4o",
            "previous_response_id": "resp_previous",
            "prompt_cache_retention": "in_memory"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "resp_compact",
            "object": "response.compaction",
            "created_at": 1700000000.0,
            "output": [],
            "usage": {
                "input_tokens": 10,
                "output_tokens": 2,
                "total_tokens": 12
            }
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let compacted = client
        .responses()
        .compact(ResponseCompactParams {
            model: ModelId::from("gpt-4o"),
            instructions: None,
            previous_response_id: Some("resp_previous".to_owned()),
            prompt_cache_key: None,
            input: None,
            prompt_cache_retention: Some(ResponseCompactPromptCacheRetention::InMemory),
        })
        .await
        .expect("compact response");

    assert_eq!(compacted.id, "resp_compact");
    assert_eq!(compacted.usage.total_tokens, 12);
}

#[tokio::test]
async fn responses_create_with_all_options() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(body_partial_json(json!({
            "model": "gpt-4o",
            "temperature": 0.7,
            "truncation": "auto",
            "service_tier": "auto",
            "store": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_response_json()))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let response = client
        .responses()
        .create(ResponseCreateParams {
            model: ModelId::from("gpt-4o"),
            input: ResponseInput::Text("Complex query".to_owned()),
            background: None,
            context_management: None,
            conversation: None,
            include: None,
            instructions: Some("Be thorough.".to_owned()),
            max_output_tokens: Some(4096),
            max_tool_calls: None,
            metadata: None,
            parallel_tool_calls: None,
            previous_response_id: None,
            prompt: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            reasoning: None,
            safety_identifier: None,
            service_tier: Some(ServiceTier::Auto),
            store: Some(true),
            stream: None,
            stream_options: None,
            temperature: Some(0.7),
            text: Some(TextConfig {
                format: None,
                verbosity: Some(TextVerbosity::Medium),
            }),
            tool_choice: Some(ToolChoice::Mode(ToolChoiceMode::Auto)),
            tools: None,
            top_logprobs: None,
            top_p: None,
            truncation: Some(Truncation::Auto),
            user: None,
            container: None,
        })
        .await
        .expect("create response with options");

    assert_eq!(response.id, "resp_abc123");
}
