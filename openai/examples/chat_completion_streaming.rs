#![allow(missing_docs)]
// Streaming chat completion example.
//
// Creates a chat completion with streaming enabled and prints token fragments
// as they arrive from the API.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example chat_completion_streaming

use futures::StreamExt;
use openai::{
    chat::{
        ChatCompletionCreateParams, ChatCompletionMessageParam, ChatCompletionUserMessageContent,
    },
    shared::ModelId,
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    let params = ChatCompletionCreateParams {
        model: ModelId::from("gpt-4o-mini"),
        messages: vec![ChatCompletionMessageParam::User {
            content: ChatCompletionUserMessageContent::Text(
                "Write me a haiku about Rust programming.".to_owned(),
            ),
            name: None,
        }],
        ..Default::default()
    };

    let mut stream = client.chat().completions().create_stream(params).await?;

    let stream = stream.by_ref();
    futures::pin_mut!(stream);

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        for choice in &chunk.choices {
            if let Some(content) = &choice.delta.content {
                print!("{content}");
            }
        }
    }
    println!();

    Ok(())
}
