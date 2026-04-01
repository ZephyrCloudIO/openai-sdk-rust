//! Integration tests for beta assistants/threads/runs baseline services.

use openai::{
    beta::{
        AssistantCreateParams, AssistantUpdateParams, RunCreateParams, ThreadCreateParams,
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
            instructions: Some("Be brief".to_owned()),
            metadata: None,
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
                name: None,
                instructions: Some("Be precise".to_owned()),
                metadata: None,
            },
        )
        .await
        .expect("update assistant");
    assert_eq!(updated.instructions.as_deref(), Some("Be precise"));

    let listed = assistants.list().await.expect("list assistants");
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
            },
        )
        .await
        .expect("create run");
    assert_eq!(created.id, "run_1");

    let got = runs.get("thread_1", "run_1").await.expect("get run");
    assert_eq!(got.status, "in_progress");

    let listed = runs.list("thread_1").await.expect("list runs");
    assert_eq!(listed.data.len(), 1);

    let cancelled = runs.cancel("thread_1", "run_1").await.expect("cancel run");
    assert_eq!(cancelled.status, "cancelled");
}

#[test]
fn graders_namespace_is_accessible() {
    let client = Client::with_api_key("test-key").expect("client init");
    let grader_models = client.graders().grader_models();
    let _ = grader_models.client_ref();
}
