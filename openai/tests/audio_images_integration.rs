//! Integration tests for audio and images services.

use openai::{
    audio::{AudioInputFile, AudioTranslationCreateParams},
    images::{ImageInputFile, ImageResponseFormat, ImageSize, ImageVariationParams},
    shared::ModelId,
    Client, ClientConfig,
};
use serde_json::json;
use wiremock::{
    matchers::{header_regex, method, path},
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
async fn audio_translate_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/audio/translations"))
        .and(header_regex("content-type", "multipart/form-data;.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "text":"translated text"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);

    let translated = client
        .audio()
        .translate(AudioTranslationCreateParams {
            file: AudioInputFile::from_bytes("abc", "audio.wav").with_content_type("audio/wav"),
            model: ModelId::from("gpt-4o-mini-transcribe"),
            prompt: None,
            temperature: None,
            response_format: None,
        })
        .await
        .expect("audio translate");

    assert_eq!(translated.text, "translated text");
}

#[tokio::test]
async fn images_create_variation_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/images/variations"))
        .and(header_regex("content-type", "multipart/form-data;.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "created": 1700000000,
            "data": [{"url": "https://example.com/variation.png"}]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);

    let response = client
        .images()
        .create_variation(ImageVariationParams {
            image: ImageInputFile::from_bytes("png-bytes", "input.png")
                .with_content_type("image/png"),
            model: Some(ModelId::from("gpt-image-1")),
            n: Some(1),
            size: Some(ImageSize::Size1024x1024),
            response_format: Some(ImageResponseFormat::Url),
            user: None,
        })
        .await
        .expect("image variation");

    assert_eq!(response.data.len(), 1);
    assert_eq!(
        response.data[0].url.as_deref(),
        Some("https://example.com/variation.png")
    );
}
