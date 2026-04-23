//! Realtime API (WebSocket) module.
//!
//! Provides a [`RealtimeSession`] for bidirectional communication with the
//! OpenAI Realtime API over WebSockets, plus HTTP-based [`CallService`] and
//! [`ClientSecretService`] sub-services.

use std::pin::Pin;

use futures::stream::Stream;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::config::RequestConfig;
use crate::error::{Error, Result};
use crate::Client;

// ---------------------------------------------------------------------------
// Re-exports
// ---------------------------------------------------------------------------

pub use self::events::*;
pub use self::services::*;

mod events;
mod services;

// ---------------------------------------------------------------------------
// Connection parameters
// ---------------------------------------------------------------------------

/// Parameters for establishing a Realtime WebSocket connection.
#[derive(Debug, Clone, Default)]
pub struct RealtimeConnectParams {
    /// The realtime model to connect to (e.g. `"gpt-4o-realtime-preview"`).
    pub model: Option<String>,
}

impl RealtimeConnectParams {
    /// Creates default connection parameters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the model for the connection.
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

// ---------------------------------------------------------------------------
// RealtimeSession
// ---------------------------------------------------------------------------

/// A live WebSocket session with the OpenAI Realtime API.
///
/// Created via [`RealtimeSession::connect`].
pub struct RealtimeSession {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl RealtimeSession {
    /// Opens a WebSocket connection to the Realtime API.
    ///
    /// # Errors
    /// Returns [`Error::WebSocket`] on connection failure, or [`Error::Config`]
    /// if the URL cannot be constructed.
    pub async fn connect(config: &RequestConfig, params: RealtimeConnectParams) -> Result<Self> {
        let url = Self::build_ws_url(config, &params)?;
        let mut request = url.as_str().into_client_request()?;

        // Inject auth header.
        let bearer_token = if let Some(workload_identity) = &config.workload_identity {
            let http_client = reqwest::Client::builder()
                .timeout(config.timeout)
                .build()
                .map_err(Error::Http)?;
            Some(workload_identity.get_token(&http_client).await?)
        } else {
            config.api_key.clone()
        };
        if let Some(api_key) = bearer_token.as_deref() {
            request.headers_mut().insert(
                header::AUTHORIZATION,
                format!("Bearer {api_key}")
                    .parse()
                    .map_err(|_| Error::Config {
                        field: "api_key",
                        message: "invalid header value".to_owned(),
                    })?,
            );
        }

        // Inject OpenAI-specific headers.
        if let Some(org) = config.organization.as_deref() {
            if let Ok(val) = org.parse() {
                request.headers_mut().insert("OpenAI-Organization", val);
            }
        }
        if let Some(project) = config.project.as_deref() {
            if let Ok(val) = project.parse() {
                request.headers_mut().insert("OpenAI-Project", val);
            }
        }

        // Inject extra headers from config.
        for (key, value) in &config.headers {
            if let (Ok(name), Ok(val)) = (
                key.parse::<tokio_tungstenite::tungstenite::http::HeaderName>(),
                value.parse::<tokio_tungstenite::tungstenite::http::HeaderValue>(),
            ) {
                request.headers_mut().insert(name, val);
            }
        }

        let (ws, _response) = connect_async(request).await?;

        Ok(Self { ws })
    }

    /// Sends a client event to the Realtime API.
    ///
    /// # Errors
    /// Returns [`Error::WebSocket`] on send failure, or [`Error::Json`] on
    /// serialization failure.
    pub async fn send(&mut self, event: ClientEvent) -> Result<()> {
        let json = serde_json::to_string(&event)?;
        self.ws.send(Message::Text(json)).await?;
        Ok(())
    }

    /// Returns a stream of server events received from the Realtime API.
    ///
    /// The stream yields `Result<ServerEvent>` items. It ends when the
    /// connection is closed.
    pub fn incoming(&mut self) -> Pin<Box<dyn Stream<Item = Result<ServerEvent>> + Send + '_>> {
        use futures::StreamExt as _;
        Box::pin(futures::stream::unfold(&mut self.ws, |ws| async move {
            match ws.next().await {
                Some(Ok(Message::Text(text))) => {
                    let event = serde_json::from_str::<ServerEvent>(&text).map_err(Error::from);
                    Some((event, ws))
                }
                Some(Ok(Message::Close(_))) | None => None,
                Some(Ok(_)) => {
                    // Skip binary/ping/pong frames.
                    Some((
                        Err(Error::Stream(
                            "unexpected non-text WebSocket frame".to_owned(),
                        )),
                        ws,
                    ))
                }
                Some(Err(e)) => Some((Err(Error::from(e)), ws)),
            }
        }))
    }

    /// Gracefully closes the WebSocket connection.
    ///
    /// # Errors
    /// Returns [`Error::WebSocket`] if the close handshake fails.
    pub async fn close(mut self) -> Result<()> {
        self.ws
            .close(Some(tokio_tungstenite::tungstenite::protocol::CloseFrame {
                code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Normal,
                reason: std::borrow::Cow::Borrowed("client closing"),
            }))
            .await?;
        Ok(())
    }

    // -- internal helpers --

    /// Constructs the WebSocket URL for the Realtime API.
    ///
    /// This is exposed publicly so callers can inspect the URL that
    /// [`connect`](Self::connect) will use.
    pub fn build_ws_url(
        config: &RequestConfig,
        params: &RealtimeConnectParams,
    ) -> Result<url::Url> {
        // Start from the base URL, replacing scheme.
        let base = config.base_url.as_str();
        let ws_base = if base.starts_with("https://") {
            base.replacen("https://", "wss://", 1)
        } else if base.starts_with("http://") {
            base.replacen("http://", "ws://", 1)
        } else {
            base.to_owned()
        };

        let mut url = url::Url::parse(&ws_base).map_err(|e| Error::Config {
            field: "base_url",
            message: e.to_string(),
        })?;

        // Append "realtime" path segment.
        {
            let mut segments = url.path_segments_mut().map_err(|_| Error::Config {
                field: "base_url",
                message: "cannot-be-a-base URL".to_owned(),
            })?;
            // Remove trailing empty segment from trailing slash.
            segments.pop_if_empty();
            segments.push("realtime");
        }

        // Add model query param if specified.
        if let Some(model) = &params.model {
            url.query_pairs_mut().append_pair("model", model);
        }

        // Add extra query params from config.
        for (key, value) in &config.query_params {
            url.query_pairs_mut().append_pair(key, value);
        }

        Ok(url)
    }
}

// ---------------------------------------------------------------------------
// RealtimeService (namespace on Client)
// ---------------------------------------------------------------------------

/// Namespace for Realtime API services.
///
/// Accessed via [`Client::realtime()`].
#[derive(Clone)]
pub struct RealtimeService {
    client: Client,
}

impl RealtimeService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns the call management sub-service.
    #[must_use]
    pub fn calls(&self) -> CallService {
        CallService::new(self.client.clone())
    }

    /// Returns the client secret sub-service.
    #[must_use]
    pub fn client_secrets(&self) -> ClientSecretService {
        ClientSecretService::new(self.client.clone())
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RequestConfig;

    #[test]
    fn ws_url_default_base() {
        let config = RequestConfig::default();
        let params = RealtimeConnectParams::new().with_model("gpt-4o-realtime-preview");
        let url = RealtimeSession::build_ws_url(&config, &params).expect("build URL");
        assert_eq!(
            url.as_str(),
            "wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview"
        );
    }

    #[test]
    fn ws_url_custom_base() {
        let mut config = RequestConfig::default();
        config.base_url = url::Url::parse("https://custom.example.com/api/v2/").expect("parse URL");
        let params = RealtimeConnectParams::new().with_model("gpt-realtime");
        let url = RealtimeSession::build_ws_url(&config, &params).expect("build URL");
        assert_eq!(
            url.as_str(),
            "wss://custom.example.com/api/v2/realtime?model=gpt-realtime"
        );
    }

    #[test]
    fn ws_url_no_model() {
        let config = RequestConfig::default();
        let params = RealtimeConnectParams::new();
        let url = RealtimeSession::build_ws_url(&config, &params).expect("build URL");
        assert_eq!(url.as_str(), "wss://api.openai.com/v1/realtime");
    }

    #[test]
    fn ws_url_includes_extra_query_params() {
        let mut config = RequestConfig::default();
        config
            .query_params
            .push(("api-version".to_owned(), "2025-01-01".to_owned()));
        let params = RealtimeConnectParams::new().with_model("gpt-realtime");
        let url = RealtimeSession::build_ws_url(&config, &params).expect("build URL");
        assert!(url.as_str().contains("model=gpt-realtime"));
        assert!(url.as_str().contains("api-version=2025-01-01"));
    }

    #[test]
    fn client_event_session_update_roundtrip() {
        let event = ClientEvent::SessionUpdate {
            session: serde_json::json!({ "instructions": "be concise" }),
            event_id: None,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("\"type\":\"session.update\""));
        assert!(json.contains("\"instructions\":\"be concise\""));
        let back: ClientEvent = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(back, ClientEvent::SessionUpdate { .. }));
    }

    #[test]
    fn client_event_input_audio_buffer_append_roundtrip() {
        let event = ClientEvent::InputAudioBufferAppend {
            audio: "base64data".to_owned(),
            event_id: None,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("\"type\":\"input_audio_buffer.append\""));
        assert!(json.contains("\"audio\":\"base64data\""));
        let back: ClientEvent = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(back, ClientEvent::InputAudioBufferAppend { .. }));
    }

    #[test]
    fn client_event_response_create_roundtrip() {
        let event = ClientEvent::ResponseCreate {
            response: Some(serde_json::json!({ "modalities": ["text"] })),
            event_id: None,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("\"type\":\"response.create\""));
        let back: ClientEvent = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(back, ClientEvent::ResponseCreate { .. }));
    }

    #[test]
    fn client_event_conversation_item_create_roundtrip() {
        let event = ClientEvent::ConversationItemCreate {
            item: serde_json::json!({
                "type": "message",
                "role": "user",
                "content": [{ "type": "input_text", "text": "hello" }]
            }),
            event_id: None,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("\"type\":\"conversation.item.create\""));
        let back: ClientEvent = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(back, ClientEvent::ConversationItemCreate { .. }));
    }

    #[test]
    fn server_event_session_created_deserialize() {
        let json = r#"{
            "type": "session.created",
            "event_id": "evt_1",
            "session": {"id": "sess_1", "model": "gpt-4o-realtime-preview"}
        }"#;
        let event: ServerEvent = serde_json::from_str(json).expect("deserialize");
        assert!(matches!(event, ServerEvent::SessionCreated { .. }));
        if let ServerEvent::SessionCreated {
            event_id, session, ..
        } = event
        {
            assert_eq!(event_id, Some("evt_1".to_owned()));
            assert_eq!(session["id"], "sess_1");
        }
    }

    #[test]
    fn server_event_response_text_delta_deserialize() {
        let json = r#"{
            "type": "response.text.delta",
            "event_id": "evt_2",
            "response_id": "resp_1",
            "item_id": "item_1",
            "output_index": 0,
            "content_index": 0,
            "delta": "Hello"
        }"#;
        let event: ServerEvent = serde_json::from_str(json).expect("deserialize");
        assert!(matches!(event, ServerEvent::ResponseTextDelta { .. }));
    }

    #[test]
    fn server_event_error_deserialize() {
        let json = r#"{
            "type": "error",
            "event_id": "evt_err",
            "error": {"type": "invalid_request", "message": "bad input"}
        }"#;
        let event: ServerEvent = serde_json::from_str(json).expect("deserialize");
        assert!(matches!(event, ServerEvent::Error { .. }));
    }

    #[test]
    fn server_event_rate_limits_updated_deserialize() {
        let json = r#"{
            "type": "rate_limits.updated",
            "event_id": "evt_rl",
            "rate_limits": [
                {"name": "requests", "limit": 100, "remaining": 99, "reset_seconds": 60.0}
            ]
        }"#;
        let event: ServerEvent = serde_json::from_str(json).expect("deserialize");
        assert!(matches!(event, ServerEvent::RateLimitsUpdated { .. }));
    }

    #[test]
    fn server_event_response_audio_delta_deserialize() {
        let json = r#"{
            "type": "response.audio.delta",
            "event_id": "evt_a",
            "response_id": "resp_1",
            "item_id": "item_1",
            "output_index": 0,
            "content_index": 0,
            "delta": "base64audio=="
        }"#;
        let event: ServerEvent = serde_json::from_str(json).expect("deserialize");
        assert!(matches!(event, ServerEvent::ResponseAudioDelta { .. }));
    }

    #[test]
    fn server_event_function_call_arguments_delta_deserialize() {
        let json = r#"{
            "type": "response.function_call_arguments.delta",
            "event_id": "evt_f",
            "response_id": "resp_1",
            "item_id": "item_1",
            "output_index": 0,
            "call_id": "call_1",
            "delta": "{\"loc\":"
        }"#;
        let event: ServerEvent = serde_json::from_str(json).expect("deserialize");
        assert!(matches!(
            event,
            ServerEvent::ResponseFunctionCallArgumentsDelta { .. }
        ));
    }

    #[test]
    fn client_event_all_simple_variants_serialize() {
        // Ensure all simple client events produce the right type tag.
        let cases: Vec<(ClientEvent, &str)> = vec![
            (
                ClientEvent::InputAudioBufferCommit { event_id: None },
                "input_audio_buffer.commit",
            ),
            (
                ClientEvent::InputAudioBufferClear { event_id: None },
                "input_audio_buffer.clear",
            ),
            (
                ClientEvent::ResponseCancel { event_id: None },
                "response.cancel",
            ),
        ];

        for (event, expected_type) in cases {
            let json = serde_json::to_string(&event).expect("serialize");
            let expected = format!("\"type\":\"{}\"", expected_type);
            assert!(json.contains(&expected), "Expected {expected} in: {json}");
        }
    }

    #[test]
    fn connect_params_builder() {
        let params = RealtimeConnectParams::new().with_model("gpt-realtime");
        assert_eq!(params.model.as_deref(), Some("gpt-realtime"));
    }

    #[test]
    fn client_secret_create_params_serialize() {
        let session_config =
            RealtimeSessionCreateRequestParam::new().with_model("gpt-4o-realtime-preview");
        let params = ClientSecretCreateParams {
            session: Some(ClientSecretSessionConfig::Realtime(session_config)),
            expires_after: None,
        };
        let json = serde_json::to_value(&params).expect("serialize");
        assert_eq!(json["session"]["model"], "gpt-4o-realtime-preview");
        assert_eq!(json["session"]["type"], "realtime");
        assert!(json.get("expires_after").is_none());
    }

    #[test]
    fn client_secret_create_response_deserialize() {
        let json = r#"{
            "id": "sess_token_abc",
            "object": "realtime.session",
            "client_secret": {
                "value": "ek_1234",
                "expires_at": 1700000000
            }
        }"#;
        let resp: ClientSecretCreateResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.client_secret.value, "ek_1234");
        assert_eq!(resp.client_secret.expires_at, 1_700_000_000);
    }

    #[test]
    fn call_accept_params_serialize() {
        let session_config = RealtimeSessionCreateRequestParam::new().with_model("gpt-realtime");
        let params = CallAcceptParams {
            session: session_config,
        };
        let json = serde_json::to_value(&params).expect("serialize");
        // Session fields are flattened to the top level (matching Go SDK wire format).
        assert_eq!(json["model"], "gpt-realtime");
        assert!(json.get("session").is_none(), "session should be flattened");
    }

    #[test]
    fn call_refer_params_serialize() {
        let params = CallReferParams {
            target_uri: "tel:+14155550123".to_owned(),
        };
        let json = serde_json::to_value(&params).expect("serialize");
        assert_eq!(json["target_uri"], "tel:+14155550123");
    }

    #[test]
    fn call_reject_params_serialize() {
        let params = CallRejectParams {
            status_code: Some(486),
        };
        let json = serde_json::to_value(&params).expect("serialize");
        assert_eq!(json["status_code"], 486);
    }
}
