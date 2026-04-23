#![allow(missing_docs)]
// Text-to-speech example.
//
// Generates spoken audio from text input and writes the result to an MP3 file.
//
// Usage: OPENAI_API_KEY=sk-... cargo run --example audio_text_to_speech

use openai::{
    audio::{AudioSpeechCreateParams, AudioSpeechVoice, AudioSpeechVoiceParam},
    shared::ModelId,
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::with_api_key(api_key)?;

    let params = AudioSpeechCreateParams {
        model: ModelId::from("tts-1"),
        input: "Hello! This is the OpenAI Rust SDK generating speech from text.".to_owned(),
        voice: AudioSpeechVoiceParam::BuiltIn(AudioSpeechVoice::Alloy),
        response_format: None,
        instructions: None,
        speed: None,
        stream_format: None,
    };

    let audio_bytes = client.audio().speech(params).await?;

    let output_path = "output_speech.mp3";
    std::fs::write(output_path, &audio_bytes)?;

    println!(
        "Audio written to {output_path} ({} bytes)",
        audio_bytes.len()
    );

    Ok(())
}
