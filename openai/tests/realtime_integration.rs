//! Integration tests for the Realtime API module.
//!
//! These tests exercise the HTTP sub-services (CallService, ClientSecretService)
//! against a mock server. WebSocket connection tests are not included because
//! `wiremock` does not support WebSocket upgrades, but the URL construction and
//! event serde are thoroughly covered in unit tests.

use openai::{
    realtime::{
        CallAcceptParams, CallReferParams, CallRejectParams, ClientSecretCreateParams,
        ClientSecretCreateResponse, ClientSecretSessionConfig, RealtimeConnectParams,
        RealtimeSession, RealtimeSessionCreateRequestParam,
    },
    Client, ClientConfig, Error, RequestConfig,
};
use serde_json::json;
use wiremock::{
    matchers::{body_partial_json, header, method, path},
    Mock, MockServer, ResponseTemplate,
};

// ---------------------------------------------------------------------------
// CallService integration tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn call_accept_posts_session_config() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/realtime/calls/call_abc/accept"))
        .and(header("Authorization", "Bearer test-key"))
        .and(body_partial_json(json!({
            "model": "gpt-realtime"
        })))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri()),
    )
    .expect("client init");

    let result = client
        .realtime()
        .calls()
        .accept(
            "call_abc",
            CallAcceptParams {
                session: RealtimeSessionCreateRequestParam {
                    model: Some("gpt-realtime".to_owned()),
                    ..Default::default()
                },
            },
        )
        .await;

    assert!(result.is_ok(), "accept should succeed: {result:?}");
}

#[tokio::test]
async fn call_hangup_posts_empty_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/realtime/calls/call_xyz/hangup"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri()),
    )
    .expect("client init");

    let result = client.realtime().calls().hangup("call_xyz").await;
    assert!(result.is_ok(), "hangup should succeed: {result:?}");
}

#[tokio::test]
async fn call_refer_posts_target_uri() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/realtime/calls/call_ref/refer"))
        .and(body_partial_json(json!({
            "target_uri": "tel:+14155550123"
        })))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri()),
    )
    .expect("client init");

    let result = client
        .realtime()
        .calls()
        .refer(
            "call_ref",
            CallReferParams {
                target_uri: "tel:+14155550123".to_owned(),
            },
        )
        .await;

    assert!(result.is_ok(), "refer should succeed: {result:?}");
}

#[tokio::test]
async fn call_reject_posts_status_code() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/realtime/calls/call_rej/reject"))
        .and(body_partial_json(json!({ "status_code": 486 })))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri()),
    )
    .expect("client init");

    let result = client
        .realtime()
        .calls()
        .reject(
            "call_rej",
            CallRejectParams {
                status_code: Some(486),
            },
        )
        .await;

    assert!(result.is_ok(), "reject should succeed: {result:?}");
}

// ---------------------------------------------------------------------------
// ClientSecretService integration tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_secret_create_returns_token() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/realtime/client_secrets"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "sess_token_abc",
            "object": "realtime.session",
            "client_secret": {
                "value": "ek_1234",
                "expires_at": 1700000000
            }
        })))
        .mount(&server)
        .await;

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri()),
    )
    .expect("client init");

    let response: ClientSecretCreateResponse = client
        .realtime()
        .client_secrets()
        .create(ClientSecretCreateParams {
            session: Some(ClientSecretSessionConfig::Realtime(
                RealtimeSessionCreateRequestParam {
                    model: Some("gpt-4o-realtime-preview".to_owned()),
                    ..Default::default()
                },
            )),
            expires_after: None,
        })
        .await
        .expect("create should succeed");

    assert_eq!(response.client_secret.value, "ek_1234");
    assert_eq!(response.client_secret.expires_at, 1_700_000_000);
    assert_eq!(response.id, "sess_token_abc");
}

// ---------------------------------------------------------------------------
// URL construction tests (using RequestConfig directly)
// ---------------------------------------------------------------------------

#[test]
fn ws_url_with_default_config_and_model() {
    let config = RequestConfig::default();
    let params = RealtimeConnectParams::new().with_model("gpt-4o-realtime-preview");
    let url = RealtimeSession::build_ws_url(&config, &params).expect("build URL");
    assert_eq!(
        url.as_str(),
        "wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview"
    );
}

#[test]
fn ws_url_with_custom_base_url() {
    let mut config = RequestConfig::default();
    config.base_url = url::Url::parse("https://my-proxy.example.com/v2/").expect("parse URL");
    let params = RealtimeConnectParams::new().with_model("gpt-realtime");
    let url = RealtimeSession::build_ws_url(&config, &params).expect("build URL");
    assert_eq!(
        url.as_str(),
        "wss://my-proxy.example.com/v2/realtime?model=gpt-realtime"
    );
}

#[test]
fn ws_url_without_model() {
    let config = RequestConfig::default();
    let params = RealtimeConnectParams::new();
    let url = RealtimeSession::build_ws_url(&config, &params).expect("build URL");
    assert_eq!(url.as_str(), "wss://api.openai.com/v1/realtime");
}

// ---------------------------------------------------------------------------
// Auth header verification test (cannot connect, but can verify URL + config)
// ---------------------------------------------------------------------------

#[test]
fn config_auth_header_present_in_connect_params() {
    // Verifies that the config carries the API key that will be injected as
    // an Authorization header during `connect()`.
    let config =
        RequestConfig::from_client_config(ClientConfig::default().with_api_key("sk-test-key"))
            .expect("config");
    assert_eq!(config.api_key.as_deref(), Some("sk-test-key"));
}

// ---------------------------------------------------------------------------
// Service hierarchy wiring test
// ---------------------------------------------------------------------------

#[test]
fn client_realtime_service_hierarchy() {
    let client = Client::new(ClientConfig::default().with_api_key("k")).expect("client");
    // Verify the service hierarchy compiles and returns correct types.
    let _rt = client.realtime();
    let _calls = client.realtime().calls();
    let _secrets = client.realtime().client_secrets();
}

// ---------------------------------------------------------------------------
// Error propagation test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn call_hangup_propagates_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/realtime/calls/bad_id/hangup"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": {
                "message": "Call not found",
                "type": "invalid_request_error",
                "code": "not_found",
                "param": null
            }
        })))
        .mount(&server)
        .await;

    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri()),
    )
    .expect("client init");

    let err = client
        .realtime()
        .calls()
        .hangup("bad_id")
        .await
        .expect_err("should fail");

    match err {
        Error::Api {
            status, message, ..
        } => {
            assert_eq!(status, 404);
            assert!(message.contains("Call not found"));
        }
        other => panic!("expected Api error, got: {other:?}"),
    }
}
