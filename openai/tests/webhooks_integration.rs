//! Integration tests for the webhooks module.
//!
//! These tests use the same test vectors as the Go SDK test suite to ensure
//! cross-language compatibility.

use std::time::Duration;

use openai::webhooks::{WebhookHeaders, WebhookService};

// Test vectors matching the Go SDK.
const TEST_PAYLOAD: &str = r#"{"id": "evt_685c059ae3a481909bdc86819b066fb6", "object": "event", "created_at": 1750861210, "type": "response.completed", "data": {"id": "resp_123"}}"#;
const TEST_SECRET: &str = "whsec_RdvaYFYUXuIFuEbvZHwMfYFhUf7aMYjYcmM24+Aj40c=";
const TEST_TIMESTAMP: i64 = 1750861210;
const TEST_WEBHOOK_ID: &str = "wh_685c059ae39c8190af8c71ed1022a24d";
const TEST_SIGNATURE: &str = "v1,gUAg4R2hWouRZqRQG4uJypNS8YK885G838+EHb4nKBY=";

fn service() -> WebhookService {
    WebhookService::new(TEST_SECRET)
}

fn headers() -> WebhookHeaders {
    WebhookHeaders::new(TEST_WEBHOOK_ID, TEST_TIMESTAMP.to_string(), TEST_SIGNATURE)
}

#[test]
fn valid_signature_is_accepted() {
    let result = service().verify_signature_with_tolerance_and_time(
        TEST_PAYLOAD.as_bytes(),
        &headers(),
        Duration::from_secs(300),
        TEST_TIMESTAMP,
    );
    assert!(result.is_ok(), "valid signature should pass: {result:?}");
}

#[test]
fn invalid_signature_is_rejected() {
    let hdrs = WebhookHeaders::new(
        TEST_WEBHOOK_ID,
        TEST_TIMESTAMP.to_string(),
        "v1,invalid_signature_here",
    );
    let result = service().verify_signature_with_tolerance_and_time(
        TEST_PAYLOAD.as_bytes(),
        &hdrs,
        Duration::from_secs(300),
        TEST_TIMESTAMP,
    );
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("webhook signature verification failed"),);
}

#[test]
fn expired_timestamp_prevents_replay_attack() {
    let old_ts = TEST_TIMESTAMP - 400; // 6m40s ago, exceeds 5m tolerance
    let hdrs = WebhookHeaders::new(TEST_WEBHOOK_ID, old_ts.to_string(), "v1,sig");
    let result = service().verify_signature_with_tolerance_and_time(
        TEST_PAYLOAD.as_bytes(),
        &hdrs,
        Duration::from_secs(300),
        TEST_TIMESTAMP,
    );
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("webhook timestamp is too old"));
}

#[test]
fn future_timestamp_is_rejected() {
    let future_ts = TEST_TIMESTAMP + 400;
    let hdrs = WebhookHeaders::new(TEST_WEBHOOK_ID, future_ts.to_string(), "v1,sig");
    let result = service().verify_signature_with_tolerance_and_time(
        TEST_PAYLOAD.as_bytes(),
        &hdrs,
        Duration::from_secs(300),
        TEST_TIMESTAMP,
    );
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("webhook timestamp is too new"));
}

#[test]
fn multiple_signatures_one_valid_passes() {
    let multi_sig = format!("v1,invalid_signature {TEST_SIGNATURE}");
    let hdrs = WebhookHeaders::new(TEST_WEBHOOK_ID, TEST_TIMESTAMP.to_string(), multi_sig);
    let result = service().verify_signature_with_tolerance_and_time(
        TEST_PAYLOAD.as_bytes(),
        &hdrs,
        Duration::from_secs(300),
        TEST_TIMESTAMP,
    );
    assert!(
        result.is_ok(),
        "at least one valid signature should pass: {result:?}"
    );
}

#[test]
fn multiple_signatures_all_invalid_fails() {
    let hdrs = WebhookHeaders::new(
        TEST_WEBHOOK_ID,
        TEST_TIMESTAMP.to_string(),
        "v1,bad1 v1,bad2",
    );
    let result = service().verify_signature_with_tolerance_and_time(
        TEST_PAYLOAD.as_bytes(),
        &hdrs,
        Duration::from_secs(300),
        TEST_TIMESTAMP,
    );
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("webhook signature verification failed"));
}

#[test]
fn malformed_timestamp_header_returns_error() {
    let hdrs = WebhookHeaders::new(TEST_WEBHOOK_ID, "not_a_number", "v1,sig");
    let result = service().verify_signature_with_tolerance_and_time(
        TEST_PAYLOAD.as_bytes(),
        &hdrs,
        Duration::from_secs(300),
        TEST_TIMESTAMP,
    );
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("invalid webhook timestamp format"));
}

#[test]
fn unwrap_event_returns_parsed_event() {
    use openai::webhooks::WebhookEvent;

    let event = service().unwrap_event_with_tolerance_and_time(
        TEST_PAYLOAD.as_bytes(),
        &headers(),
        Duration::from_secs(300),
        TEST_TIMESTAMP,
    );
    assert!(event.is_ok(), "unwrap_event should succeed: {event:?}");
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
fn signature_without_v1_prefix_is_rejected() {
    // A bare signature (no v1, prefix) that doesn't match the expected value
    let hdrs = WebhookHeaders::new(
        TEST_WEBHOOK_ID,
        TEST_TIMESTAMP.to_string(),
        "9WlByKQUfBVM08XRYmo3WqR/dQXtjGJkV1edShZZ+C0=",
    );
    let result = service().verify_signature_with_tolerance_and_time(
        TEST_PAYLOAD.as_bytes(),
        &hdrs,
        Duration::from_secs(300),
        TEST_TIMESTAMP,
    );
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("webhook signature verification failed"));
}

#[test]
fn empty_secret_returns_error() {
    let svc = WebhookService::new("");
    let result = svc.verify_signature_with_tolerance_and_time(
        TEST_PAYLOAD.as_bytes(),
        &headers(),
        Duration::from_secs(300),
        TEST_TIMESTAMP,
    );
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("webhook secret must be provided"));
}

#[test]
fn custom_tolerance_widens_acceptance_window() {
    let old_ts = TEST_TIMESTAMP - 400;
    let hdrs = WebhookHeaders::new(TEST_WEBHOOK_ID, old_ts.to_string(), "v1,sig");

    // 5 min tolerance rejects
    let result = service().verify_signature_with_tolerance_and_time(
        TEST_PAYLOAD.as_bytes(),
        &hdrs,
        Duration::from_secs(300),
        TEST_TIMESTAMP,
    );
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("webhook timestamp is too old"));

    // 10 min tolerance passes timestamp check but fails on signature
    let result = service().verify_signature_with_tolerance_and_time(
        TEST_PAYLOAD.as_bytes(),
        &hdrs,
        Duration::from_secs(600),
        TEST_TIMESTAMP,
    );
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("webhook signature verification failed"));
}

#[test]
fn client_webhooks_accessor_creates_service() {
    // Verify the Client::webhooks() accessor compiles and returns a working service.
    let config = openai::ClientConfig::default().with_api_key("test-key");
    let client = openai::Client::new(config).unwrap();
    let svc = client.webhooks(TEST_SECRET);
    let result = svc.verify_signature_with_tolerance_and_time(
        TEST_PAYLOAD.as_bytes(),
        &headers(),
        Duration::from_secs(300),
        TEST_TIMESTAMP,
    );
    assert!(
        result.is_ok(),
        "client accessor service should work: {result:?}"
    );
}

#[test]
fn client_webhooks_from_config_uses_configured_secret() {
    let config = openai::ClientConfig::default()
        .with_api_key("test-key")
        .with_webhook_secret(TEST_SECRET);
    let client = openai::Client::new(config).unwrap();
    let svc = client.webhooks_from_config();
    let result = svc.verify_signature_with_tolerance_and_time(
        TEST_PAYLOAD.as_bytes(),
        &headers(),
        Duration::from_secs(300),
        TEST_TIMESTAMP,
    );
    assert!(
        result.is_ok(),
        "configured webhook service should work: {result:?}"
    );
}
