//! Integration tests for the core client behavior.

use openai::{
    audio::{AudioInputFile, AudioSpeechCreateParams, AudioTranscriptionCreateParams},
    chat::{ChatCompletionCreateParams, ChatMessageParam, ChatRole},
    completions::CompletionCreateParams,
    embeddings::EmbeddingCreateParams,
    files::{FileCreateParams, FileUploadPart},
    images::{ImageEditParams, ImageGenerateParams, ImageInputFile},
    moderations::ModerationCreateParams,
    shared::ModelId,
    uploads::{UploadCompleteParams, UploadCreateParams, UploadPartCreateParams},
    Client, ClientConfig, Error, OneOrMany,
};
use serde_json::json;
use wiremock::{
    matchers::{body_partial_json, header, header_regex, method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

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
            messages: vec![ChatMessageParam {
                role: ChatRole::User,
                content: "hi".to_owned(),
            }],
            stream: None,
            temperature: None,
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
            "results":[{"flagged":false}]
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
            prompt: "hello".to_owned(),
            max_tokens: None,
            temperature: None,
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
        })
        .await
        .expect("embeddings create");
    assert_eq!(embeddings.data.len(), 1);

    let moderation = client
        .moderations()
        .create(ModerationCreateParams {
            model: Some(ModelId::from("omni-moderation-latest")),
            input: OneOrMany::One("hello".to_owned()),
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
            purpose: "assistants".to_owned(),
            file: FileUploadPart::from_bytes("hello", "input.txt").with_content_type("text/plain"),
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
        })
        .await
        .expect("audio transcription");
    assert_eq!(transcription.text, "hello world");

    let speech = client
        .audio()
        .speech(AudioSpeechCreateParams {
            model: ModelId::from("gpt-4o-mini-tts"),
            input: "hello".to_owned(),
            voice: "alloy".to_owned(),
            format: Some("mp3".to_owned()),
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
            size: Some("1024x1024".to_owned()),
            response_format: Some("url".to_owned()),
        })
        .await
        .expect("image generate");
    assert_eq!(generated.data.len(), 1);

    let edited = client
        .images()
        .edit(ImageEditParams {
            image: ImageInputFile::from_bytes("png-bytes", "input.png")
                .with_content_type("image/png"),
            prompt: "add a cloud".to_owned(),
            mask: None,
            model: Some(ModelId::from("gpt-image-1")),
            n: Some(1),
            size: None,
            response_format: Some("b64_json".to_owned()),
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
            mime_type: Some("application/octet-stream".to_owned()),
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
            },
        )
        .await
        .expect("upload complete");
    assert_eq!(completed.status.as_deref(), Some("completed"));
}
