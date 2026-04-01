//! Integration tests for streaming endpoints.

use futures::{pin_mut, StreamExt};
use openai::{
    chat::{ChatCompletionCreateParams, ChatMessageParam, ChatRole},
    shared::ModelId,
    Client, ClientConfig, Error,
};
use wiremock::{
    matchers::{method, path},
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
            messages: vec![ChatMessageParam {
                role: ChatRole::User,
                content: "hello".to_owned(),
            }],
            stream: None,
            temperature: None,
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
            messages: vec![ChatMessageParam {
                role: ChatRole::User,
                content: "hello".to_owned(),
            }],
            stream: None,
            temperature: None,
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
