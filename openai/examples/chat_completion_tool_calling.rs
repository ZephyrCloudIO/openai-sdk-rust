#![allow(missing_docs)]
// Chat completion tool calling example.
//
// Demonstrates how to define a function tool and handle tool calls from the
// model in a multi-turn conversation.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example chat_completion_tool_calling

use openai::{
    chat::{
        ChatCompletionCreateParams, ChatCompletionMessageParam, ChatCompletionTool,
        ChatCompletionUserMessageContent, FunctionDefinition,
    },
    shared::ModelId,
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    // Define a function tool that the model can call.
    let weather_tool = ChatCompletionTool {
        tool_type: "function".to_owned(),
        function: FunctionDefinition {
            name: "get_weather".to_owned(),
            description: Some("Get the current weather for a given location.".to_owned()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "City name, e.g. San Francisco"
                    }
                },
                "required": ["location"]
            })),
            strict: None,
        },
    };

    let params = ChatCompletionCreateParams {
        model: ModelId::from("gpt-4o-mini"),
        messages: vec![ChatCompletionMessageParam::User {
            content: ChatCompletionUserMessageContent::Text(
                "What is the weather like in San Francisco?".to_owned(),
            ),
            name: None,
        }],
        tools: Some(vec![weather_tool]),
        ..Default::default()
    };

    let completion = client.chat().completions().create(params).await?;

    for choice in &completion.choices {
        // Check if the model wants to call a tool.
        if let Some(tool_calls) = &choice.message.tool_calls {
            for tool_call in tool_calls {
                println!(
                    "Tool call: {} (id: {})",
                    tool_call.function.name, tool_call.id
                );
                println!("Arguments: {}", tool_call.function.arguments);
            }
        }

        if let Some(content) = &choice.message.content {
            println!("Response: {content}");
        }

        println!("Finish reason: {:?}", choice.finish_reason);
    }

    Ok(())
}
