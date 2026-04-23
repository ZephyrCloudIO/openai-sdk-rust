//! Error and result types.

/// SDK result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Envelope returned by OpenAI API on error responses.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiErrorEnvelope {
    /// The nested API error payload.
    pub error: ApiErrorPayload,
}

/// Nested API error payload fields.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiErrorPayload {
    /// Human-readable message.
    pub message: String,
    /// Error code.
    pub code: Option<String>,
    /// Parameter name if relevant.
    pub param: Option<String>,
    /// Error type.
    #[serde(rename = "type")]
    pub error_type: Option<String>,
}

/// SDK error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// API error response.
    #[error("API error {status}: {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// API error code.
        code: Option<String>,
        /// Message.
        message: String,
        /// API parameter name.
        param: Option<String>,
        /// API error type.
        error_type: Option<String>,
    },

    /// HTTP transport error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON decode/encode error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Websocket error.
    #[error("WebSocket error: {0}")]
    WebSocket(Box<tokio_tungstenite::tungstenite::Error>),

    /// Webhook verification failed.
    #[error("Webhook signature verification failed: {0}")]
    WebhookVerification(String),

    /// Streaming decode failed.
    #[error("Stream error: {0}")]
    Stream(String),

    /// Authentication failed.
    #[error("Auth error: {0}")]
    Auth(#[from] crate::auth::AuthError),

    /// Invalid client configuration.
    #[error("Invalid client config `{field}`: {message}")]
    Config {
        /// Config field name.
        field: &'static str,
        /// Validation message.
        message: String,
    },
}

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(value: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocket(Box::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn api_error_display_includes_status_and_message() {
        let err = Error::Api {
            status: 401,
            code: Some("invalid_api_key".to_owned()),
            message: "Invalid API key".to_owned(),
            param: None,
            error_type: Some("invalid_request_error".to_owned()),
        };

        let msg = err.to_string();
        assert!(msg.contains("401"));
        assert!(msg.contains("Invalid API key"));
    }

    #[test]
    fn from_reqwest_error_maps_to_http_variant() {
        let reqwest_err = reqwest::Client::new()
            .get("not a valid url")
            .build()
            .expect_err("invalid URL should fail request build");

        let err = Error::from(reqwest_err);
        assert!(matches!(err, Error::Http(_)));
    }

    #[test]
    fn from_json_error_maps_to_json_variant() {
        let json_err = serde_json::from_str::<serde_json::Value>("{")
            .expect_err("invalid JSON should fail parsing");

        let err = Error::from(json_err);
        assert!(matches!(err, Error::Json(_)));
    }

    #[test]
    fn from_websocket_error_maps_to_websocket_variant() {
        let websocket_err = tokio_tungstenite::tungstenite::Error::ConnectionClosed;
        let err = Error::from(websocket_err);
        assert!(matches!(err, Error::WebSocket(_)));
    }
}
