#![allow(missing_docs)]
// Azure OpenAI example.
//
// Demonstrates configuring the client for Azure OpenAI Service and making
// a chat completion request through an Azure deployment.
//
// Usage:
//   AZURE_OPENAI_ENDPOINT=https://your-resource.openai.azure.com \
//   AZURE_OPENAI_API_KEY=your-key \
//   AZURE_OPENAI_API_VERSION=2025-03-01-preview \
//   cargo run --example azure

use openai::{
    chat::{
        ChatCompletionCreateParams, ChatCompletionMessageParam, ChatCompletionUserMessageContent,
    },
    shared::ModelId,
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endpoint =
        std::env::var("AZURE_OPENAI_ENDPOINT").expect("AZURE_OPENAI_ENDPOINT must be set");
    let api_key = std::env::var("AZURE_OPENAI_API_KEY").expect("AZURE_OPENAI_API_KEY must be set");
    let api_version = std::env::var("AZURE_OPENAI_API_VERSION")
        .unwrap_or_else(|_| "2025-03-01-preview".to_owned());

    // Configure the client for Azure using the openai-azure helper.
    let config = openai_azure::with_api_key(&endpoint, &api_version, &api_key);
    let client = Client::new(config)?;

    // Use your Azure deployment name as the model ID (e.g. "gpt-4o").
    let deployment_name = "gpt-4o";

    let params = ChatCompletionCreateParams {
        model: ModelId::from(deployment_name),
        messages: vec![ChatCompletionMessageParam::User {
            content: ChatCompletionUserMessageContent::Text(
                "Write me a haiku about computers".to_owned(),
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

    Ok(())
}
