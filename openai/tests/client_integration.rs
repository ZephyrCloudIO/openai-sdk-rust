//! Integration tests for the core client behavior.

use openai::{
    audio::{
        AudioInputFile, AudioSpeechCreateParams, AudioSpeechResponseFormat, AudioSpeechVoice,
        AudioSpeechVoiceParam, AudioTranscriptionCreateParams,
    },
    auth::{AuthError, SubjectTokenProvider, SubjectTokenType, WorkloadIdentity},
    chat::{
        ChatCompletionCreateParams, ChatCompletionMessageParam, ChatCompletionUserMessageContent,
    },
    completions::CompletionCreateParams,
    embeddings::EmbeddingCreateParams,
    files::{FileCreateParams, FileListParams, FilePurpose, FileUploadPart},
    images::{
        ImageEditParams, ImageGenerateParams, ImageInputFile, ImageResponseFormat, ImageSize,
    },
    moderations::{ModerationCreateParams, ModerationInput},
    shared::ModelId,
    uploads::{UploadCompleteParams, UploadCreateParams, UploadPartCreateParams},
    Client, ClientConfig, Error, OneOrMany, RawJsonExt, RequestOptions,
};
use reqwest::{header::HeaderValue, Method};
use serde_json::json;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use wiremock::{
    matchers::{body_partial_json, header, header_regex, method, path, query_param},
    Mock, MockServer, Request, Respond, ResponseTemplate,
};

#[derive(Debug)]
struct StaticSubjectTokenProvider {
    token: String,
    token_type: SubjectTokenType,
}

#[async_trait::async_trait]
impl SubjectTokenProvider for StaticSubjectTokenProvider {
    fn token_type(&self) -> SubjectTokenType {
        self.token_type
    }

    async fn get_token(&self, _http_client: &reqwest::Client) -> Result<String, AuthError> {
        Ok(self.token.clone())
    }
}

struct SequentialTokenResponder {
    calls: Arc<AtomicUsize>,
}

impl Respond for SequentialTokenResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        ResponseTemplate::new(200).set_body_json(json!({
            "access_token": format!("token-{call}"),
            "expires_in": 3600
        }))
    }
}

struct UnauthorizedThenModelsResponder {
    calls: Arc<AtomicUsize>,
}

impl Respond for UnauthorizedThenModelsResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        if call == 1 {
            ResponseTemplate::new(401).set_body_json(json!({
                "error": {
                    "message": "Unauthorized",
                    "type": "invalid_request_error",
                    "code": "unauthorized",
                    "param": null
                }
            }))
        } else {
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_json(json!({ "object": "list", "data": [] }))
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RawOptionResponse {
    ok: bool,
}

struct FilesPaginationResponder {
    calls: Arc<AtomicUsize>,
}

impl Respond for FilesPaginationResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        let (id, filename, has_more) = if call == 1 {
            ("file_1", "one.txt", true)
        } else {
            ("file_2", "two.txt", false)
        };

        ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id": id,
                "object":"file",
                "bytes":5,
                "created_at":123,
                "filename": filename,
                "purpose":"assistants"
            }],
            "has_more": has_more
        }))
    }
}

#[tokio::test]
async fn retries_retryable_status_and_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": {
                "message": "upstream unavailable",
                "type": "server_error",
                "code": "internal_error",
                "param": null
            }
        })))
        .mount(&server)
        .await;

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri())
            .with_max_retries(2),
    )
    .expect("client init");

    let err = client.models().list().await.expect_err("must fail");
    match err {
        Error::Api {
            status,
            message,
            code,
            ..
        } => {
            assert_eq!(status, 500);
            assert_eq!(message, "upstream unavailable");
            assert_eq!(code.as_deref(), Some("internal_error"));
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let requests = server.received_requests().await.expect("read requests");
    assert_eq!(requests.len(), 3, "initial call + 2 retries");
}

#[tokio::test]
async fn injects_auth_headers_and_global_query_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .and(header("authorization", "Bearer test-key"))
        .and(header("openai-organization", "org_123"))
        .and(header("openai-project", "proj_123"))
        .and(header_regex("user-agent", "OpenAI/Rust .*"))
        .and(header("x-stainless-lang", "rust"))
        .and(header("x-stainless-retry-count", "0"))
        .and(header_regex("x-stainless-timeout", "[0-9]+"))
        .and(query_param("api-version", "2024-10-01"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_json(json!({ "object": "list", "data": [] })),
        )
        .mount(&server)
        .await;

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri())
            .with_organization("org_123")
            .with_project("proj_123")
            .with_query_param("api-version", "2024-10-01"),
    )
    .expect("client init");

    let list = client.models().list().await.expect("list models");
    assert_eq!(list.object, "list");
    assert!(list.data.is_empty());
}

#[tokio::test]
async fn request_options_override_retries_for_raw_execute() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": {
                "message": "no retry",
                "type": "server_error",
                "code": "internal_error",
                "param": null
            }
        })))
        .mount(&server)
        .await;

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri())
            .with_max_retries(2),
    )
    .expect("client init");

    let err = client
        .execute_with_options::<serde_json::Value, serde_json::Value>(
            Method::GET,
            "/models",
            None,
            RequestOptions::new().with_max_retries(0),
        )
        .await
        .expect_err("must fail");
    assert!(matches!(err, Error::Api { status: 500, .. }));

    let requests = server.received_requests().await.expect("read requests");
    assert_eq!(requests.len(), 1);
}

#[tokio::test]
async fn request_options_apply_full_raw_request_shape() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/raw"))
        .and(query_param("mode", "set"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "unknown": {"from": "server"}
        })))
        .mount(&server)
        .await;

    let response_capture = Arc::new(Mutex::new(None));
    let body_capture = Arc::new(Mutex::new(None));
    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri()),
    )
    .expect("client init");

    let response: RawOptionResponse = client
        .execute_with_options::<serde_json::Value, RawOptionResponse>(
            Method::POST,
            "/raw",
            None,
            RequestOptions::new()
                .with_query_param("remove", "yes")
                .with_query_del("remove")
                .with_query("mode", "set")
                .with_header("x-remove", "yes")
                .with_header_del("x-remove")
                .with_header_add("x-added", "yes")
                .with_request_body(
                    "application/json",
                    br#"{"remove":true,"nested":{}}"#.to_vec(),
                )
                .with_json_set("nested.value", 42)
                .with_json_del("remove")
                .with_response_into(response_capture.clone())
                .with_response_body_into(body_capture.clone())
                .with_middleware(|mut request, next| async move {
                    request
                        .headers_mut()
                        .insert("x-middleware", HeaderValue::from_static("yes"));
                    next(request).await
                }),
        )
        .await
        .expect("raw execute");

    assert!(response.ok);
    assert!(response.raw_json().contains("\"unknown\""));
    assert!(response.extra_fields().contains_key("unknown"));

    let captured = response_capture
        .lock()
        .expect("response capture lock")
        .clone()
        .expect("captured response");
    assert_eq!(captured.status, 200);
    assert!(captured.url.contains("mode=set"));

    let captured_body = body_capture
        .lock()
        .expect("body capture lock")
        .clone()
        .expect("captured response body");
    assert!(String::from_utf8(captured_body)
        .expect("utf8 body")
        .contains("\"unknown\""));

    let requests = server.received_requests().await.expect("read requests");
    let request = requests
        .iter()
        .find(|request| request.url.path() == "/raw")
        .expect("raw request");
    assert!(request.headers.get("x-remove").is_none());
    assert_eq!(
        request
            .headers
            .get("x-added")
            .and_then(|value| value.to_str().ok()),
        Some("yes")
    );
    assert_eq!(
        request
            .headers
            .get("x-middleware")
            .and_then(|value| value.to_str().ok()),
        Some("yes")
    );

    let body: serde_json::Value = serde_json::from_slice(&request.body).expect("request body json");
    assert!(body.get("remove").is_none());
    assert_eq!(body["nested"]["value"], 42);
    assert_eq!(request.url.query(), Some("mode=set"));
}

#[tokio::test]
async fn cursor_pages_fetch_next_page_with_last_item_id() {
    let server = MockServer::start().await;
    let calls = Arc::new(AtomicUsize::new(0));

    Mock::given(method("GET"))
        .and(path("/files"))
        .and(query_param("limit", "1"))
        .respond_with(FilesPaginationResponder {
            calls: calls.clone(),
        })
        .expect(2)
        .mount(&server)
        .await;

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri())
            .with_max_retries(0),
    )
    .expect("client init");

    let first = client
        .files()
        .list(Some(FileListParams {
            after: None,
            limit: Some(1),
            purpose: None,
            order: None,
        }))
        .await
        .expect("first page");
    assert_eq!(first.data[0].id, "file_1");

    let second = first
        .next_page()
        .await
        .expect("next page request")
        .expect("next page");
    assert_eq!(second.data[0].id, "file_2");

    let requests = server.received_requests().await.expect("read requests");
    let second_query = requests
        .iter()
        .filter(|request| request.url.path() == "/files")
        .nth(1)
        .expect("second files request")
        .url
        .query()
        .unwrap_or_default()
        .to_owned();
    assert!(second_query.contains("limit=1"));
    assert!(second_query.contains("after=file_1"));
}

#[tokio::test]
async fn workload_identity_exchanges_token_and_injects_bearer_auth() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(body_partial_json(json!({
            "client_id": "client-id",
            "subject_token": "subject-token",
            "subject_token_type": "urn:ietf:params:oauth:token-type:jwt",
            "identity_provider_id": "idp-id",
            "service_account_id": "sa-id"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "exchanged-token",
            "expires_in": 3600
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .and(header("authorization", "Bearer exchanged-token"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_json(json!({ "object": "list", "data": [] })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let workload_identity = WorkloadIdentity::new(
        "client-id",
        "idp-id",
        "sa-id",
        Arc::new(StaticSubjectTokenProvider {
            token: "subject-token".to_owned(),
            token_type: SubjectTokenType::Jwt,
        }),
    )
    .with_token_exchange_url(format!("{}/oauth/token", server.uri()));
    let client = Client::new(
        ClientConfig::default()
            .with_api_key("static-key-should-not-be-used")
            .with_base_url(server.uri())
            .with_workload_identity(workload_identity),
    )
    .expect("client init");

    let list = client.models().list().await.expect("list models");
    assert_eq!(list.object, "list");
}

#[tokio::test]
async fn workload_identity_invalidates_token_and_retries_once_on_401() {
    let server = MockServer::start().await;
    let token_calls = Arc::new(AtomicUsize::new(0));
    let api_calls = Arc::new(AtomicUsize::new(0));

    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(SequentialTokenResponder {
            calls: token_calls.clone(),
        })
        .expect(2)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(UnauthorizedThenModelsResponder {
            calls: api_calls.clone(),
        })
        .expect(2)
        .mount(&server)
        .await;

    let workload_identity = WorkloadIdentity::new(
        "client-id",
        "idp-id",
        "sa-id",
        Arc::new(StaticSubjectTokenProvider {
            token: "subject-token".to_owned(),
            token_type: SubjectTokenType::Jwt,
        }),
    )
    .with_token_exchange_url(format!("{}/oauth/token", server.uri()));
    let client = Client::new(
        ClientConfig::default()
            .with_base_url(server.uri())
            .with_workload_identity(workload_identity),
    )
    .expect("client init");

    let list = client.models().list().await.expect("list models");
    assert_eq!(list.object, "list");
    assert_eq!(token_calls.load(Ordering::SeqCst), 2);
    assert_eq!(api_calls.load(Ordering::SeqCst), 2);

    let requests = server.received_requests().await.expect("read requests");
    let model_auth_headers = requests
        .iter()
        .filter(|request| request.url.path() == "/models")
        .map(|request| {
            request
                .headers
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        model_auth_headers,
        vec!["Bearer token-1".to_owned(), "Bearer token-2".to_owned()]
    );
}

#[tokio::test]
async fn chat_create_posts_expected_payload() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(body_partial_json(json!({"model":"gpt-4o-mini"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"chatcmpl_1",
            "object":"chat.completion",
            "model":"gpt-4o-mini",
            "choices":[{"index":0,"message":{"role":"assistant","content":"hello"},"finish_reason":"stop"}]
        })))
        .mount(&server)
        .await;

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri()),
    )
    .expect("client init");

    let response = client
        .chat()
        .completions()
        .create(ChatCompletionCreateParams {
            model: ModelId::from("gpt-4o-mini"),
            messages: vec![ChatCompletionMessageParam::User {
                content: ChatCompletionUserMessageContent::Text("hi".to_owned()),
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
        .expect("chat completion create");

    assert_eq!(response.id, "chatcmpl_1");
    assert_eq!(response.choices.len(), 1);
}

#[tokio::test]
async fn core_services_create_requests_succeed() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"cmpl_1",
            "object":"text_completion",
            "choices":[{"index":0,"text":"ok","finish_reason":"stop"}]
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{"object":"embedding","index":0,"embedding":[0.1,0.2]}]
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/moderations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"modr_1",
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

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri()),
    )
    .expect("client init");

    let completion = client
        .completions()
        .create(CompletionCreateParams {
            model: ModelId::from("gpt-3.5-turbo-instruct"),
            prompt: openai::completions::CompletionPrompt::Single("hello".to_owned()),
            max_tokens: None,
            temperature: None,
            ..Default::default()
        })
        .await
        .expect("completion create");
    assert_eq!(completion.id, "cmpl_1");

    let embeddings = client
        .embeddings()
        .create(EmbeddingCreateParams {
            model: ModelId::from("text-embedding-3-small"),
            input: OneOrMany::One("hello".to_owned()),
            dimensions: None,
            user: None,
            encoding_format: None,
        })
        .await
        .expect("embeddings create");
    assert_eq!(embeddings.data.len(), 1);

    let moderation = client
        .moderations()
        .create(ModerationCreateParams {
            model: Some(ModelId::from("omni-moderation-latest")),
            input: ModerationInput::from("hello"),
        })
        .await
        .expect("moderations create");
    assert_eq!(moderation.id, "modr_1");
}

#[tokio::test]
async fn models_get_and_delete_succeed() {
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

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri()),
    )
    .expect("client init");

    let model = client.models().get("gpt-4o-mini").await.expect("get model");
    assert_eq!(model.id.as_ref(), "gpt-4o-mini");

    let deleted = client
        .models()
        .delete("ft:gpt-4o:custom")
        .await
        .expect("delete model");
    assert!(deleted.deleted);
}

#[tokio::test]
async fn files_upload_and_content_download_use_expected_paths() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/files"))
        .and(header_regex("content-type", "multipart/form-data;.*"))
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

    Mock::given(method("GET"))
        .and(path("/files/file_1/content"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("hello", "application/octet-stream"))
        .mount(&server)
        .await;

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri()),
    )
    .expect("client init");

    let file = client
        .files()
        .create(FileCreateParams {
            purpose: FilePurpose::Assistants,
            file: FileUploadPart::from_bytes("hello", "input.txt").with_content_type("text/plain"),
            expires_after: None,
        })
        .await
        .expect("file create");
    assert_eq!(file.id, "file_1");

    let content = client
        .files()
        .content("file_1")
        .await
        .expect("file content");
    assert_eq!(content, b"hello");
}

#[tokio::test]
async fn audio_image_and_upload_requests_work() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/audio/transcriptions"))
        .and(header_regex("content-type", "multipart/form-data;.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"text":"hello world"})))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/audio/speech"))
        .and(body_partial_json(json!({"voice":"alloy"})))
        .respond_with(ResponseTemplate::new(200).set_body_raw("AUDIO", "audio/mpeg"))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/images/generations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "created":123,
            "data":[{"url":"https://example.com/image.png"}]
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/images/edits"))
        .and(header_regex("content-type", "multipart/form-data;.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "created":124,
            "data":[{"b64_json":"ZmFrZQ=="}]
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"upload_1",
            "object":"upload",
            "bytes":5,
            "filename":"input.bin",
            "purpose":"assistants"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/uploads/upload_1/parts"))
        .and(header_regex("content-type", "multipart/form-data;.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"part_1",
            "object":"upload.part",
            "upload_id":"upload_1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/uploads/upload_1/complete"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"upload_1",
            "object":"upload",
            "bytes":5,
            "filename":"input.bin",
            "purpose":"assistants",
            "status":"completed"
        })))
        .mount(&server)
        .await;

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri()),
    )
    .expect("client init");

    let transcription = client
        .audio()
        .transcribe(AudioTranscriptionCreateParams {
            file: AudioInputFile::from_bytes("abc", "audio.wav").with_content_type("audio/wav"),
            model: ModelId::from("gpt-4o-mini-transcribe"),
            prompt: None,
            language: None,
            temperature: None,
            response_format: None,
            include: None,
            chunking_strategy: None,
            timestamp_granularities: None,
            known_speaker_names: None,
            known_speaker_references: None,
        })
        .await
        .expect("audio transcription");
    assert_eq!(transcription.text, "hello world");

    let speech = client
        .audio()
        .speech(AudioSpeechCreateParams {
            model: ModelId::from("gpt-4o-mini-tts"),
            input: "hello".to_owned(),
            voice: AudioSpeechVoiceParam::BuiltIn(AudioSpeechVoice::Alloy),
            response_format: Some(AudioSpeechResponseFormat::Mp3),
            instructions: None,
            speed: None,
            stream_format: None,
        })
        .await
        .expect("speech bytes");
    assert_eq!(speech, b"AUDIO");

    let generated = client
        .images()
        .generate(ImageGenerateParams {
            prompt: "draw mountain".to_owned(),
            model: Some(ModelId::from("gpt-image-1")),
            n: Some(1),
            size: Some(ImageSize::Size1024x1024),
            response_format: Some(ImageResponseFormat::Url),
            output_compression: None,
            partial_images: None,
            user: None,
            background: None,
            moderation: None,
            output_format: None,
            quality: None,
            style: None,
        })
        .await
        .expect("image generate");
    assert_eq!(generated.data.len(), 1);

    let edited = client
        .images()
        .edit(ImageEditParams {
            image: openai::images::ImageEditInput::Single(
                ImageInputFile::from_bytes("png-bytes", "input.png").with_content_type("image/png"),
            ),
            prompt: "add a cloud".to_owned(),
            mask: None,
            model: Some(ModelId::from("gpt-image-1")),
            n: Some(1),
            size: None,
            response_format: Some(ImageResponseFormat::B64Json),
            output_compression: None,
            partial_images: None,
            user: None,
            background: None,
            input_fidelity: None,
            output_format: None,
            quality: None,
        })
        .await
        .expect("image edit");
    assert_eq!(edited.data.len(), 1);

    let upload = client
        .uploads()
        .create(UploadCreateParams {
            bytes: 5,
            filename: "input.bin".to_owned(),
            purpose: "assistants".to_owned(),
            mime_type: "application/octet-stream".to_owned(),
            expires_after: None,
        })
        .await
        .expect("upload create");
    assert_eq!(upload.id, "upload_1");

    let part = client
        .uploads()
        .create_part(
            "upload_1",
            UploadPartCreateParams {
                data: b"hello".to_vec(),
                filename: Some("part-1.bin".to_owned()),
                content_type: Some("application/octet-stream".to_owned()),
            },
        )
        .await
        .expect("upload part");
    assert_eq!(part.id, "part_1");

    let completed = client
        .uploads()
        .complete(
            "upload_1",
            UploadCompleteParams {
                part_ids: vec!["part_1".to_owned()],
                md5: None,
            },
        )
        .await
        .expect("upload complete");
    assert_eq!(
        completed.status,
        Some(openai::uploads::UploadStatus::Completed)
    );
}
