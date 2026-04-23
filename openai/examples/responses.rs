#![allow(missing_docs)]
// Basic Responses API example.
//
// Creates a simple model response and prints the output text.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example responses

use openai::{
    responses::{ResponseCreateParams, ResponseInput},
    shared::ModelId,
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    let params = ResponseCreateParams {
        model: ModelId::from("gpt-4o-mini"),
        input: ResponseInput::Text("Write me a haiku about Rust programming.".to_owned()),
        background: None,
        context_management: None,
        conversation: None,
        include: None,
        instructions: None,
        max_output_tokens: None,
        max_tool_calls: None,
        metadata: None,
        parallel_tool_calls: None,
        previous_response_id: None,
        prompt: None,
        prompt_cache_key: None,
        prompt_cache_retention: None,
        reasoning: None,
        safety_identifier: None,
        service_tier: None,
        store: None,
        stream: None,
        stream_options: None,
        temperature: None,
        text: None,
        tool_choice: None,
        tools: None,
        top_logprobs: None,
        top_p: None,
        truncation: None,
        user: None,
        container: None,
    };

    let response = client.responses().create(params).await?;

    println!("Response ID: {}", response.id);
    println!("Status: {:?}", response.status);

    let text = response.output_text();
    if !text.is_empty() {
        println!("{text}");
    }

    if let Some(usage) = &response.usage {
        println!(
            "\nTokens: {} input + {} output = {} total",
            usage.input_tokens, usage.output_tokens, usage.total_tokens
        );
    }

    Ok(())
}
