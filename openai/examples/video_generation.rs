#![allow(missing_docs)]
// Video generation example.
//
// Creates a video generation job from a text prompt.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example video_generation

use openai::{videos::VideoCreateParams, Client};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    let params = VideoCreateParams {
        prompt: "A serene forest scene with sunlight filtering through the trees".to_owned(),
        input_reference: None,
        model: Some("sora".to_owned()),
        seconds: Some("5".to_owned()),
        size: Some("1920x1080".to_owned()),
    };

    let video = client.videos().create(params).await?;

    println!("Video ID: {}", video.id);
    println!("Status: {:?}", video.status);
    println!("Created at: {}", video.created_at);

    Ok(())
}
