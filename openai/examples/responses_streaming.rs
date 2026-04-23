#![allow(missing_docs)]
// Streaming Responses API example.
//
// Creates a streaming model response and prints token fragments as they arrive.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example responses_streaming

use futures::StreamExt;
use openai::{
    responses::{ResponseCreateParams, ResponseInput, ResponseStreamEvent},
    shared::ModelId,
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    let params = ResponseCreateParams {
        model: ModelId::from("gpt-4o-mini"),
        input: ResponseInput::Text("Explain the Rust borrow checker in one paragraph.".to_owned()),
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

    let stream = client.responses().create_stream(params).await?;
    futures::pin_mut!(stream);

    while let Some(event_result) = stream.next().await {
        let event = event_result?;
        match event {
            ResponseStreamEvent::ResponseCreated { response, .. } => {
                println!("Response created: {}", response.id);
            }
            _ => {
                // Other events (deltas, completions, etc.) are handled here.
                // For brevity, we only print the created event.
            }
        }
    }

    println!("\n[stream complete]");

    Ok(())
}
