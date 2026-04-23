#![allow(missing_docs)]
// Streaming chat completion with accumulator example.
//
// Creates a streaming chat completion and uses ChatCompletionAccumulator to
// collect chunks into a full ChatCompletion. Prints delta content as it
// arrives, then prints the final accumulated result with tool calls and usage.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example chat_completion_accumulating

use futures::StreamExt;
use openai::{
    chat::{
        ChatCompletionAccumulator, ChatCompletionCreateParams, ChatCompletionMessageParam,
        ChatCompletionTool, ChatCompletionUserMessageContent, FunctionDefinition,
    },
    shared::ModelId,
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    let question = "Tell me about Greece's largest city.";

    let tools = vec![
        ChatCompletionTool {
            tool_type: "function".to_owned(),
            function: FunctionDefinition {
                name: "get_live_weather".to_owned(),
                description: Some("Get weather at the given location".to_owned()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": { "type": "string" }
                    },
                    "required": ["location"]
                })),
                strict: None,
            },
        },
        ChatCompletionTool {
            tool_type: "function".to_owned(),
            function: FunctionDefinition {
                name: "get_population".to_owned(),
                description: Some("Get population of a given town".to_owned()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "town": { "type": "string" },
                        "nation": { "type": "string" },
                        "rounding": {
                            "type": "integer",
                            "description": "Nearest base 10 to round to, e.g. 1000 or 1000000"
                        }
                    },
                    "required": ["town", "nation"]
                })),
                strict: None,
            },
        },
    ];

    println!("> {question}");
    println!();

    let params = ChatCompletionCreateParams {
        model: ModelId::from("gpt-4o"),
        messages: vec![
            ChatCompletionMessageParam::System {
                content: "Share only a brief description of the place in 50 words. Then immediately make some tool calls and announce them.".to_owned(),
                name: None,
            },
            ChatCompletionMessageParam::User {
                content: ChatCompletionUserMessageContent::Text(question.to_owned()),
                name: None,
            },
        ],
        tools: Some(tools),
        ..Default::default()
    };

    let stream = client.chat().completions().create_stream(params).await?;
    futures::pin_mut!(stream);

    let mut acc = ChatCompletionAccumulator::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        acc.add_chunk(&chunk);

        // Print delta content as it arrives.
        for choice in &chunk.choices {
            if let Some(content) = &choice.delta.content {
                print!("{content}");
            }
        }
    }
    println!();

    // Convert the accumulated chunks into a full ChatCompletion.
    let completion = acc.into_completion();

    // Print any tool calls the model made.
    for choice in &completion.choices {
        if let Some(tool_calls) = &choice.message.tool_calls {
            for tc in tool_calls {
                println!(
                    "Tool call: {} (id: {}) args: {}",
                    tc.function.name, tc.id, tc.function.arguments
                );
            }
        }
        println!("Finish reason: {:?}", choice.finish_reason);
    }

    if let Some(usage) = &completion.usage {
        println!(
            "Tokens: {} prompt + {} completion = {} total",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );
    }

    Ok(())
}
