//! Webhook signature verification for OpenAI webhooks.
//!
//! This module provides HMAC-SHA256 signature verification to ensure that
//! incoming webhook payloads were genuinely sent by OpenAI.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::{Error, Result};

type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// Webhook event types
// ---------------------------------------------------------------------------

/// Data payload common to most webhook events (contains the resource ID).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebhookEventData {
    /// The unique ID of the resource that triggered the event.
    pub id: String,
}

/// A SIP header from a Realtime incoming call webhook.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SipHeader {
    /// Name of the SIP header.
    pub name: String,
    /// Value of the SIP header.
    pub value: String,
}

/// Data payload for a `realtime.call.incoming` webhook event.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RealtimeCallIncomingData {
    /// The unique ID of this call.
    pub call_id: String,
    /// Headers from the SIP Invite.
    pub sip_headers: Vec<SipHeader>,
}

/// A typed webhook event, discriminated by the `type` field.
///
/// Covers all 15 event types defined in the OpenAI webhook specification.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum WebhookEvent {
    /// Sent when a batch API request has been cancelled.
    #[serde(rename = "batch.cancelled")]
    BatchCancelled {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload.
        data: WebhookEventData,
    },
    /// Sent when a batch API request has been completed.
    #[serde(rename = "batch.completed")]
    BatchCompleted {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload.
        data: WebhookEventData,
    },
    /// Sent when a batch API request has expired.
    #[serde(rename = "batch.expired")]
    BatchExpired {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload.
        data: WebhookEventData,
    },
    /// Sent when a batch API request has failed.
    #[serde(rename = "batch.failed")]
    BatchFailed {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload.
        data: WebhookEventData,
    },
    /// Sent when an eval run has been canceled.
    #[serde(rename = "eval.run.canceled")]
    EvalRunCanceled {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload.
        data: WebhookEventData,
    },
    /// Sent when an eval run has failed.
    #[serde(rename = "eval.run.failed")]
    EvalRunFailed {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload.
        data: WebhookEventData,
    },
    /// Sent when an eval run has succeeded.
    #[serde(rename = "eval.run.succeeded")]
    EvalRunSucceeded {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload.
        data: WebhookEventData,
    },
    /// Sent when a fine-tuning job has been cancelled.
    #[serde(rename = "fine_tuning.job.cancelled")]
    FineTuningJobCancelled {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload.
        data: WebhookEventData,
    },
    /// Sent when a fine-tuning job has failed.
    #[serde(rename = "fine_tuning.job.failed")]
    FineTuningJobFailed {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload.
        data: WebhookEventData,
    },
    /// Sent when a fine-tuning job has succeeded.
    #[serde(rename = "fine_tuning.job.succeeded")]
    FineTuningJobSucceeded {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload.
        data: WebhookEventData,
    },
    /// Sent when a Realtime API receives an incoming SIP call.
    #[serde(rename = "realtime.call.incoming")]
    RealtimeCallIncoming {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload with call details.
        data: RealtimeCallIncomingData,
    },
    /// Sent when a background response has been cancelled.
    #[serde(rename = "response.cancelled")]
    ResponseCancelled {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload.
        data: WebhookEventData,
    },
    /// Sent when a background response has been completed.
    #[serde(rename = "response.completed")]
    ResponseCompleted {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload.
        data: WebhookEventData,
    },
    /// Sent when a background response has failed.
    #[serde(rename = "response.failed")]
    ResponseFailed {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload.
        data: WebhookEventData,
    },
    /// Sent when a background response was incomplete.
    #[serde(rename = "response.incomplete")]
    ResponseIncomplete {
        /// The unique event ID.
        id: String,
        /// Unix timestamp of when the event was created.
        created_at: i64,
        /// Event data payload.
        data: WebhookEventData,
    },
}

/// Default tolerance window for webhook timestamp validation (5 minutes).
const DEFAULT_TOLERANCE: Duration = Duration::from_secs(5 * 60);

/// Parsed webhook headers required for signature verification.
#[derive(Debug, Clone)]
pub struct WebhookHeaders {
    /// The unique identifier for this webhook delivery.
    pub webhook_id: String,
    /// The Unix timestamp (seconds) when the webhook was sent.
    pub webhook_timestamp: String,
    /// The signature(s) of the webhook payload, potentially space-separated.
    pub webhook_signature: String,
}

impl WebhookHeaders {
    /// Creates headers from explicit values.
    pub fn new(
        webhook_id: impl Into<String>,
        webhook_timestamp: impl Into<String>,
        webhook_signature: impl Into<String>,
    ) -> Self {
        Self {
            webhook_id: webhook_id.into(),
            webhook_timestamp: webhook_timestamp.into(),
            webhook_signature: webhook_signature.into(),
        }
    }

    /// Extracts webhook headers from a generic header map.
    ///
    /// Looks for `webhook-id`, `webhook-timestamp`, and `webhook-signature` headers
    /// (case-insensitive).
    pub fn from_header_map(headers: &std::collections::HashMap<String, String>) -> Result<Self> {
        let get = |key: &str| -> Result<String> {
            // Try exact key, then lowercase
            headers
                .get(key)
                .or_else(|| headers.get(&key.to_lowercase()))
                .cloned()
                .ok_or_else(|| Error::WebhookVerification(format!("missing required {key} header")))
        };

        Ok(Self {
            webhook_id: get("webhook-id")?,
            webhook_timestamp: get("webhook-timestamp")?,
            webhook_signature: get("webhook-signature")?,
        })
    }
}

/// Service for verifying OpenAI webhook signatures.
///
/// # Example
///
/// ```no_run
/// use openai::webhooks::{WebhookService, WebhookHeaders};
///
/// let service = WebhookService::new("whsec_MfKQ9r8GKYqrTwjUPD8ILPZIo2LaLaSw");
/// let headers = WebhookHeaders::new(
///     "wh_123",
///     "1614265330",
///     "v1,abc123signature",
/// );
/// let body = b"{}";
/// service.verify_signature(body, &headers).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct WebhookService {
    secret: String,
    tolerance: Duration,
}

impl WebhookService {
    /// Creates a new webhook service with the given signing secret.
    ///
    /// The secret should be the full webhook secret string (e.g. `whsec_...`).
    /// The `whsec_` prefix is automatically stripped during signature computation.
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            tolerance: DEFAULT_TOLERANCE,
        }
    }

    /// Sets a custom tolerance window for timestamp validation.
    ///
    /// The default tolerance is 5 minutes. Webhook payloads with timestamps
    /// outside this window (past or future) are rejected.
    #[must_use]
    pub fn with_tolerance(mut self, tolerance: Duration) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Verifies the webhook signature using the current system time.
    ///
    /// Returns `Ok(())` if the signature is valid and the timestamp is within
    /// the tolerance window. Returns `Error::WebhookVerification` otherwise.
    pub fn verify_signature(&self, body: &[u8], headers: &WebhookHeaders) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::WebhookVerification(format!("system time error: {e}")))?
            .as_secs() as i64;
        self.verify_signature_with_time(body, headers, self.tolerance, now)
    }

    /// Verifies the webhook signature with custom tolerance and a fixed "now"
    /// timestamp (useful for testing).
    pub fn verify_signature_with_tolerance_and_time(
        &self,
        body: &[u8],
        headers: &WebhookHeaders,
        tolerance: Duration,
        now_unix: i64,
    ) -> Result<()> {
        self.verify_signature_with_time(body, headers, tolerance, now_unix)
    }

    /// Verifies the signature and deserializes the body into a typed [`WebhookEvent`].
    pub fn unwrap_event(&self, body: &[u8], headers: &WebhookHeaders) -> Result<WebhookEvent> {
        self.verify_signature(body, headers)?;
        serde_json::from_slice(body).map_err(Error::Json)
    }

    /// Verifies the signature with custom tolerance and time, then deserializes the body
    /// into a typed [`WebhookEvent`].
    pub fn unwrap_event_with_tolerance_and_time(
        &self,
        body: &[u8],
        headers: &WebhookHeaders,
        tolerance: Duration,
        now_unix: i64,
    ) -> Result<WebhookEvent> {
        self.verify_signature_with_time(body, headers, tolerance, now_unix)?;
        serde_json::from_slice(body).map_err(Error::Json)
    }

    // ── internal ─────────────────────────────────────────────────────

    fn verify_signature_with_time(
        &self,
        body: &[u8],
        headers: &WebhookHeaders,
        tolerance: Duration,
        now_unix: i64,
    ) -> Result<()> {
        if self.secret.is_empty() {
            return Err(Error::WebhookVerification(
                "webhook secret must be provided either in the method call or configured on the client".to_owned(),
            ));
        }

        // Validate required header fields are non-empty.
        if headers.webhook_id.is_empty() {
            return Err(Error::WebhookVerification(
                "missing required webhook-id header".to_owned(),
            ));
        }
        if headers.webhook_timestamp.is_empty() {
            return Err(Error::WebhookVerification(
                "missing required webhook-timestamp header".to_owned(),
            ));
        }
        if headers.webhook_signature.is_empty() {
            return Err(Error::WebhookVerification(
                "missing required webhook-signature header".to_owned(),
            ));
        }

        // Parse the timestamp.
        let timestamp_seconds: i64 = headers.webhook_timestamp.parse().map_err(|_| {
            Error::WebhookVerification("invalid webhook timestamp format".to_owned())
        })?;

        // Validate timestamp within tolerance window.
        let tolerance_secs = tolerance.as_secs() as i64;

        if now_unix - timestamp_seconds > tolerance_secs {
            return Err(Error::WebhookVerification(
                "webhook timestamp is too old".to_owned(),
            ));
        }
        if timestamp_seconds > now_unix + tolerance_secs {
            return Err(Error::WebhookVerification(
                "webhook timestamp is too new".to_owned(),
            ));
        }

        // Decode the secret. Strip `whsec_` prefix if present.
        let secret_payload = if let Some(stripped) = self.secret.strip_prefix("whsec_") {
            stripped
        } else {
            &self.secret
        };

        let decoded_secret = BASE64.decode(secret_payload).map_err(|e| {
            Error::WebhookVerification(format!("invalid webhook secret format: {e}"))
        })?;

        // Construct signed content: {webhook_id}.{timestamp}.{body}
        let body_str = std::str::from_utf8(body)
            .map_err(|e| Error::WebhookVerification(format!("invalid body encoding: {e}")))?;
        let signed_content = format!(
            "{}.{}.{}",
            headers.webhook_id, headers.webhook_timestamp, body_str
        );

        // Compute HMAC-SHA256 signature.
        let mut mac = HmacSha256::new_from_slice(&decoded_secret)
            .map_err(|e| Error::WebhookVerification(format!("HMAC key error: {e}")))?;
        mac.update(signed_content.as_bytes());
        let expected_signature = BASE64.encode(mac.finalize().into_bytes());

        // Parse signatures from header: space-separated, each optionally prefixed with `v1,`.
        let signatures: Vec<&str> = headers
            .webhook_signature
            .split_whitespace()
            .map(|part| part.strip_prefix("v1,").unwrap_or(part))
            .collect();

        // Accept if any signature matches using constant-time comparison.
        for sig in &signatures {
            if constant_time_eq(expected_signature.as_bytes(), sig.as_bytes()) {
                return Ok(());
            }
        }

        Err(Error::WebhookVerification(
            "webhook signature verification failed".to_owned(),
        ))
    }
}

/// Constant-time comparison of two byte slices.
///
/// Returns `true` only if the slices are of equal length and every byte matches.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Test vectors from the Go test suite.
    const TEST_PAYLOAD: &str = r#"{"id": "evt_685c059ae3a481909bdc86819b066fb6", "object": "event", "created_at": 1750861210, "type": "response.completed", "data": {"id": "resp_123"}}"#;
    const TEST_SECRET: &str = "whsec_RdvaYFYUXuIFuEbvZHwMfYFhUf7aMYjYcmM24+Aj40c=";
    const TEST_TIMESTAMP: i64 = 1750861210;
    const TEST_WEBHOOK_ID: &str = "wh_685c059ae39c8190af8c71ed1022a24d";
    const TEST_SIGNATURE: &str = "v1,gUAg4R2hWouRZqRQG4uJypNS8YK885G838+EHb4nKBY=";

    fn test_headers() -> WebhookHeaders {
        WebhookHeaders::new(TEST_WEBHOOK_ID, TEST_TIMESTAMP.to_string(), TEST_SIGNATURE)
    }

    fn test_service() -> WebhookService {
        WebhookService::new(TEST_SECRET)
    }

    // ── Valid signature ──────────────────────────────────────────────

    #[test]
    fn valid_signature_verification() {
        let service = test_service();
        let headers = test_headers();
        let result = service.verify_signature_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(5 * 60),
            TEST_TIMESTAMP,
        );
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }

    // ── Invalid signature ────────────────────────────────────────────

    #[test]
    fn invalid_signature_is_rejected() {
        let service = test_service();
        let headers = WebhookHeaders::new(
            TEST_WEBHOOK_ID,
            TEST_TIMESTAMP.to_string(),
            "v1,invalid_signature_here",
        );
        let result = service.verify_signature_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(5 * 60),
            TEST_TIMESTAMP,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Webhook signature verification failed: webhook signature verification failed"
        );
    }

    // ── Missing secret ───────────────────────────────────────────────

    #[test]
    fn missing_secret_returns_error() {
        let service = WebhookService::new("");
        let headers = test_headers();
        let result = service.verify_signature_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(5 * 60),
            TEST_TIMESTAMP,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("webhook secret must be provided"),
            "unexpected error: {msg}"
        );
    }

    // ── Invalid timestamp format ─────────────────────────────────────

    #[test]
    fn invalid_timestamp_format_returns_error() {
        let service = test_service();
        let headers = WebhookHeaders::new(TEST_WEBHOOK_ID, "not_a_number", "v1,signature");
        let result = service.verify_signature_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(5 * 60),
            TEST_TIMESTAMP,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("invalid webhook timestamp format"),
            "unexpected error: {msg}"
        );
    }

    // ── Timestamp too old (replay attack prevention) ─────────────────

    #[test]
    fn timestamp_too_old_is_rejected() {
        let service = test_service();
        // Timestamp 400 seconds in the past (exceeds 5 minute tolerance)
        let old_ts = TEST_TIMESTAMP - 400;
        let headers = WebhookHeaders::new(TEST_WEBHOOK_ID, old_ts.to_string(), "v1,signature");
        let result = service.verify_signature_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(5 * 60),
            TEST_TIMESTAMP,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("webhook timestamp is too old"),
            "unexpected error: {msg}"
        );
    }

    // ── Timestamp too new ────────────────────────────────────────────

    #[test]
    fn timestamp_too_new_is_rejected() {
        let service = test_service();
        // Timestamp 400 seconds in the future (exceeds 5 minute tolerance)
        let future_ts = TEST_TIMESTAMP + 400;
        let headers = WebhookHeaders::new(TEST_WEBHOOK_ID, future_ts.to_string(), "v1,signature");
        let result = service.verify_signature_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(5 * 60),
            TEST_TIMESTAMP,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("webhook timestamp is too new"),
            "unexpected error: {msg}"
        );
    }

    // ── Signature without v1 prefix ──────────────────────────────────

    #[test]
    fn signature_without_v1_prefix_is_rejected() {
        let service = test_service();
        // Bare signature (no v1, prefix) that does NOT match the expected value
        let headers = WebhookHeaders::new(
            TEST_WEBHOOK_ID,
            TEST_TIMESTAMP.to_string(),
            "9WlByKQUfBVM08XRYmo3WqR/dQXtjGJkV1edShZZ+C0=",
        );
        let result = service.verify_signature_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(5 * 60),
            TEST_TIMESTAMP,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("webhook signature verification failed"),
            "unexpected error: {msg}"
        );
    }

    // ── Multiple signatures, one valid ───────────────────────────────

    #[test]
    fn multiple_signatures_one_valid_passes() {
        let service = test_service();
        let multi_sig = format!("v1,invalid_signature {TEST_SIGNATURE}");
        let headers = WebhookHeaders::new(TEST_WEBHOOK_ID, TEST_TIMESTAMP.to_string(), multi_sig);
        let result = service.verify_signature_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(5 * 60),
            TEST_TIMESTAMP,
        );
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }

    // ── Multiple signatures, all invalid ─────────────────────────────

    #[test]
    fn multiple_signatures_all_invalid_fails() {
        let service = test_service();
        let headers = WebhookHeaders::new(
            TEST_WEBHOOK_ID,
            TEST_TIMESTAMP.to_string(),
            "v1,invalid_signature1 v1,invalid_signature2",
        );
        let result = service.verify_signature_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(5 * 60),
            TEST_TIMESTAMP,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("webhook signature verification failed"),
            "unexpected error: {msg}"
        );
    }

    // ── Custom tolerance ─────────────────────────────────────────────

    #[test]
    fn custom_tolerance_allows_older_timestamps() {
        let service = test_service();
        // 400 seconds old exceeds 5 min but is within 10 min
        let old_ts = TEST_TIMESTAMP - 400;
        let headers = WebhookHeaders::new(TEST_WEBHOOK_ID, old_ts.to_string(), "v1,signature");

        // Should fail with default 5 minute tolerance
        let result = service.verify_signature_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(5 * 60),
            TEST_TIMESTAMP,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("webhook timestamp is too old"),
            "unexpected error: {msg}"
        );

        // Should pass timestamp check with 10 minute tolerance (but fail on signature)
        let result = service.verify_signature_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(10 * 60),
            TEST_TIMESTAMP,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("webhook signature verification failed"),
            "unexpected error: {msg}"
        );
    }

    // ── Unwrap event ─────────────────────────────────────────────────

    #[test]
    fn unwrap_event_parses_valid_payload() {
        let service = test_service();
        let headers = test_headers();
        let event = service.unwrap_event_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(5 * 60),
            TEST_TIMESTAMP,
        );
        assert!(event.is_ok(), "expected Ok, got: {event:?}");
        let value = event.unwrap();
        match value {
            WebhookEvent::ResponseCompleted { id, data, .. } => {
                assert_eq!(id, "evt_685c059ae3a481909bdc86819b066fb6");
                assert_eq!(data.id, "resp_123");
            }
            other => panic!("expected ResponseCompleted, got {other:?}"),
        }
    }

    #[test]
    fn webhook_event_batch_cancelled_round_trip() {
        let json = r#"{
            "type":"batch.cancelled",
            "id":"evt_1",
            "created_at":1000,
            "data":{"id":"batch_abc"}
        }"#;
        let event: WebhookEvent = serde_json::from_str(json).expect("deserialize");
        match &event {
            WebhookEvent::BatchCancelled { id, data, .. } => {
                assert_eq!(id, "evt_1");
                assert_eq!(data.id, "batch_abc");
            }
            other => panic!("expected BatchCancelled, got {other:?}"),
        }
    }

    #[test]
    fn webhook_event_realtime_call_incoming_round_trip() {
        let json = r#"{
            "type":"realtime.call.incoming",
            "id":"evt_2",
            "created_at":2000,
            "data":{"call_id":"call_abc","sip_headers":[{"name":"From","value":"sip:user@example.com"}]}
        }"#;
        let event: WebhookEvent = serde_json::from_str(json).expect("deserialize");
        match &event {
            WebhookEvent::RealtimeCallIncoming { data, .. } => {
                assert_eq!(data.call_id, "call_abc");
                assert_eq!(data.sip_headers.len(), 1);
                assert_eq!(data.sip_headers[0].name, "From");
            }
            other => panic!("expected RealtimeCallIncoming, got {other:?}"),
        }
    }

    #[test]
    fn webhook_event_all_variants_deserialize() {
        let types = vec![
            (
                "batch.cancelled",
                r#"{"type":"batch.cancelled","id":"e1","created_at":0,"data":{"id":"x"}}"#,
            ),
            (
                "batch.completed",
                r#"{"type":"batch.completed","id":"e2","created_at":0,"data":{"id":"x"}}"#,
            ),
            (
                "batch.expired",
                r#"{"type":"batch.expired","id":"e3","created_at":0,"data":{"id":"x"}}"#,
            ),
            (
                "batch.failed",
                r#"{"type":"batch.failed","id":"e4","created_at":0,"data":{"id":"x"}}"#,
            ),
            (
                "eval.run.canceled",
                r#"{"type":"eval.run.canceled","id":"e5","created_at":0,"data":{"id":"x"}}"#,
            ),
            (
                "eval.run.failed",
                r#"{"type":"eval.run.failed","id":"e6","created_at":0,"data":{"id":"x"}}"#,
            ),
            (
                "eval.run.succeeded",
                r#"{"type":"eval.run.succeeded","id":"e7","created_at":0,"data":{"id":"x"}}"#,
            ),
            (
                "fine_tuning.job.cancelled",
                r#"{"type":"fine_tuning.job.cancelled","id":"e8","created_at":0,"data":{"id":"x"}}"#,
            ),
            (
                "fine_tuning.job.failed",
                r#"{"type":"fine_tuning.job.failed","id":"e9","created_at":0,"data":{"id":"x"}}"#,
            ),
            (
                "fine_tuning.job.succeeded",
                r#"{"type":"fine_tuning.job.succeeded","id":"e10","created_at":0,"data":{"id":"x"}}"#,
            ),
            (
                "response.cancelled",
                r#"{"type":"response.cancelled","id":"e12","created_at":0,"data":{"id":"x"}}"#,
            ),
            (
                "response.completed",
                r#"{"type":"response.completed","id":"e13","created_at":0,"data":{"id":"x"}}"#,
            ),
            (
                "response.failed",
                r#"{"type":"response.failed","id":"e14","created_at":0,"data":{"id":"x"}}"#,
            ),
            (
                "response.incomplete",
                r#"{"type":"response.incomplete","id":"e15","created_at":0,"data":{"id":"x"}}"#,
            ),
        ];
        for (label, json) in types {
            let event: WebhookEvent = serde_json::from_str(json)
                .unwrap_or_else(|e| panic!("failed to deserialize {label}: {e}"));
            // Re-serialize to confirm round-trip works
            let _json = serde_json::to_string(&event)
                .unwrap_or_else(|e| panic!("failed to serialize {label}: {e}"));
        }
    }

    // ── Missing headers ──────────────────────────────────────────────

    #[test]
    fn missing_webhook_id_returns_error() {
        let service = test_service();
        let headers = WebhookHeaders::new("", TEST_TIMESTAMP.to_string(), TEST_SIGNATURE);
        let result = service.verify_signature_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(5 * 60),
            TEST_TIMESTAMP,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("missing required webhook-id header"),
            "unexpected: {msg}"
        );
    }

    #[test]
    fn missing_webhook_timestamp_returns_error() {
        let service = test_service();
        let headers = WebhookHeaders::new(TEST_WEBHOOK_ID, "", TEST_SIGNATURE);
        let result = service.verify_signature_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(5 * 60),
            TEST_TIMESTAMP,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("missing required webhook-timestamp header"),
            "unexpected: {msg}"
        );
    }

    #[test]
    fn missing_webhook_signature_returns_error() {
        let service = test_service();
        let headers = WebhookHeaders::new(TEST_WEBHOOK_ID, TEST_TIMESTAMP.to_string(), "");
        let result = service.verify_signature_with_tolerance_and_time(
            TEST_PAYLOAD.as_bytes(),
            &headers,
            Duration::from_secs(5 * 60),
            TEST_TIMESTAMP,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("missing required webhook-signature header"),
            "unexpected: {msg}"
        );
    }

    // ── from_header_map ──────────────────────────────────────────────

    #[test]
    fn from_header_map_extracts_headers() {
        let mut map = std::collections::HashMap::new();
        map.insert("webhook-id".to_owned(), TEST_WEBHOOK_ID.to_owned());
        map.insert("webhook-timestamp".to_owned(), TEST_TIMESTAMP.to_string());
        map.insert("webhook-signature".to_owned(), TEST_SIGNATURE.to_owned());

        let headers = WebhookHeaders::from_header_map(&map).unwrap();
        assert_eq!(headers.webhook_id, TEST_WEBHOOK_ID);
        assert_eq!(headers.webhook_timestamp, TEST_TIMESTAMP.to_string());
        assert_eq!(headers.webhook_signature, TEST_SIGNATURE);
    }

    #[test]
    fn from_header_map_missing_key_returns_error() {
        let map = std::collections::HashMap::new();
        let result = WebhookHeaders::from_header_map(&map);
        assert!(result.is_err());
    }

    // ── Constant-time comparison ─────────────────────────────────────

    #[test]
    fn constant_time_eq_basic() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
        assert!(!constant_time_eq(b"", b"a"));
        assert!(constant_time_eq(b"", b""));
    }

    // ── with_tolerance builder ───────────────────────────────────────

    #[test]
    fn with_tolerance_overrides_default() {
        let service = WebhookService::new(TEST_SECRET).with_tolerance(Duration::from_secs(10));
        assert_eq!(service.tolerance, Duration::from_secs(10));
    }

    // --- Proptest fuzz tests ---

    mod fuzz {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn verify_signature_never_panics(
                secret in ".*",
                webhook_id in ".*",
                webhook_timestamp in ".*",
                webhook_signature in ".*",
                body in prop::collection::vec(any::<u8>(), 0..512),
            ) {
                let service = WebhookService::new(secret);
                let headers = WebhookHeaders::new(
                    webhook_id,
                    webhook_timestamp,
                    webhook_signature,
                );
                // Must not panic; errors are fine.
                let _ = service.verify_signature(&body, &headers);
            }

            #[test]
            fn from_header_map_never_panics(
                keys in prop::collection::vec(".*", 0..6),
                values in prop::collection::vec(".*", 0..6),
            ) {
                let mut map = std::collections::HashMap::new();
                for (k, v) in keys.into_iter().zip(values.into_iter()) {
                    map.insert(k, v);
                }
                let _ = WebhookHeaders::from_header_map(&map);
            }

            #[test]
            fn constant_time_eq_never_panics(
                a in prop::collection::vec(any::<u8>(), 0..128),
                b in prop::collection::vec(any::<u8>(), 0..128),
            ) {
                let _ = constant_time_eq(&a, &b);
            }
        }
    }
}
