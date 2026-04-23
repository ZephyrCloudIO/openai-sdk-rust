#![allow(missing_docs)]
// Batches example.
//
// Lists existing batches. If an input file ID is provided as the first
// argument, creates a new batch job targeting the /v1/chat/completions endpoint.
//
// Usage:
//   OPENAI_API_KEY=sk-... cargo run --example batches
//   OPENAI_API_KEY=sk-... cargo run --example batches -- <input-file-id>

use openai::{
    batches::{BatchCompletionWindow, BatchCreateParams, BatchEndpoint},
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    // List existing batches.
    let page = client.batches().list(None).await?;
    println!("Batches ({} on this page):", page.data.len());
    for batch in &page.data {
        println!(
            "  {} - endpoint: {}, status: {:?}",
            batch.id, batch.endpoint, batch.status
        );
    }

    // Optionally create a new batch if an input file ID is provided.
    if let Some(input_file_id) = std::env::args().nth(1) {
        println!("\nCreating batch with input file: {input_file_id}");

        let params = BatchCreateParams {
            input_file_id,
            endpoint: BatchEndpoint::V1ChatCompletions,
            completion_window: BatchCompletionWindow::TwentyFourHours,
            metadata: None,
            output_expires_after: None,
        };

        let batch = client.batches().create(params).await?;
        println!("Created batch: {} (status: {:?})", batch.id, batch.status);
    }

    Ok(())
}
