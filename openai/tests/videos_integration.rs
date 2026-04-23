//! Integration tests for videos service.

use std::time::Duration;

use openai::{
    videos::{
        VideoCreateParams, VideoDownloadContentParams, VideoEditParams, VideoExtendParams,
        VideoFileUpload, VideoListParams, VideoNewCharacterParams, VideoReference,
        VideoRemixParams,
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

fn video_json(id: &str, status: &str) -> serde_json::Value {
    json!({
        "id": id,
        "object": "video",
        "created_at": 1700000000,
        "completed_at": 0,
        "expires_at": 0,
        "progress": 0,
        "prompt": "A cat in space",
        "status": status
    })
}

#[tokio::test]
async fn videos_crud_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/videos"))
        .respond_with(ResponseTemplate::new(200).set_body_json(video_json("video_1", "queued")))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/videos/video_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(video_json("video_1", "completed")))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/videos"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[video_json("video_1", "completed")],
            "has_more":false,
            "first_id":"video_1",
            "last_id":"video_1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/videos/video_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"video_1",
            "object":"video.deleted",
            "deleted":true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let videos = client.videos();

    // Create
    let created = videos
        .create(VideoCreateParams {
            prompt: "A cat in space".to_owned(),
            input_reference: None,
            model: Some("sora-2".to_owned()),
            seconds: Some("4".to_owned()),
            size: Some("720x1280".to_owned()),
        })
        .await
        .expect("create video");
    assert_eq!(created.id, "video_1");

    // Get
    let got = videos.get("video_1").await.expect("get video");
    assert_eq!(got.id, "video_1");

    // List
    let listed = videos
        .list(VideoListParams {
            after: None,
            limit: None,
            order: None,
        })
        .await
        .expect("list videos");
    assert_eq!(listed.data.len(), 1);

    // Delete
    let deleted = videos.delete("video_1").await.expect("delete video");
    assert!(deleted.deleted);
}

#[tokio::test]
async fn video_create_and_poll_returns_terminal_video() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/videos"))
        .respond_with(ResponseTemplate::new(200).set_body_json(video_json("video_poll", "queued")))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/videos/video_poll"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(video_json("video_poll", "completed")),
        )
        .mount(&server)
        .await;

    let client = test_client(&server);
    let video = client
        .videos()
        .create_and_poll(
            VideoCreateParams {
                prompt: "A cat in space".to_owned(),
                input_reference: None,
                model: Some("sora-2".to_owned()),
                seconds: Some("4".to_owned()),
                size: Some("720x1280".to_owned()),
            },
            Duration::from_millis(0),
        )
        .await
        .expect("create and poll video");

    assert_eq!(video.id, "video_poll");
}

#[tokio::test]
async fn video_character_operations() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/videos/characters"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"char_1",
            "created_at":1700000000,
            "name":"Hero"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/videos/characters/char_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"char_1",
            "created_at":1700000000,
            "name":"Hero"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let videos = client.videos();

    let character = videos
        .create_character(VideoNewCharacterParams {
            name: "Hero".to_owned(),
            video: VideoFileUpload::from_bytes(b"videodata".to_vec(), "char.mp4"),
        })
        .await
        .expect("create character");
    assert_eq!(character.id, "char_1");
    assert_eq!(character.name, "Hero");

    let fetched = videos.get_character("char_1").await.expect("get character");
    assert_eq!(fetched.id, "char_1");
}

#[tokio::test]
async fn video_download_content() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/videos/video_1/content"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"mp4bytes".to_vec()))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let bytes = client
        .videos()
        .download_content("video_1", VideoDownloadContentParams { variant: None })
        .await
        .expect("download content");
    assert_eq!(bytes, b"mp4bytes");
}

#[tokio::test]
async fn video_edit() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/videos/edits"))
        .respond_with(ResponseTemplate::new(200).set_body_json(video_json("video_2", "queued")))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let edited = client
        .videos()
        .edit(VideoEditParams {
            prompt: "Make it blue".to_owned(),
            video: VideoReference::File(VideoFileUpload::from_bytes(
                b"videodata".to_vec(),
                "source.mp4",
            )),
        })
        .await
        .expect("edit video");
    assert_eq!(edited.id, "video_2");
}

#[tokio::test]
async fn video_extend() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/videos/extensions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(video_json("video_3", "queued")))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let extended = client
        .videos()
        .extend(VideoExtendParams {
            prompt: "Continue the scene".to_owned(),
            seconds: "8".to_owned(),
            video: VideoReference::Id("video_1".to_owned()),
        })
        .await
        .expect("extend video");
    assert_eq!(extended.id, "video_3");
}

#[tokio::test]
async fn video_remix() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/videos/video_1/remix"))
        .and(body_partial_json(json!({"prompt":"Different style"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(video_json("video_4", "queued")))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let remixed = client
        .videos()
        .remix(
            "video_1",
            VideoRemixParams {
                prompt: "Different style".to_owned(),
            },
        )
        .await
        .expect("remix video");
    assert_eq!(remixed.id, "video_4");
}
