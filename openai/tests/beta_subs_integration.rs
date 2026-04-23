//! Integration tests for beta sub-services: messages, run steps, streaming, tools.

use openai::{
    beta::{
        AssistantCreateParams, AssistantTool, CodeInterpreterResources, FileSearchResources,
        FunctionDefinition, MessageCreateParams, MessageRole, MessageUpdateParams, RunStatus,
        RunStepStatus, RunStepType, ThreadCreateAndRunParams, ThreadCreateParams, ToolResources,
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

// ============================================================================
// Thread Messages
// ============================================================================

#[tokio::test]
async fn messages_create() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/threads/thread_1/messages"))
        .and(body_partial_json(json!({"role":"user","content":"Hello"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"msg_1",
            "object":"thread.message",
            "thread_id":"thread_1",
            "role":"user",
            "content":[{"type":"text","text":{"value":"Hello"}}]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let messages = client.beta().threads().messages();

    let msg = messages
        .create(
            "thread_1",
            MessageCreateParams {
                role: MessageRole::User,
                content: "Hello".to_owned(),
                attachments: None,
                metadata: None,
            },
        )
        .await
        .expect("create message");
    assert_eq!(msg.id, "msg_1");
    assert_eq!(msg.role, MessageRole::User);
}

#[tokio::test]
async fn messages_get() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/threads/thread_1/messages/msg_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"msg_1",
            "object":"thread.message",
            "thread_id":"thread_1",
            "role":"user",
            "content":[{"type":"text","text":{"value":"Hello"}}]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let msg = client
        .beta()
        .threads()
        .messages()
        .get("thread_1", "msg_1")
        .await
        .expect("get message");
    assert_eq!(msg.id, "msg_1");
}

#[tokio::test]
async fn messages_update() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/threads/thread_1/messages/msg_1"))
        .and(body_partial_json(json!({"metadata":{"reviewed":"true"}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"msg_1",
            "object":"thread.message",
            "thread_id":"thread_1",
            "role":"user",
            "content":[{"type":"text","text":{"value":"Hello"}}],
            "metadata":{"reviewed":"true"}
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let msg = client
        .beta()
        .threads()
        .messages()
        .update(
            "thread_1",
            "msg_1",
            MessageUpdateParams {
                metadata: Some(std::collections::HashMap::from([(
                    "reviewed".to_owned(),
                    "true".to_owned(),
                )])),
            },
        )
        .await
        .expect("update message");
    assert_eq!(
        msg.metadata
            .as_ref()
            .and_then(|m| m.get("reviewed"))
            .map(String::as_str),
        Some("true")
    );
}

#[tokio::test]
async fn messages_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/threads/thread_1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id":"msg_1",
                "object":"thread.message",
                "thread_id":"thread_1",
                "role":"user",
                "content":[{"type":"text","text":{"value":"Hi"}}]
            },{
                "id":"msg_2",
                "object":"thread.message",
                "thread_id":"thread_1",
                "role":"assistant",
                "content":[{"type":"text","text":{"value":"Hello!"}}]
            }],
            "has_more":false,
            "first_id":"msg_1",
            "last_id":"msg_2"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let page = client
        .beta()
        .threads()
        .messages()
        .list("thread_1", None)
        .await
        .expect("list messages");
    assert_eq!(page.data.len(), 2);
    assert_eq!(page.data[0].id, "msg_1");
    assert_eq!(page.data[1].id, "msg_2");
}

#[tokio::test]
async fn messages_delete() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/threads/thread_1/messages/msg_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"msg_1",
            "object":"thread.message.deleted",
            "deleted":true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let deleted = client
        .beta()
        .threads()
        .messages()
        .delete("thread_1", "msg_1")
        .await
        .expect("delete message");
    assert!(deleted.deleted);
    assert_eq!(deleted.id, "msg_1");
}

// ============================================================================
// Run Steps
// ============================================================================

#[tokio::test]
async fn run_steps_get() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/threads/thread_1/runs/run_1/steps/step_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"step_1",
            "object":"thread.run.step",
            "thread_id":"thread_1",
            "run_id":"run_1",
            "assistant_id":"asst_1",
            "type":"message_creation",
            "status":"completed",
            "step_details":{
                "type":"message_creation",
                "message_creation":{"message_id":"msg_1"}
            }
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let step = client
        .beta()
        .threads()
        .runs()
        .steps()
        .get("thread_1", "run_1", "step_1")
        .await
        .expect("get run step");
    assert_eq!(step.id, "step_1");
    assert_eq!(step.step_type, RunStepType::MessageCreation);
    assert_eq!(step.status, RunStepStatus::Completed);
}

#[tokio::test]
async fn run_steps_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/threads/thread_1/runs/run_1/steps"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[
                {
                    "id":"step_1",
                    "object":"thread.run.step",
                    "thread_id":"thread_1",
                    "run_id":"run_1",
                    "assistant_id":"asst_1",
                    "type":"message_creation",
                    "status":"completed",
                    "step_details":{
                        "type":"message_creation",
                        "message_creation":{"message_id":"msg_1"}
                    }
                },
                {
                    "id":"step_2",
                    "object":"thread.run.step",
                    "thread_id":"thread_1",
                    "run_id":"run_1",
                    "assistant_id":"asst_1",
                    "type":"tool_calls",
                    "status":"completed",
                    "step_details":{
                        "type":"tool_calls",
                        "tool_calls":[{
                            "id":"call_1",
                            "type":"function",
                            "function":{
                                "name":"get_weather",
                                "arguments":"{}",
                                "output":"sunny"
                            }
                        }]
                    }
                }
            ],
            "has_more":false,
            "first_id":"step_1",
            "last_id":"step_2"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let page = client
        .beta()
        .threads()
        .runs()
        .steps()
        .list("thread_1", "run_1", None)
        .await
        .expect("list run steps");
    assert_eq!(page.data.len(), 2);
    assert_eq!(page.data[0].step_type, RunStepType::MessageCreation);
    assert_eq!(page.data[1].step_type, RunStepType::ToolCalls);
}

// ============================================================================
// Assistant with tools
// ============================================================================

#[tokio::test]
async fn assistant_with_tools() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/assistants"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"asst_tools",
            "object":"assistant",
            "model":"gpt-4o",
            "name":"ToolBot",
            "tools":[
                {"type":"code_interpreter"},
                {"type":"file_search"},
                {"type":"function","function":{"name":"greet","description":"Greets","parameters":{"type":"object"}}}
            ],
            "tool_resources":{
                "code_interpreter":{"file_ids":["file-1"]},
                "file_search":{"vector_store_ids":["vs-1"]}
            }
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let assistant = client
        .beta()
        .assistants()
        .create(AssistantCreateParams {
            model: "gpt-4o".to_owned(),
            name: Some("ToolBot".to_owned()),
            description: None,
            instructions: None,
            tools: Some(vec![
                AssistantTool::CodeInterpreter,
                AssistantTool::FileSearch { file_search: None },
                AssistantTool::Function {
                    function: FunctionDefinition {
                        name: "greet".to_owned(),
                        description: Some("Greets".to_owned()),
                        parameters: Some(json!({"type": "object"})),
                    },
                },
            ]),
            tool_resources: Some(ToolResources {
                code_interpreter: Some(CodeInterpreterResources {
                    file_ids: Some(vec!["file-1".to_owned()]),
                }),
                file_search: Some(FileSearchResources {
                    vector_store_ids: Some(vec!["vs-1".to_owned()]),
                }),
            }),
            metadata: None,
            temperature: None,
            top_p: None,
            reasoning_effort: None,
            response_format: None,
        })
        .await
        .expect("create assistant with tools");

    assert_eq!(assistant.id, "asst_tools");
    let tools = assistant.tools.expect("tools should be present");
    assert_eq!(tools.len(), 3);
    assert!(matches!(tools[0], AssistantTool::CodeInterpreter));

    let resources = assistant
        .tool_resources
        .expect("tool_resources should be present");
    assert_eq!(
        resources
            .code_interpreter
            .as_ref()
            .and_then(|ci| ci.file_ids.as_ref())
            .map(|ids| ids.len()),
        Some(1)
    );
}

// ============================================================================
// Submit tool outputs
// ============================================================================

#[tokio::test]
async fn submit_tool_outputs() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/threads/thread_1/runs/run_1/submit_tool_outputs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"run_1",
            "object":"thread.run",
            "thread_id":"thread_1",
            "assistant_id":"asst_1",
            "status":"queued"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let run = client
        .beta()
        .threads()
        .runs()
        .submit_tool_outputs(
            "thread_1",
            "run_1",
            openai::beta::SubmitToolOutputsParams {
                tool_outputs: vec![openai::beta::ToolOutput {
                    tool_call_id: "call_1".to_owned(),
                    output: "42".to_owned(),
                }],
                stream: None,
            },
        )
        .await
        .expect("submit tool outputs");
    assert_eq!(run.status, RunStatus::Queued);
}

// ============================================================================
// Create and run thread
// ============================================================================

#[tokio::test]
async fn create_and_run() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/threads/runs"))
        .and(body_partial_json(json!({"assistant_id":"asst_1"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"run_1",
            "object":"thread.run",
            "thread_id":"thread_new",
            "assistant_id":"asst_1",
            "status":"queued"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let run = client
        .beta()
        .threads()
        .create_and_run(ThreadCreateAndRunParams {
            assistant_id: "asst_1".to_owned(),
            thread: Some(ThreadCreateParams {
                messages: Some(vec![MessageCreateParams {
                    role: MessageRole::User,
                    content: "What is the weather?".to_owned(),
                    attachments: None,
                    metadata: None,
                }]),
                tool_resources: None,
                metadata: None,
            }),
            model: None,
            instructions: None,
            tools: None,
            stream: None,
            metadata: None,
            temperature: None,
            top_p: None,
            max_completion_tokens: None,
            max_prompt_tokens: None,
            response_format: None,
            tool_choice: None,
            truncation_strategy: None,
        })
        .await
        .expect("create_and_run");
    assert_eq!(run.id, "run_1");
    assert_eq!(run.thread_id, "thread_new");
}

// ============================================================================
// Service hierarchy access
// ============================================================================

#[test]
fn sub_service_access_compiles() {
    let client = Client::with_api_key("test-key").expect("client init");
    let _messages = client.beta().threads().messages();
    let _steps = client.beta().threads().runs().steps();
}
