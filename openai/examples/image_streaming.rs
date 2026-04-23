#![allow(missing_docs)]
// Streaming image generation example.
//
// Uses the image generation streaming endpoint to receive partial images as
// they are produced, then saves the final completed image. This demonstrates
// the ImageGenStreamEvent types (PartialImage and Completed).
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example image_streaming

use std::path::PathBuf;

use base64::Engine;
use futures::StreamExt;
use openai::{
    images::{ImageGenStreamEvent, ImageGenerateParams, ImageModel, ImageSize},
    shared::ModelId,
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    println!("Starting image streaming example...");

    let params = ImageGenerateParams {
        prompt: "A cute baby sea otter".to_owned(),
        model: Some(ModelId::from(ImageModel::GptImage1.to_string())),
        n: Some(1),
        size: Some(ImageSize::Size1024x1024),
        partial_images: Some(3),
        response_format: None,
        output_compression: None,
        user: None,
        background: None,
        moderation: None,
        output_format: None,
        quality: None,
        style: None,
    };

    let stream = client.images().generate_streaming(params).await?;
    futures::pin_mut!(stream);

    while let Some(event_result) = stream.next().await {
        let event = event_result?;

        match event {
            ImageGenStreamEvent::PartialImage(partial) => {
                println!(
                    "  Partial image {}/3 received",
                    partial.partial_image_index + 1
                );
                println!("   Size: {} characters (base64)", partial.b64_json.len());

                let filename = format!("partial_{}.png", partial.partial_image_index + 1);
                save_base64_image(&partial.b64_json, &filename)?;
                let abs_path =
                    std::fs::canonicalize(&filename).unwrap_or_else(|_| PathBuf::from(&filename));
                println!("   Saved to: {}", abs_path.display());
            }
            ImageGenStreamEvent::Completed(completed) => {
                println!();
                println!("Final image completed!");
                println!("   Size: {} characters (base64)", completed.b64_json.len());

                let filename = "final_image.png";
                save_base64_image(&completed.b64_json, filename)?;
                let abs_path =
                    std::fs::canonicalize(filename).unwrap_or_else(|_| PathBuf::from(filename));
                println!("   Saved to: {}", abs_path.display());

                if let Some(usage) = &completed.usage {
                    println!(
                        "   Tokens: {} input + {} output = {} total",
                        usage.input_tokens, usage.output_tokens, usage.total_tokens
                    );
                }
            }
        }
    }

    Ok(())
}

fn save_base64_image(b64_data: &str, filename: &str) -> anyhow::Result<()> {
    let image_data = base64::engine::general_purpose::STANDARD.decode(b64_data)?;
    std::fs::write(filename, image_data)?;
    Ok(())
}
