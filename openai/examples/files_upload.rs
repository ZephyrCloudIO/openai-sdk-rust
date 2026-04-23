#![allow(missing_docs)]
// File upload example.
//
// Uploads a file to the OpenAI API and then lists all uploaded files.
// Provide a file path as the first argument.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example files_upload -- path/to/file.jsonl

use openai::{
    files::{FileCreateParams, FilePurpose, FileUploadPart},
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo run --example files_upload -- <file-path>");
        std::process::exit(1);
    });

    let bytes = std::fs::read(&path)?;
    let filename = std::path::Path::new(&path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let params = FileCreateParams {
        purpose: FilePurpose::FineTune,
        file: FileUploadPart::from_bytes(bytes, filename),
        expires_after: None,
    };

    let file_obj = client.files().create(params).await?;
    println!(
        "Uploaded file: {} (id: {}, {} bytes)",
        file_obj.filename, file_obj.id, file_obj.bytes
    );

    // List all files.
    let file_list = client.files().list(None).await?;
    println!("\nAll files ({} total):", file_list.data.len());
    for f in &file_list.data {
        println!(
            "  {} - {} ({} bytes, purpose: {:?})",
            f.id, f.filename, f.bytes, f.purpose
        );
    }

    Ok(())
}
