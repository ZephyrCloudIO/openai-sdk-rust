#![allow(missing_docs)]
// Basic chat completion example.
//
// Creates a simple chat completion request and prints the assistant's response.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example chat_completion

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

    let completion = client.chat().completions().create(params).await?;

    for choice in &completion.choices {
        if let Some(content) = &choice.message.content {
            println!("{content}");
        }
    }

    if let Some(usage) = &completion.usage {
        println!(
            "\nTokens used: {} prompt + {} completion = {} total",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );
    }

    Ok(())
}
