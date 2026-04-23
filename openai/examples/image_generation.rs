#![allow(missing_docs)]
// Image generation example.
//
// Generates an image from a text prompt using DALL-E and prints the URL.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example image_generation

use openai::{
    images::{ImageGenerateParams, ImageSize},
    shared::ModelId,
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    let params = ImageGenerateParams {
        prompt: "A watercolor painting of a crab writing Rust code on a laptop".to_owned(),
        model: Some(ModelId::from("dall-e-3")),
        n: Some(1),
        size: Some(ImageSize::Size1024x1024),
        response_format: None,
        output_compression: None,
        partial_images: None,
        user: None,
        background: None,
        moderation: None,
        output_format: None,
        quality: None,
        style: None,
    };

    let response = client.images().generate(params).await?;

    for (i, image) in response.data.iter().enumerate() {
        if let Some(url) = &image.url {
            println!("Image {}: {url}", i + 1);
        }
        if let Some(revised) = &image.revised_prompt {
            println!("Revised prompt: {revised}");
        }
    }

    Ok(())
}
