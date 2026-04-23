//! Integration tests for beta assistants/threads/runs baseline services.

use openai::{
    beta::{
        AssistantCreateParams, AssistantUpdateParams, ChatKitSessionCreateParams,
        ChatSessionWorkflowParam, RunCreateParams, RunStatus, ThreadCreateParams,
        ThreadUpdateParams,
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
async fn assistants_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/assistants"))
        .and(body_partial_json(json!({"model":"gpt-4o-mini"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"asst_1",
            "object":"assistant",
            "model":"gpt-4o-mini",
            "name":"Helper",
            "instructions":"Be brief"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/assistants/asst_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"asst_1",
            "object":"assistant",
            "model":"gpt-4o-mini",
            "name":"Helper",
            "instructions":"Be brief"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/assistants/asst_1"))
        .and(body_partial_json(json!({"instructions":"Be precise"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"asst_1",
            "object":"assistant",
            "model":"gpt-4o-mini",
            "name":"Helper",
            "instructions":"Be precise"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/assistants"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id":"asst_1",
                "object":"assistant",
                "model":"gpt-4o-mini",
                "name":"Helper",
                "instructions":"Be precise"
            }],
            "has_more":false,
            "first_id":"asst_1",
            "last_id":"asst_1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/assistants/asst_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"asst_1",
            "object":"assistant.deleted",
            "deleted":true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let assistants = client.beta().assistants();

    let created = assistants
        .create(AssistantCreateParams {
            model: "gpt-4o-mini".to_owned(),
            name: Some("Helper".to_owned()),
            description: None,
            instructions: Some("Be brief".to_owned()),
            tools: None,
            tool_resources: None,
            metadata: None,
            temperature: None,
            top_p: None,
            reasoning_effort: None,
            response_format: None,
        })
        .await
        .expect("create assistant");
    assert_eq!(created.id, "asst_1");

    let got = assistants.get("asst_1").await.expect("get assistant");
    assert_eq!(got.model, "gpt-4o-mini");

    let updated = assistants
        .update(
            "asst_1",
            AssistantUpdateParams {
                model: None,
                name: None,
                description: None,
                instructions: Some("Be precise".to_owned()),
                tools: None,
                tool_resources: None,
                metadata: None,
                temperature: None,
                top_p: None,
                reasoning_effort: None,
                response_format: None,
            },
        )
        .await
        .expect("update assistant");
    assert_eq!(updated.instructions.as_deref(), Some("Be precise"));

    let listed = assistants.list(None).await.expect("list assistants");
    assert_eq!(listed.data.len(), 1);

    let deleted = assistants.delete("asst_1").await.expect("delete assistant");
    assert!(deleted.deleted);
}

#[tokio::test]
async fn threads_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/threads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"thread_1",
            "object":"thread",
            "metadata":{"topic":"support"}
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/threads/thread_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"thread_1",
            "object":"thread",
            "metadata":{"topic":"support"}
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/threads/thread_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"thread_1",
            "object":"thread",
            "metadata":{"topic":"billing"}
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/threads/thread_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"thread_1",
            "object":"thread.deleted",
            "deleted":true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let threads = client.beta().threads();

    let created = threads
        .create(ThreadCreateParams {
            messages: None,
            tool_resources: None,
            metadata: Some(std::collections::HashMap::from([(
                "topic".to_owned(),
                "support".to_owned(),
            )])),
        })
        .await
        .expect("create thread");
    assert_eq!(created.id, "thread_1");

    let got = threads.get("thread_1").await.expect("get thread");
    assert_eq!(got.object, "thread");

    let updated = threads
        .update(
            "thread_1",
            ThreadUpdateParams {
                tool_resources: None,
                metadata: Some(std::collections::HashMap::from([(
                    "topic".to_owned(),
                    "billing".to_owned(),
                )])),
            },
        )
        .await
        .expect("update thread");
    assert_eq!(
        updated
            .metadata
            .as_ref()
            .and_then(|m| m.get("topic"))
            .map(String::as_str),
        Some("billing")
    );

    let deleted = threads.delete("thread_1").await.expect("delete thread");
    assert!(deleted.deleted);
}

#[tokio::test]
async fn thread_runs_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/threads/thread_1/runs"))
        .and(body_partial_json(json!({"assistant_id":"asst_1"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"run_1",
            "object":"thread.run",
            "thread_id":"thread_1",
            "assistant_id":"asst_1",
            "status":"queued"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/threads/thread_1/runs/run_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"run_1",
            "object":"thread.run",
            "thread_id":"thread_1",
            "assistant_id":"asst_1",
            "status":"in_progress"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/threads/thread_1/runs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id":"run_1",
                "object":"thread.run",
                "thread_id":"thread_1",
                "assistant_id":"asst_1",
                "status":"in_progress"
            }],
            "has_more":false,
            "first_id":"run_1",
            "last_id":"run_1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/threads/thread_1/runs/run_1/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"run_1",
            "object":"thread.run",
            "thread_id":"thread_1",
            "assistant_id":"asst_1",
            "status":"cancelled"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let runs = client.beta().threads().runs();

    let created = runs
        .create(
            "thread_1",
            RunCreateParams {
                assistant_id: "asst_1".to_owned(),
                model: None,
                instructions: None,
                additional_instructions: None,
                additional_messages: None,
                tools: None,
                stream: None,
                metadata: None,
                max_completion_tokens: None,
                max_prompt_tokens: None,
                temperature: None,
                top_p: None,
                parallel_tool_calls: None,
                truncation_strategy: None,
                response_format: None,
                tool_choice: None,
                reasoning_effort: None,
                include: None,
            },
        )
        .await
        .expect("create run");
    assert_eq!(created.id, "run_1");

    let got = runs.get("thread_1", "run_1").await.expect("get run");
    assert_eq!(got.status, RunStatus::InProgress);

    let listed = runs.list("thread_1", None).await.expect("list runs");
    assert_eq!(listed.data.len(), 1);

    let cancelled = runs.cancel("thread_1", "run_1").await.expect("cancel run");
    assert_eq!(cancelled.status, RunStatus::Cancelled);
}

#[tokio::test]
async fn chatkit_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chatkit/sessions"))
        .and(body_partial_json(json!({
            "user": "user_1",
            "workflow": {"id": "workflow_1"}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "session_1",
            "object": "chatkit.session",
            "client_secret": "ek_test",
            "status": "active",
            "user": "user_1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/chatkit/sessions/session_1/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "session_1",
            "object": "chatkit.session",
            "status": "cancelled",
            "user": "user_1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/chatkit/threads/thread_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "thread_1",
            "object": "chatkit.thread",
            "created_at": 1700000000,
            "status": {"type": "active"},
            "title": "Support",
            "user": "user_1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/chatkit/threads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [{
                "id": "thread_1",
                "object": "chatkit.thread",
                "status": {"type": "active"},
                "user": "user_1"
            }],
            "has_more": false,
            "first_id": "thread_1",
            "last_id": "thread_1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/chatkit/threads/thread_1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [{"type": "message", "role": "user", "content": "hello"}],
            "has_more": false,
            "first_id": "item_1",
            "last_id": "item_1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/chatkit/threads/thread_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "thread_1",
            "object": "chatkit.thread.deleted",
            "deleted": true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let chatkit = client.beta().chat_kit();

    let session = chatkit
        .sessions()
        .create(ChatKitSessionCreateParams {
            user: "user_1".to_owned(),
            workflow: ChatSessionWorkflowParam {
                id: "workflow_1".to_owned(),
                version: None,
                state_variables: None,
                tracing: None,
            },
            chatkit_configuration: None,
            expires_after: None,
            rate_limits: None,
        })
        .await
        .expect("create ChatKit session");
    assert_eq!(session.id, "session_1");

    let cancelled = chatkit
        .sessions()
        .cancel("session_1")
        .await
        .expect("cancel ChatKit session");
    assert_eq!(cancelled.id, "session_1");

    let thread = chatkit
        .threads()
        .get("thread_1")
        .await
        .expect("get ChatKit thread");
    assert_eq!(thread.id, "thread_1");

    let threads = chatkit
        .threads()
        .list(None)
        .await
        .expect("list ChatKit threads");
    assert_eq!(threads.data.len(), 1);

    let items = chatkit
        .threads()
        .list_items("thread_1", None)
        .await
        .expect("list ChatKit thread items");
    assert_eq!(items.data.len(), 1);

    let deleted = chatkit
        .threads()
        .delete("thread_1")
        .await
        .expect("delete ChatKit thread");
    assert!(deleted.deleted);
}

#[test]
fn graders_namespace_is_accessible() {
    let client = Client::with_api_key("test-key").expect("client init");
    let grader_models = client.graders().grader_models();
    let _ = grader_models.client_ref();
}
