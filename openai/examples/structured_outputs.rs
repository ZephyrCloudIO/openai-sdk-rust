#![allow(missing_docs)]
// Structured outputs example.
//
// Uses the chat completions API with a json_schema response format to
// extract structured data from a prompt.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example structured_outputs

use openai::{
    chat::{
        ChatCompletionCreateParams, ChatCompletionMessageParam, ChatCompletionUserMessageContent,
        ResponseFormat, ResponseFormatJsonSchemaDefinition,
    },
    shared::ModelId,
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    // Define a JSON Schema for the structured output.
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "language": { "type": "string" },
            "year_created": { "type": "integer" }
        },
        "required": ["name", "language", "year_created"],
        "additionalProperties": false
    });

    let params = ChatCompletionCreateParams {
        model: ModelId::from("gpt-4o-mini"),
        messages: vec![ChatCompletionMessageParam::User {
            content: ChatCompletionUserMessageContent::Text(
                "Give me information about the Rust programming language. \
                 Return a JSON object with name, language family, and year_created."
                    .to_owned(),
            ),
            name: None,
        }],
        response_format: Some(ResponseFormat::JsonSchema {
            json_schema: ResponseFormatJsonSchemaDefinition {
                name: "language_info".to_owned(),
                description: Some("Information about a programming language".to_owned()),
                schema: Some(schema),
                strict: Some(true),
            },
        }),
        ..Default::default()
    };

    let completion = client.chat().completions().create(params).await?;

    for choice in &completion.choices {
        if let Some(content) = &choice.message.content {
            println!("Structured output:");
            // Pretty-print the JSON
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
                println!("{}", serde_json::to_string_pretty(&parsed)?);
            } else {
                println!("{content}");
            }
        }
    }

    Ok(())
}
