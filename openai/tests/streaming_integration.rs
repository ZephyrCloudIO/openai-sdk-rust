//! Integration tests for streaming endpoints.

use futures::{pin_mut, StreamExt};
use openai::{
    audio::{AudioInputFile, AudioTranscriptionCreateParams},
    chat::{
        ChatCompletionCreateParams, ChatCompletionMessageParam, ChatCompletionUserMessageContent,
    },
    completions::{CompletionCreateParams, CompletionPrompt},
    images::{ImageEditInput, ImageEditParams, ImageGenerateParams, ImageInputFile},
    shared::ModelId,
    Client, ClientConfig, Error,
};
use serde_json::json;
use wiremock::{
    matchers::{body_partial_json, header_regex, method, path},
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
async fn chat_create_stream_yields_chunks_and_stops_on_done() {
    let server = MockServer::start().await;

    let body = concat!(
        "data: {\"id\":\"chatcmpl_1\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"hel\"},\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"chatcmpl_1\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"lo\"},\"finish_reason\":\"stop\"}]}\n\n",
        "data: [DONE]\n\n"
    );

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body, "text/event-stream"),
        )
        .mount(&server)
        .await;

    let client = test_client(&server);
    let stream = client
        .chat()
        .completions()
        .create_stream(ChatCompletionCreateParams {
            model: ModelId::from("gpt-4o-mini"),
            messages: vec![ChatCompletionMessageParam::User {
                content: ChatCompletionUserMessageContent::Text("hello".to_owned()),
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
        .expect("create stream");

    pin_mut!(stream);

    let first = stream
        .next()
        .await
        .expect("first event")
        .expect("first event ok");
    assert_eq!(first.choices.len(), 1);

    let second = stream
        .next()
        .await
        .expect("second event")
        .expect("second event ok");
    assert_eq!(
        second.choices[0].finish_reason,
        Some(openai::shared::FinishReason::Stop)
    );

    assert!(
        stream.next().await.is_none(),
        "stream should end after [DONE]"
    );
}

#[tokio::test]
async fn chat_create_stream_surfaces_error_event() {
    let server = MockServer::start().await;

    let body = concat!(
        "event: error\n",
        "data: {\"error\":{\"message\":\"stream failed\"}}\n\n"
    );

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body, "text/event-stream"),
        )
        .mount(&server)
        .await;

    let client = test_client(&server);
    let stream = client
        .chat()
        .completions()
        .create_stream(ChatCompletionCreateParams {
            model: ModelId::from("gpt-4o-mini"),
            messages: vec![ChatCompletionMessageParam::User {
                content: ChatCompletionUserMessageContent::Text("hello".to_owned()),
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
        .expect("create stream");

    pin_mut!(stream);

    let first = stream.next().await.expect("stream event");
    match first {
        Err(Error::Stream(message)) => assert_eq!(message, "stream failed"),
        other => panic!("unexpected stream result: {other:?}"),
    }

    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn completions_create_stream_yields_completion() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/completions"))
        .and(body_partial_json(json!({"stream": true})))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(
                    "data: {\"id\":\"cmpl_1\",\"object\":\"text_completion\",\"choices\":[{\"index\":0,\"text\":\"hello\",\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n",
                    "text/event-stream",
                ),
        )
        .mount(&server)
        .await;

    let client = test_client(&server);
    let stream = client
        .completions()
        .create_stream(CompletionCreateParams {
            model: ModelId::from("gpt-3.5-turbo-instruct"),
            prompt: CompletionPrompt::Single("say hi".to_owned()),
            ..CompletionCreateParams::default()
        })
        .await
        .expect("completion stream");

    pin_mut!(stream);
    let event = stream
        .next()
        .await
        .expect("event")
        .expect("event should decode");
    assert_eq!(event.choices[0].text, "hello");
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn completion_adjacent_streaming_endpoints_force_streaming() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/images/generations"))
        .and(body_partial_json(json!({"stream": true})))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(
                    "data: {\"type\":\"image_generation.partial_image\",\"b64_json\":\"abc\",\"partial_image_index\":0}\n\ndata: [DONE]\n\n",
                    "text/event-stream",
                ),
        )
        .mount(&server)
        .await;

    let client = test_client(&server);
    let stream = client
        .images()
        .generate_streaming(ImageGenerateParams {
            prompt: "draw".to_owned(),
            model: Some(ModelId::from("gpt-image-1")),
            n: None,
            size: None,
            response_format: None,
            output_compression: None,
            partial_images: Some(1),
            user: None,
            background: None,
            moderation: None,
            output_format: None,
            quality: None,
            style: None,
        })
        .await
        .expect("image stream");

    pin_mut!(stream);
    assert!(stream.next().await.expect("event").is_ok());
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn multipart_streaming_endpoints_include_stream_flag() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/audio/transcriptions"))
        .and(header_regex("content-type", "multipart/form-data;.*"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(
                    "data: {\"type\":\"transcript.text.delta\",\"delta\":\"hi\"}\n\ndata: [DONE]\n\n",
                    "text/event-stream",
                ),
        )
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/images/edits"))
        .and(header_regex("content-type", "multipart/form-data;.*"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(
                    "data: {\"type\":\"image_edit.partial_image\",\"b64_json\":\"abc\",\"partial_image_index\":0}\n\ndata: [DONE]\n\n",
                    "text/event-stream",
                ),
        )
        .mount(&server)
        .await;

    let client = test_client(&server);

    let audio_stream = client
        .audio()
        .transcribe_streaming(AudioTranscriptionCreateParams {
            file: AudioInputFile::from_bytes("abc", "audio.wav"),
            model: ModelId::from("gpt-4o-transcribe"),
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
        .expect("audio stream");
    pin_mut!(audio_stream);
    assert!(audio_stream.next().await.expect("event").is_ok());

    let image_stream = client
        .images()
        .edit_streaming(ImageEditParams {
            image: ImageEditInput::Single(ImageInputFile::from_bytes("abc", "image.png")),
            prompt: "edit".to_owned(),
            mask: None,
            model: Some(ModelId::from("gpt-image-1")),
            n: None,
            size: None,
            response_format: None,
            output_compression: None,
            partial_images: Some(1),
            user: None,
            background: None,
            input_fidelity: None,
            output_format: None,
            quality: None,
        })
        .await
        .expect("image edit stream");
    pin_mut!(image_stream);
    assert!(image_stream.next().await.expect("event").is_ok());

    let requests = server.received_requests().await.expect("requests");
    for endpoint in ["/audio/transcriptions", "/images/edits"] {
        let request = requests
            .iter()
            .find(|request| request.url.path() == endpoint)
            .expect("streaming request");
        let body = String::from_utf8_lossy(&request.body);
        assert!(body.contains("name=\"stream\""));
        assert!(body.contains("true"));
    }
}
