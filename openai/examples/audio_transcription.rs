#![allow(missing_docs)]
// Audio transcription example.
//
// Transcribes an audio file using the Whisper model. Provide a path to an
// audio file as the first argument, or it will use a small placeholder.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example audio_transcription -- path/to/audio.mp3

use openai::{
    audio::{AudioInputFile, AudioTranscriptionCreateParams},
    shared::ModelId,
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    // Read audio file from first argument, or print usage.
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo run --example audio_transcription -- <audio-file>");
        eprintln!("Supported formats: mp3, mp4, mpeg, mpga, m4a, wav, webm");
        std::process::exit(1);
    });

    let bytes = std::fs::read(&path)?;
    let filename = std::path::Path::new(&path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let params = AudioTranscriptionCreateParams {
        file: AudioInputFile::from_bytes(bytes, filename),
        model: ModelId::from("whisper-1"),
        prompt: None,
        language: None,
        temperature: None,
        response_format: None,
        include: None,
        chunking_strategy: None,
        timestamp_granularities: None,
        known_speaker_names: None,
        known_speaker_references: None,
    };

    let transcription = client.audio().transcribe(params).await?;

    println!("Transcription:\n{}", transcription.text);

    Ok(())
}
