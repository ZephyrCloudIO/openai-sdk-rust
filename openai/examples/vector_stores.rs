#![allow(missing_docs)]
// Vector stores example.
//
// Creates a vector store, lists all vector stores, then deletes the created one.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example vector_stores

use openai::{vector_stores::VectorStoreCreateParams, Client};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    // Create a vector store.
    let params = VectorStoreCreateParams {
        name: Some("example-vector-store".to_owned()),
        file_ids: None,
        metadata: None,
    };

    let store = client.vector_stores().create(params).await?;
    println!(
        "Created vector store: {} (id: {}, status: {:?})",
        store.name.as_deref().unwrap_or("unnamed"),
        store.id,
        store
            .status
            .map(|s| format!("{:?}", s))
            .unwrap_or("unknown".to_owned())
    );

    // List all vector stores.
    let list = client.vector_stores().list(None).await?;
    println!("\nAll vector stores ({} on this page):", list.data.len());
    for vs in &list.data {
        println!("  {} - {}", vs.id, vs.name.as_deref().unwrap_or("unnamed"));
    }

    // Clean up: delete the created store.
    let deleted = client.vector_stores().delete(&store.id).await?;
    println!("\nDeleted vector store {}: {}", deleted.id, deleted.deleted);

    Ok(())
}
