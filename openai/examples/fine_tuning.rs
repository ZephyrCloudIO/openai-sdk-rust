#![allow(missing_docs)]
// Fine-tuning example.
//
// Lists existing fine-tuning jobs. If a training file ID is provided as the
// first argument, creates a new fine-tuning job.
//
// Usage:
//   OPENAI_API_KEY=sk-... cargo run --example fine_tuning
//   OPENAI_API_KEY=sk-... cargo run --example fine_tuning -- <training-file-id>

use openai::{fine_tuning::FineTuningJobCreateParams, Client};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    // List existing fine-tuning jobs.
    let jobs = client.fine_tuning().list_jobs().await?;
    println!("Fine-tuning jobs ({} on this page):", jobs.data.len());
    for job in &jobs.data {
        println!(
            "  {} - model: {}, status: {:?}",
            job.id, job.model, job.status
        );
    }

    // Optionally create a new job if a training file ID is provided.
    if let Some(training_file) = std::env::args().nth(1) {
        println!("\nCreating fine-tuning job with training file: {training_file}");

        let params = FineTuningJobCreateParams {
            model: "gpt-4o-mini-2024-07-18".to_owned(),
            training_file,
            validation_file: None,
            suffix: None,
            metadata: None,
            seed: None,
            integrations: None,
            hyperparameters: None,
            method: None,
        };

        let job = client.fine_tuning().create_job(params).await?;
        println!("Created job: {} (status: {:?})", job.id, job.status);
    }

    Ok(())
}
