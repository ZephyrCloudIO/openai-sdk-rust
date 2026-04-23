//! Audio APIs.

use futures::Stream;
use reqwest::multipart::{Form, Part};

use crate::{shared::ModelId, ssestream::SseStream, Client, Result};

// ---------------------------------------------------------------------------
// Audio model enums
// ---------------------------------------------------------------------------

/// Audio transcription model identifiers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AudioModel {
    /// Whisper V2 model.
    #[serde(rename = "whisper-1")]
    Whisper1,
    /// GPT-4o transcribe model.
    #[serde(rename = "gpt-4o-transcribe")]
    Gpt4oTranscribe,
    /// GPT-4o mini transcribe model.
    #[serde(rename = "gpt-4o-mini-transcribe")]
    Gpt4oMiniTranscribe,
    /// GPT-4o mini transcribe 2025-12-15 snapshot.
    #[serde(rename = "gpt-4o-mini-transcribe-2025-12-15")]
    Gpt4oMiniTranscribe20251215,
    /// GPT-4o transcribe diarize model.
    #[serde(rename = "gpt-4o-transcribe-diarize")]
    Gpt4oTranscribeDiarize,
}

impl std::fmt::Display for AudioModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Whisper1 => write!(f, "whisper-1"),
            Self::Gpt4oTranscribe => write!(f, "gpt-4o-transcribe"),
            Self::Gpt4oMiniTranscribe => write!(f, "gpt-4o-mini-transcribe"),
            Self::Gpt4oMiniTranscribe20251215 => write!(f, "gpt-4o-mini-transcribe-2025-12-15"),
            Self::Gpt4oTranscribeDiarize => write!(f, "gpt-4o-transcribe-diarize"),
        }
    }
}

/// Speech model identifiers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpeechModel {
    /// Standard TTS model.
    #[serde(rename = "tts-1")]
    Tts1,
    /// High-definition TTS model.
    #[serde(rename = "tts-1-hd")]
    Tts1Hd,
    /// GPT-4o mini TTS model.
    #[serde(rename = "gpt-4o-mini-tts")]
    Gpt4oMiniTts,
    /// GPT-4o mini TTS 2025-12-15 snapshot.
    #[serde(rename = "gpt-4o-mini-tts-2025-12-15")]
    Gpt4oMiniTts20251215,
}

impl std::fmt::Display for SpeechModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tts1 => write!(f, "tts-1"),
            Self::Tts1Hd => write!(f, "tts-1-hd"),
            Self::Gpt4oMiniTts => write!(f, "gpt-4o-mini-tts"),
            Self::Gpt4oMiniTts20251215 => write!(f, "gpt-4o-mini-tts-2025-12-15"),
        }
    }
}

// ---------------------------------------------------------------------------
// Voice and format enums
// ---------------------------------------------------------------------------

/// Built-in voice options for speech generation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioSpeechVoice {
    /// Alloy voice.
    Alloy,
    /// Ash voice.
    Ash,
    /// Ballad voice.
    Ballad,
    /// Coral voice.
    Coral,
    /// Echo voice.
    Echo,
    /// Fable voice.
    Fable,
    /// Onyx voice.
    Onyx,
    /// Nova voice.
    Nova,
    /// Sage voice.
    Sage,
    /// Shimmer voice.
    Shimmer,
    /// Verse voice.
    Verse,
    /// Marin voice.
    Marin,
    /// Cedar voice.
    Cedar,
}

impl std::fmt::Display for AudioSpeechVoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        f.write_str(&s)
    }
}

/// Voice parameter: either a built-in voice name or a custom voice ID.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum AudioSpeechVoiceParam {
    /// A built-in voice variant.
    BuiltIn(AudioSpeechVoice),
    /// A custom voice ID string (e.g. from a voice clone).
    Custom(String),
    /// A custom voice reference with an ID field.
    CustomId {
        /// The custom voice ID, e.g. `voice_1234`.
        id: String,
    },
}

/// Audio response format for speech output.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioSpeechResponseFormat {
    /// MP3 format.
    Mp3,
    /// Opus format.
    Opus,
    /// AAC format.
    Aac,
    /// FLAC format.
    Flac,
    /// WAV format.
    Wav,
    /// PCM format.
    Pcm,
}

/// Audio stream format for streaming speech.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioStreamFormat {
    /// Server-Sent Events format (not supported for tts-1 or tts-1-hd).
    Sse,
    /// Raw audio format.
    Audio,
}

/// Response format for transcription/translation output.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioResponseFormat {
    /// JSON format.
    Json,
    /// Plain text format.
    Text,
    /// SubRip Subtitle format.
    Srt,
    /// Verbose JSON with segments and words.
    VerboseJson,
    /// WebVTT format.
    Vtt,
    /// Diarized JSON with speaker annotations.
    DiarizedJson,
}

impl std::fmt::Display for AudioResponseFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Text => write!(f, "text"),
            Self::Srt => write!(f, "srt"),
            Self::VerboseJson => write!(f, "verbose_json"),
            Self::Vtt => write!(f, "vtt"),
            Self::DiarizedJson => write!(f, "diarized_json"),
        }
    }
}

/// Translation-specific response format.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioTranslationResponseFormat {
    /// JSON format.
    Json,
    /// Plain text format.
    Text,
    /// SubRip Subtitle format.
    Srt,
    /// Verbose JSON with segments and words.
    VerboseJson,
    /// WebVTT format.
    Vtt,
}

impl std::fmt::Display for AudioTranslationResponseFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Text => write!(f, "text"),
            Self::Srt => write!(f, "srt"),
            Self::VerboseJson => write!(f, "verbose_json"),
            Self::Vtt => write!(f, "vtt"),
        }
    }
}

/// Items that can be included in the transcription response.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TranscriptionInclude {
    /// Include log probabilities.
    Logprobs,
}

impl std::fmt::Display for TranscriptionInclude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Logprobs => write!(f, "logprobs"),
        }
    }
}

// ---------------------------------------------------------------------------
// Chunking strategy
// ---------------------------------------------------------------------------

/// Chunking strategy for audio transcription.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ChunkingStrategy {
    /// Automatic chunking with VAD.
    Auto(String),
    /// Manual VAD configuration.
    VadConfig(VadConfig),
}

impl ChunkingStrategy {
    /// Create an automatic chunking strategy.
    #[must_use]
    pub fn auto() -> Self {
        Self::Auto("auto".to_owned())
    }

    /// Create a VAD-based chunking strategy.
    #[must_use]
    pub fn vad(config: VadConfig) -> Self {
        Self::VadConfig(config)
    }
}

/// VAD (Voice Activity Detection) configuration for chunking.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VadConfig {
    /// Must be `"server_vad"`.
    #[serde(rename = "type")]
    pub config_type: String,
    /// Amount of audio to include before VAD detected speech (in milliseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: Option<i64>,
    /// Duration of silence to detect speech stop (in milliseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: Option<i64>,
    /// Sensitivity threshold (0.0 to 1.0) for voice activity detection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            config_type: "server_vad".to_owned(),
            prefix_padding_ms: None,
            silence_duration_ms: None,
            threshold: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Verbose transcription types
// ---------------------------------------------------------------------------

/// Verbose transcription response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionVerbose {
    /// The duration of the input audio.
    pub duration: f64,
    /// The language of the input audio.
    pub language: String,
    /// The transcribed text.
    pub text: String,
    /// Segments of the transcribed text and their corresponding details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<Vec<TranscriptionSegment>>,
    /// Extracted words and their corresponding timestamps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub words: Option<Vec<TranscriptionWord>>,
    /// Usage statistics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TranscriptionVerboseUsage>,
}

/// Usage statistics for verbose transcription (billed by audio duration).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionVerboseUsage {
    /// Duration of the input audio in seconds.
    pub seconds: f64,
    /// The type of the usage object. Always `"duration"`.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub usage_type: Option<String>,
}

/// A segment of transcribed text with timing and quality details.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionSegment {
    /// Unique identifier of the segment.
    pub id: i64,
    /// Average logprob of the segment.
    pub avg_logprob: f64,
    /// Compression ratio of the segment.
    pub compression_ratio: f64,
    /// End time of the segment in seconds.
    pub end: f64,
    /// Probability of no speech in the segment.
    pub no_speech_prob: f64,
    /// Seek offset of the segment.
    pub seek: i64,
    /// Start time of the segment in seconds.
    pub start: f64,
    /// Temperature parameter used for generating the segment.
    pub temperature: f64,
    /// Text content of the segment.
    pub text: String,
    /// Array of token IDs for the text content.
    pub tokens: Vec<i64>,
}

/// A single transcribed word with timing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionWord {
    /// End time of the word in seconds.
    pub end: f64,
    /// Start time of the word in seconds.
    pub start: f64,
    /// The text content of the word.
    pub word: String,
}

/// Log probability for a transcription token.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionLogprob {
    /// The token in the transcription.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// The bytes of the token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<f64>>,
    /// The log probability of the token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprob: Option<f64>,
}

// ---------------------------------------------------------------------------
// Transcription usage union
// ---------------------------------------------------------------------------

/// Transcription usage: either token-based or duration-based.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum TranscriptionUsage {
    /// Token-based usage.
    #[serde(rename = "tokens")]
    Tokens(TranscriptionUsageTokens),
    /// Duration-based usage.
    #[serde(rename = "duration")]
    Duration(TranscriptionUsageDuration),
}

/// Usage statistics for models billed by token usage.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionUsageTokens {
    /// Number of input tokens billed for this request.
    pub input_tokens: i64,
    /// Number of output tokens generated.
    pub output_tokens: i64,
    /// Total number of tokens used (input + output).
    pub total_tokens: i64,
    /// Details about the input tokens billed for this request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_token_details: Option<TranscriptionUsageTokensInputTokenDetails>,
}

/// Details about input tokens for transcription.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionUsageTokensInputTokenDetails {
    /// Number of audio tokens billed for this request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<i64>,
    /// Number of text tokens billed for this request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_tokens: Option<i64>,
}

/// Usage statistics for models billed by audio input duration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionUsageDuration {
    /// Duration of the input audio in seconds.
    pub seconds: f64,
}

// ---------------------------------------------------------------------------
// Transcription response union
// ---------------------------------------------------------------------------

/// Transcription response: either a simple or verbose result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum AudioTranscriptionResponse {
    /// Verbose transcription with duration, language, segments, and words.
    Verbose(TranscriptionVerbose),
    /// Simple transcription with just text.
    Simple(AudioTranscription),
}

// ---------------------------------------------------------------------------
// Transcription streaming event types
// ---------------------------------------------------------------------------

/// Streaming transcription event types.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum TranscriptionStreamEvent {
    /// A text segment event (diarized transcription).
    #[serde(rename = "transcript.text.segment")]
    TextSegment(TranscriptionTextSegmentEvent),
    /// A text delta event.
    #[serde(rename = "transcript.text.delta")]
    TextDelta(TranscriptionTextDeltaEvent),
    /// A text done event.
    #[serde(rename = "transcript.text.done")]
    TextDone(TranscriptionTextDoneEvent),
}

/// Emitted when a diarized transcription returns a completed segment.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionTextSegmentEvent {
    /// Unique identifier for the segment.
    pub id: String,
    /// End timestamp of the segment in seconds.
    pub end: f64,
    /// Speaker label for this segment.
    pub speaker: String,
    /// Start timestamp of the segment in seconds.
    pub start: f64,
    /// Transcript text for this segment.
    pub text: String,
}

/// Emitted when there is an additional text delta.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionTextDeltaEvent {
    /// The text delta that was additionally transcribed.
    pub delta: String,
    /// Log probabilities of the delta (if requested via `include`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Vec<TranscriptionStreamLogprob>>,
    /// Identifier of the diarized segment that this delta belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_id: Option<String>,
}

/// Emitted when the transcription is complete.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionTextDoneEvent {
    /// The text that was transcribed.
    pub text: String,
    /// Log probabilities of the transcription (if requested via `include`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Vec<TranscriptionStreamLogprob>>,
    /// Usage statistics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TranscriptionTextDoneEventUsage>,
}

/// Log probability for a streaming transcription token.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionStreamLogprob {
    /// The token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// The bytes of the token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<i64>>,
    /// The log probability of the token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprob: Option<f64>,
}

/// Usage statistics for the text done streaming event.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionTextDoneEventUsage {
    /// Number of input tokens billed for this request.
    pub input_tokens: i64,
    /// Number of output tokens generated.
    pub output_tokens: i64,
    /// Total number of tokens used (input + output).
    pub total_tokens: i64,
    /// Details about the input tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_token_details: Option<TranscriptionUsageTokensInputTokenDetails>,
}

// ---------------------------------------------------------------------------
// Audio service
// ---------------------------------------------------------------------------

/// Audio service root.
#[derive(Clone)]
pub struct AudioService {
    client: Client,
}

impl AudioService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a transcription from an audio file.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn transcribe(
        &self,
        params: AudioTranscriptionCreateParams,
    ) -> Result<AudioTranscription> {
        self.client
            .post_multipart_json("/audio/transcriptions", transcription_form(params, false))
            .await
    }

    /// Creates a streaming transcription from an audio file.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or stream decode failures.
    pub async fn transcribe_streaming(
        &self,
        params: AudioTranscriptionCreateParams,
    ) -> Result<impl Stream<Item = Result<TranscriptionStreamEvent>>> {
        let response = self
            .client
            .post_raw_multipart("/audio/transcriptions", transcription_form(params, true))
            .await?;
        Ok(SseStream::new(response))
    }

    /// Creates an English translation from an audio file.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn translate(
        &self,
        params: AudioTranslationCreateParams,
    ) -> Result<AudioTranslation> {
        let model = params.model.to_string();
        let filename = params.file.filename;
        let bytes = params.file.bytes;
        let content_type = params.file.content_type;
        let prompt = params.prompt;
        let temperature = params.temperature;
        let response_format = params.response_format;

        self.client
            .post_multipart_json("/audio/translations", move || {
                let part = if let Some(content_type) = content_type.as_deref() {
                    Part::bytes(bytes.clone())
                        .file_name(filename.clone())
                        .mime_str(content_type)
                        .unwrap_or_else(|_| Part::bytes(bytes.clone()).file_name(filename.clone()))
                } else {
                    Part::bytes(bytes.clone()).file_name(filename.clone())
                };

                let mut form = Form::new().part("file", part).text("model", model.clone());
                if let Some(prompt) = prompt.as_deref() {
                    form = form.text("prompt", prompt.to_owned());
                }
                if let Some(temperature) = temperature {
                    form = form.text("temperature", temperature.to_string());
                }
                if let Some(ref response_format) = response_format {
                    form = form.text("response_format", response_format.to_string());
                }
                form
            })
            .await
    }

    /// Generates spoken audio bytes from text input.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn speech(&self, params: AudioSpeechCreateParams) -> Result<Vec<u8>> {
        self.client.post_json_bytes("/audio/speech", &params).await
    }
}

fn transcription_form(params: AudioTranscriptionCreateParams, stream: bool) -> impl Fn() -> Form {
    let model = params.model.to_string();
    let filename = params.file.filename;
    let bytes = params.file.bytes;
    let content_type = params.file.content_type;
    let prompt = params.prompt;
    let language = params.language;
    let temperature = params.temperature;
    let response_format = params.response_format;
    let include = params.include;
    let chunking_strategy = params.chunking_strategy;
    let timestamp_granularities = params.timestamp_granularities;
    let known_speaker_names = params.known_speaker_names;
    let known_speaker_references = params.known_speaker_references;

    move || {
        let part = if let Some(content_type) = content_type.as_deref() {
            Part::bytes(bytes.clone())
                .file_name(filename.clone())
                .mime_str(content_type)
                .unwrap_or_else(|_| Part::bytes(bytes.clone()).file_name(filename.clone()))
        } else {
            Part::bytes(bytes.clone()).file_name(filename.clone())
        };

        let mut form = Form::new().part("file", part).text("model", model.clone());
        if stream {
            form = form.text("stream", "true");
        }
        if let Some(prompt) = prompt.as_deref() {
            form = form.text("prompt", prompt.to_owned());
        }
        if let Some(language) = language.as_deref() {
            form = form.text("language", language.to_owned());
        }
        if let Some(temperature) = temperature {
            form = form.text("temperature", temperature.to_string());
        }
        if let Some(ref response_format) = response_format {
            form = form.text("response_format", response_format.to_string());
        }
        if let Some(ref include_items) = include {
            for item in include_items {
                form = form.text("include[]", item.to_string());
            }
        }
        if let Some(ref strategy) = chunking_strategy {
            if let Ok(json) = serde_json::to_string(strategy) {
                form = form.text("chunking_strategy", json);
            }
        }
        if let Some(ref granularities) = timestamp_granularities {
            for g in granularities {
                form = form.text("timestamp_granularities[]", g.clone());
            }
        }
        if let Some(ref names) = known_speaker_names {
            for name in names {
                form = form.text("known_speaker_names[]", name.clone());
            }
        }
        if let Some(ref refs) = known_speaker_references {
            for r in refs {
                form = form.text("known_speaker_references[]", r.clone());
            }
        }
        form
    }
}

// ---------------------------------------------------------------------------
// In-memory audio file input
// ---------------------------------------------------------------------------

/// In-memory audio file input.
#[derive(Debug, Clone)]
pub struct AudioInputFile {
    /// Audio bytes.
    pub bytes: Vec<u8>,
    /// Filename provided to API.
    pub filename: String,
    /// Optional MIME type.
    pub content_type: Option<String>,
}

impl AudioInputFile {
    /// Creates an input file from bytes.
    #[must_use]
    pub fn from_bytes(bytes: impl Into<Vec<u8>>, filename: impl Into<String>) -> Self {
        Self {
            bytes: bytes.into(),
            filename: filename.into(),
            content_type: None,
        }
    }

    /// Sets MIME type.
    #[must_use]
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Request params
// ---------------------------------------------------------------------------

/// Audio transcription request.
#[derive(Debug, Clone)]
pub struct AudioTranscriptionCreateParams {
    /// Audio input file.
    pub file: AudioInputFile,
    /// Model ID.
    pub model: ModelId,
    /// Optional prompt.
    pub prompt: Option<String>,
    /// Optional source language hint (ISO-639-1).
    pub language: Option<String>,
    /// Optional sampling temperature (0 to 1).
    pub temperature: Option<f32>,
    /// Output format: json, text, srt, verbose_json, vtt, or diarized_json.
    pub response_format: Option<AudioResponseFormat>,
    /// Additional information to include (e.g. logprobs).
    pub include: Option<Vec<TranscriptionInclude>>,
    /// Chunking strategy for long audio.
    pub chunking_strategy: Option<ChunkingStrategy>,
    /// Timestamp granularities: `"word"` and/or `"segment"`.
    pub timestamp_granularities: Option<Vec<String>>,
    /// Optional list of speaker names for diarization.
    pub known_speaker_names: Option<Vec<String>>,
    /// Optional list of audio samples for speaker identification (data URLs).
    pub known_speaker_references: Option<Vec<String>>,
}

/// Audio translation request.
#[derive(Debug, Clone)]
pub struct AudioTranslationCreateParams {
    /// Audio input file.
    pub file: AudioInputFile,
    /// Model ID.
    pub model: ModelId,
    /// Optional prompt.
    pub prompt: Option<String>,
    /// Optional sampling temperature.
    pub temperature: Option<f32>,
    /// Output format: json, text, srt, verbose_json, or vtt.
    pub response_format: Option<AudioTranslationResponseFormat>,
}

/// Audio speech generation request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioSpeechCreateParams {
    /// Model ID.
    pub model: ModelId,
    /// Input text.
    pub input: String,
    /// Voice name (built-in string or custom voice).
    pub voice: AudioSpeechVoiceParam,
    /// Optional output response format (mp3, opus, aac, flac, wav, pcm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<AudioSpeechResponseFormat>,
    /// Control the voice with additional instructions. Does not work with tts-1 or tts-1-hd.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// The speed of the generated audio. Select a value from 0.25 to 4.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    /// The format to stream the audio in: `sse` or `audio`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_format: Option<AudioStreamFormat>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Transcription response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioTranscription {
    /// Decoded text.
    pub text: String,
    /// Log probabilities (only with gpt-4o-transcribe models when logprobs are requested).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Vec<TranscriptionLogprob>>,
    /// Token usage statistics for the request.
    ///
    /// This is a union type: either token-based usage (for GPT-4o transcribe
    /// models) or duration-based usage (for Whisper models).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TranscriptionUsage>,
}

/// Translation response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioTranslation {
    /// Decoded text.
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ModelId;

    #[test]
    fn speech_params_serialize_with_voice_and_optional_fields() {
        let params = AudioSpeechCreateParams {
            model: ModelId::from("gpt-4o-mini-tts"),
            input: "hello".to_owned(),
            voice: AudioSpeechVoiceParam::BuiltIn(AudioSpeechVoice::Alloy),
            response_format: None,
            instructions: None,
            speed: None,
            stream_format: None,
        };

        let value = serde_json::to_value(&params).expect("serialize speech params");
        assert_eq!(
            value.get("model"),
            Some(&serde_json::Value::String("gpt-4o-mini-tts".to_owned()))
        );
        assert_eq!(
            value.get("voice"),
            Some(&serde_json::Value::String("alloy".to_owned()))
        );
        assert!(value.get("response_format").is_none());
        assert!(value.get("instructions").is_none());
        assert!(value.get("speed").is_none());
        assert!(value.get("stream_format").is_none());
    }

    #[test]
    fn speech_params_serialize_with_all_optional_fields() {
        let params = AudioSpeechCreateParams {
            model: ModelId::from("tts-1"),
            input: "hi".to_owned(),
            voice: AudioSpeechVoiceParam::BuiltIn(AudioSpeechVoice::Coral),
            response_format: Some(AudioSpeechResponseFormat::Opus),
            instructions: Some("Speak slowly".to_owned()),
            speed: Some(0.5),
            stream_format: Some(AudioStreamFormat::Sse),
        };

        let value = serde_json::to_value(&params).expect("serialize speech params");
        assert_eq!(
            value.get("response_format"),
            Some(&serde_json::Value::String("opus".to_owned()))
        );
        assert_eq!(
            value.get("instructions"),
            Some(&serde_json::Value::String("Speak slowly".to_owned()))
        );
        assert_eq!(value.get("speed"), Some(&serde_json::json!(0.5)));
        assert_eq!(
            value.get("stream_format"),
            Some(&serde_json::Value::String("sse".to_owned()))
        );
    }

    #[test]
    fn audio_input_file_can_set_content_type() {
        let file = AudioInputFile::from_bytes(vec![0_u8, 1, 2], "audio.wav")
            .with_content_type("audio/wav");
        assert_eq!(file.content_type.as_deref(), Some("audio/wav"));
    }

    #[test]
    fn audio_model_serialization() {
        let model = AudioModel::Whisper1;
        let json = serde_json::to_string(&model).expect("serialize audio model");
        assert_eq!(json, "\"whisper-1\"");

        let model = AudioModel::Gpt4oTranscribeDiarize;
        let json = serde_json::to_string(&model).expect("serialize audio model");
        assert_eq!(json, "\"gpt-4o-transcribe-diarize\"");
    }

    #[test]
    fn speech_model_serialization() {
        let model = SpeechModel::Tts1;
        let json = serde_json::to_string(&model).expect("serialize speech model");
        assert_eq!(json, "\"tts-1\"");

        let model = SpeechModel::Gpt4oMiniTts;
        let json = serde_json::to_string(&model).expect("serialize speech model");
        assert_eq!(json, "\"gpt-4o-mini-tts\"");
    }

    #[test]
    fn audio_response_format_display() {
        assert_eq!(AudioResponseFormat::VerboseJson.to_string(), "verbose_json");
        assert_eq!(
            AudioResponseFormat::DiarizedJson.to_string(),
            "diarized_json"
        );
    }

    #[test]
    fn voice_enum_variants() {
        let voices = vec![
            AudioSpeechVoice::Alloy,
            AudioSpeechVoice::Ash,
            AudioSpeechVoice::Ballad,
            AudioSpeechVoice::Coral,
            AudioSpeechVoice::Echo,
            AudioSpeechVoice::Sage,
            AudioSpeechVoice::Shimmer,
            AudioSpeechVoice::Verse,
            AudioSpeechVoice::Marin,
            AudioSpeechVoice::Cedar,
        ];
        assert_eq!(voices.len(), 10);
    }

    #[test]
    fn chunking_strategy_auto_serialization() {
        let strategy = ChunkingStrategy::auto();
        let json = serde_json::to_value(&strategy).expect("serialize auto strategy");
        assert_eq!(json, serde_json::Value::String("auto".to_owned()));
    }

    #[test]
    fn chunking_strategy_vad_serialization() {
        let strategy = ChunkingStrategy::vad(VadConfig {
            prefix_padding_ms: Some(300),
            silence_duration_ms: Some(500),
            threshold: Some(0.5),
            ..VadConfig::default()
        });
        let json = serde_json::to_value(&strategy).expect("serialize vad strategy");
        assert_eq!(
            json.get("type"),
            Some(&serde_json::Value::String("server_vad".to_owned()))
        );
        assert_eq!(json.get("prefix_padding_ms"), Some(&serde_json::json!(300)));
    }

    #[test]
    fn verbose_transcription_deserialization() {
        let json = r#"{
            "duration": 12.5,
            "language": "en",
            "text": "Hello world",
            "segments": [{
                "id": 0,
                "avg_logprob": -0.5,
                "compression_ratio": 1.2,
                "end": 3.0,
                "no_speech_prob": 0.01,
                "seek": 0,
                "start": 0.0,
                "temperature": 0.0,
                "text": "Hello",
                "tokens": [1, 2, 3]
            }],
            "words": [{"end": 1.0, "start": 0.0, "word": "Hello"}]
        }"#;

        let verbose: TranscriptionVerbose =
            serde_json::from_str(json).expect("deserialize verbose transcription");
        assert!((verbose.duration - 12.5).abs() < f64::EPSILON);
        assert_eq!(verbose.language, "en");
        assert_eq!(verbose.segments.as_ref().map(Vec::len), Some(1));
        assert_eq!(verbose.words.as_ref().map(Vec::len), Some(1));
    }

    #[test]
    fn transcription_stream_event_deserialization() {
        let segment_json = r#"{
            "type": "transcript.text.segment",
            "id": "seg_0",
            "end": 3.5,
            "speaker": "speaker_1",
            "start": 0.5,
            "text": "Hello"
        }"#;

        let event: TranscriptionStreamEvent =
            serde_json::from_str(segment_json).expect("deserialize segment event");
        match event {
            TranscriptionStreamEvent::TextSegment(seg) => {
                assert_eq!(seg.id, "seg_0");
                assert_eq!(seg.speaker, "speaker_1");
            }
            _ => panic!("Expected TextSegment variant"),
        }

        let delta_json = r#"{
            "type": "transcript.text.delta",
            "delta": "world"
        }"#;

        let event: TranscriptionStreamEvent =
            serde_json::from_str(delta_json).expect("deserialize delta event");
        match event {
            TranscriptionStreamEvent::TextDelta(delta) => {
                assert_eq!(delta.delta, "world");
            }
            _ => panic!("Expected TextDelta variant"),
        }

        let done_json = r#"{
            "type": "transcript.text.done",
            "text": "Hello world"
        }"#;

        let event: TranscriptionStreamEvent =
            serde_json::from_str(done_json).expect("deserialize done event");
        match event {
            TranscriptionStreamEvent::TextDone(done) => {
                assert_eq!(done.text, "Hello world");
            }
            _ => panic!("Expected TextDone variant"),
        }
    }

    #[test]
    fn custom_voice_id_serialization() {
        let voice = AudioSpeechVoiceParam::CustomId {
            id: "voice_1234".to_owned(),
        };
        let json = serde_json::to_value(&voice).expect("serialize custom voice id");
        assert_eq!(
            json.get("id"),
            Some(&serde_json::Value::String("voice_1234".to_owned()))
        );
    }

    #[test]
    fn audio_transcription_usage_token_based() {
        let json = r#"{
            "text": "Hello world",
            "usage": {
                "type": "tokens",
                "input_tokens": 50,
                "output_tokens": 30,
                "total_tokens": 80
            }
        }"#;

        let transcription: AudioTranscription =
            serde_json::from_str(json).expect("deserialize transcription with token usage");
        assert_eq!(transcription.text, "Hello world");
        match transcription.usage {
            Some(TranscriptionUsage::Tokens(ref t)) => {
                assert_eq!(t.input_tokens, 50);
                assert_eq!(t.output_tokens, 30);
                assert_eq!(t.total_tokens, 80);
            }
            other => panic!("Expected Tokens usage, got {:?}", other),
        }
    }

    #[test]
    fn audio_transcription_usage_duration_based() {
        let json = r#"{
            "text": "Hello world",
            "usage": {
                "type": "duration",
                "seconds": 12.5
            }
        }"#;

        let transcription: AudioTranscription =
            serde_json::from_str(json).expect("deserialize transcription with duration usage");
        assert_eq!(transcription.text, "Hello world");
        match transcription.usage {
            Some(TranscriptionUsage::Duration(ref d)) => {
                assert!((d.seconds - 12.5).abs() < f64::EPSILON);
            }
            other => panic!("Expected Duration usage, got {:?}", other),
        }
    }

    #[test]
    fn audio_transcription_usage_absent() {
        let json = r#"{"text": "Hello world"}"#;

        let transcription: AudioTranscription =
            serde_json::from_str(json).expect("deserialize transcription without usage");
        assert!(transcription.usage.is_none());
    }
}
