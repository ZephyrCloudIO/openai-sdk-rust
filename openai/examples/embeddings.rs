#![allow(missing_docs)]
// Embeddings example.
//
// Creates an embedding vector for the provided text input and prints the
// resulting dimensions and first few values.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example embeddings

use openai::{embeddings::EmbeddingCreateParams, param::OneOrMany, shared::ModelId, Client};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    let params = EmbeddingCreateParams {
        model: ModelId::from("text-embedding-3-small"),
        input: OneOrMany::One("The quick brown fox jumps over the lazy dog.".to_owned()),
        dimensions: None,
        user: None,
        encoding_format: None,
    };

    let response = client.embeddings().create(params).await?;

    for item in &response.data {
        println!(
            "Embedding index {}: {} dimensions",
            item.index,
            item.embedding.len()
        );
        let preview: Vec<_> = item.embedding.iter().take(5).collect();
        println!("First 5 values: {preview:?}");
    }

    if let Some(usage) = &response.usage {
        println!(
            "\nTokens used: {} prompt, {} total",
            usage.prompt_tokens, usage.total_tokens
        );
    }

    Ok(())
}
