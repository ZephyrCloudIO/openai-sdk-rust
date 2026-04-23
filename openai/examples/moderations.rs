#![allow(missing_docs)]
// Moderations example.
//
// Runs a moderation check on input text and prints the flagging results.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example moderations

use openai::{
    moderations::{ModerationCreateParams, ModerationInput},
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    let params = ModerationCreateParams {
        model: None,
        input: ModerationInput::from("This is a perfectly normal sentence about programming."),
    };

    let response = client.moderations().create(params).await?;

    println!("Model: {}", response.model);
    for (i, result) in response.results.iter().enumerate() {
        println!("Result {}: flagged = {}", i, result.flagged);
        println!(
            "  harassment: {}, hate: {}, violence: {}, sexual: {}",
            result.categories.harassment,
            result.categories.hate,
            result.categories.violence,
            result.categories.sexual
        );
        println!(
            "  Scores: harassment={:.4}, violence={:.4}, sexual={:.4}",
            result.category_scores.harassment,
            result.category_scores.violence,
            result.category_scores.sexual,
        );
    }

    Ok(())
}
