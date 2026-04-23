#![allow(missing_docs)]
// List models example.
//
// Retrieves all available models from the API and prints their IDs.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example models_list

use openai::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    let model_list = client.models().list().await?;

    println!("Available models ({} total):", model_list.data.len());
    for model in &model_list.data {
        let owner = &model.owned_by;
        println!("  {} (owned by: {owner})", model.id);
    }

    Ok(())
}
