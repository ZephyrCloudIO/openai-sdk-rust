//! HTTP-based sub-services for the Realtime API: Call management and Client
//! Secrets.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::Client;

use super::events::RealtimeSessionCreateRequestParam;

// ---------------------------------------------------------------------------
// CallService
// ---------------------------------------------------------------------------

/// Service for managing Realtime SIP/WebRTC calls.
///
/// Accessed via [`super::RealtimeService::calls()`].
#[derive(Clone)]
pub struct CallService {
    client: Client,
}

impl CallService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Accept an incoming SIP call and configure the realtime session.
    ///
    /// # Errors
    /// Returns an error on transport or API failures.
    pub async fn accept(&self, call_id: &str, params: CallAcceptParams) -> Result<()> {
        let path = format!("/realtime/calls/{call_id}/accept");
        self.client.post_empty_json(&path, &params).await
    }

    /// End an active Realtime API call.
    ///
    /// # Errors
    /// Returns an error on transport or API failures.
    pub async fn hangup(&self, call_id: &str) -> Result<()> {
        let path = format!("/realtime/calls/{call_id}/hangup");
        let empty = serde_json::Value::Object(serde_json::Map::new());
        self.client.post_empty_json(&path, &empty).await
    }

    /// Transfer an active SIP call to a new destination using SIP REFER.
    ///
    /// # Errors
    /// Returns an error on transport or API failures.
    pub async fn refer(&self, call_id: &str, params: CallReferParams) -> Result<()> {
        let path = format!("/realtime/calls/{call_id}/refer");
        self.client.post_empty_json(&path, &params).await
    }

    /// Decline an incoming SIP call with a SIP status code.
    ///
    /// # Errors
    /// Returns an error on transport or API failures.
    pub async fn reject(&self, call_id: &str, params: CallRejectParams) -> Result<()> {
        let path = format!("/realtime/calls/{call_id}/reject");
        self.client.post_empty_json(&path, &params).await
    }
}

/// Parameters for accepting a SIP call.
///
/// The Go SDK's `CallAcceptParams.MarshalJSON` flattens the session config to
/// the top level, so the wire format is `{"model": "...", "instructions": "..."}`
/// rather than `{"session": {"model": "..."}}`. We use `#[serde(flatten)]` to
/// match that behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallAcceptParams {
    /// Realtime session configuration for the accepted call (flattened to
    /// top-level JSON fields on the wire).
    #[serde(flatten)]
    pub session: RealtimeSessionCreateRequestParam,
}

/// Parameters for transferring a SIP call via REFER.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallReferParams {
    /// URI for the SIP Refer-To header (e.g. `"tel:+14155550123"`).
    pub target_uri: String,
}

/// Parameters for rejecting a SIP call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRejectParams {
    /// SIP response code (defaults to 603 / Decline if omitted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<i64>,
}

// ---------------------------------------------------------------------------
// ClientSecretService
// ---------------------------------------------------------------------------

/// Service for creating ephemeral client secrets for client-side Realtime
/// connections.
///
/// Accessed via [`super::RealtimeService::client_secrets()`].
#[derive(Clone)]
pub struct ClientSecretService {
    client: Client,
}

impl ClientSecretService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Create a short-lived client secret for use in browser or mobile clients.
    ///
    /// The returned [`ClientSecretCreateResponse`] contains the ephemeral key
    /// value and its expiration timestamp.
    ///
    /// # Errors
    /// Returns an error on transport or API failures.
    pub async fn create(
        &self,
        params: ClientSecretCreateParams,
    ) -> Result<ClientSecretCreateResponse> {
        self.client
            .post_json("/realtime/client_secrets", &params)
            .await
    }
}

/// Parameters for creating a Realtime client secret.
///
/// This replaces the minimal struct with the full session configuration fields,
/// matching the Go SDK's `ClientSecretNewParams`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSecretCreateParams {
    /// Session configuration. Choose a realtime or transcription session.
    ///
    /// Uses the full [`RealtimeSessionCreateRequestParam`] type with all
    /// 14+ session configuration fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<ClientSecretSessionConfig>,
    /// Optional expiration configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<ClientSecretExpiresAfter>,
}

/// Session configuration variant: either a realtime or transcription session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientSecretSessionConfig {
    /// A standard realtime session.
    #[serde(rename = "realtime")]
    Realtime(RealtimeSessionCreateRequestParam),
    /// A transcription-only session.
    #[serde(rename = "transcription")]
    Transcription(TranscriptionSessionConfig),
}

/// Minimal transcription session configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TranscriptionSessionConfig {
    /// Configuration for input audio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<TranscriptionAudioConfig>,
    /// Additional fields to include in server outputs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}

/// Audio configuration for transcription sessions (input only).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TranscriptionAudioConfig {
    /// Input audio configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<super::events::RealtimeAudioInputConfig>,
}

/// Expiration configuration for a client secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSecretExpiresAfter {
    /// Anchor point for expiration. Currently only `"created_at"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
    /// Seconds after anchor until the secret expires (10 to 7200).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds: Option<i64>,
}

/// Response from creating a client secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSecretCreateResponse {
    /// Session/token identifier.
    #[serde(default)]
    pub id: String,
    /// Object type.
    #[serde(default)]
    pub object: String,
    /// The ephemeral client secret.
    pub client_secret: ClientSecret,
}

/// An ephemeral client secret value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSecret {
    /// The secret token value (e.g. `"ek_1234"`).
    pub value: String,
    /// Unix timestamp when the secret expires.
    pub expires_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_accept_params_serializes_flat() {
        // The Go SDK flattens CallAcceptParams so that session config fields
        // appear at the top level. Verify our #[serde(flatten)] achieves this.
        let params = CallAcceptParams {
            session: RealtimeSessionCreateRequestParam {
                model: Some("gpt-4o-realtime-preview".to_owned()),
                instructions: Some("Be helpful".to_owned()),
                ..Default::default()
            },
        };

        let json = serde_json::to_value(&params).expect("serialize CallAcceptParams");

        // Fields should be at the top level, NOT nested under "session".
        assert!(
            json.get("session").is_none(),
            "session field should be flattened, not nested"
        );
        assert_eq!(
            json.get("model"),
            Some(&serde_json::Value::String(
                "gpt-4o-realtime-preview".to_owned()
            )),
            "model should be at top level"
        );
        assert_eq!(
            json.get("instructions"),
            Some(&serde_json::Value::String("Be helpful".to_owned())),
            "instructions should be at top level"
        );
    }

    #[test]
    fn call_accept_params_deserializes_from_flat() {
        let json = r#"{
            "model": "gpt-4o-realtime-preview",
            "instructions": "Be helpful"
        }"#;

        let params: CallAcceptParams =
            serde_json::from_str(json).expect("deserialize flat CallAcceptParams");
        assert_eq!(
            params.session.model.as_deref(),
            Some("gpt-4o-realtime-preview")
        );
        assert_eq!(params.session.instructions.as_deref(), Some("Be helpful"));
    }

    #[test]
    fn call_refer_params_serialization() {
        let params = CallReferParams {
            target_uri: "tel:+14155550123".to_owned(),
        };
        let json = serde_json::to_value(&params).expect("serialize CallReferParams");
        assert_eq!(
            json.get("target_uri"),
            Some(&serde_json::Value::String("tel:+14155550123".to_owned()))
        );
    }

    #[test]
    fn call_reject_params_serialization() {
        let params = CallRejectParams {
            status_code: Some(486),
        };
        let json = serde_json::to_value(&params).expect("serialize CallRejectParams");
        assert_eq!(json.get("status_code"), Some(&serde_json::json!(486)));

        let params_default = CallRejectParams { status_code: None };
        let json = serde_json::to_value(&params_default).expect("serialize empty CallRejectParams");
        assert!(json.get("status_code").is_none());
    }
}
