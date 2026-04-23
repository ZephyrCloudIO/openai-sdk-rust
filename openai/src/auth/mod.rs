//! Authentication helpers for short-lived token and workload identity flows.

use std::{
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, Notify};

use crate::shared::OAuthErrorCode;

/// Token exchange grant type used by workload identity authentication.
pub const TOKEN_EXCHANGE_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:token-exchange";
/// OAuth JWT subject token type URN.
pub const JWT_TOKEN_TYPE: &str = "urn:ietf:params:oauth:token-type:jwt";
/// OAuth ID token subject token type URN.
pub const ID_TOKEN_TYPE: &str = "urn:ietf:params:oauth:token-type:id_token";
/// Default Kubernetes service account token path.
pub const DEFAULT_K8S_TOKEN_PATH: &str = "/var/run/secrets/kubernetes.io/serviceaccount/token";
/// Default GCP audience.
pub const DEFAULT_AUDIENCE: &str = "https://api.openai.com/v1";
/// Default Azure managed identity resource.
pub const DEFAULT_AZURE_RESOURCE: &str = "https://management.azure.com/";
/// Default Azure IMDS API version.
pub const DEFAULT_AZURE_API_VERSION: &str = "2018-02-01";
/// Default token exchange URL.
pub const TOKEN_EXCHANGE_URL: &str = "https://auth.openai.com/oauth/token";

const DEFAULT_TOKEN_EXPIRY: Duration = Duration::from_secs(60 * 60);
const DEFAULT_REFRESH_BUFFER: Duration = Duration::from_secs(20 * 60);

/// Authentication error type.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// A cloud/provider subject token could not be acquired.
    #[error(transparent)]
    SubjectTokenProvider(#[from] SubjectTokenProviderError),

    /// OAuth token exchange failed with an RFC-defined error.
    #[error(transparent)]
    OAuth(#[from] OAuthError),

    /// Workload identity config is invalid.
    #[error("WorkloadIdentity: {field} {message}")]
    WorkloadIdentityConfig {
        /// Invalid field name.
        field: &'static str,
        /// Validation message.
        message: String,
    },

    /// Subject token type is not supported by the token exchange flow.
    #[error("unsupported subject token type {0}")]
    UnsupportedSubjectTokenType(SubjectTokenType),

    /// Token exchange request failed.
    #[error("failed to exchange token: {0}")]
    TokenExchange(String),
}

/// Error raised when a cloud subject token provider fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectTokenProviderError {
    /// Provider name.
    pub provider: String,
    /// Provider-specific message.
    pub message: String,
}

impl SubjectTokenProviderError {
    /// Creates a subject token provider error.
    #[must_use]
    pub fn new(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for SubjectTokenProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} provider error: {}", self.provider, self.message)
    }
}

impl std::error::Error for SubjectTokenProviderError {}

/// Error raised when OAuth token exchange returns an OAuth error payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthError {
    /// HTTP status code.
    pub status_code: u16,
    /// OAuth error code.
    pub error_code: OAuthErrorCode,
    /// OAuth error description.
    pub error_description: String,
}

impl fmt::Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OAuth error (status {}): {} - {}",
            self.status_code, self.error_code, self.error_description
        )
    }
}

impl std::error::Error for OAuthError {}

/// Subject token type supplied by a workload identity provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectTokenType {
    /// JWT subject token.
    Jwt,
    /// ID token subject token.
    Id,
}

impl fmt::Display for SubjectTokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Jwt => f.write_str("jwt"),
            Self::Id => f.write_str("id"),
        }
    }
}

/// Provider of a cloud subject token for workload identity token exchange.
#[async_trait::async_trait]
pub trait SubjectTokenProvider: Send + Sync {
    /// Returns the subject token type.
    fn token_type(&self) -> SubjectTokenType;

    /// Retrieves a subject token.
    async fn get_token(&self, http_client: &reqwest::Client) -> Result<String, AuthError>;
}

/// Workload identity configuration.
#[derive(Clone)]
pub struct WorkloadIdentity {
    /// OAuth client ID.
    pub client_id: String,
    /// Identity provider ID.
    pub identity_provider_id: String,
    /// Service account ID.
    pub service_account_id: String,
    /// Cloud subject token provider.
    pub provider: Arc<dyn SubjectTokenProvider>,
    /// Refresh buffer before token expiry. Defaults to 20 minutes.
    pub refresh_buffer: Option<Duration>,
    /// Token exchange URL. Defaults to [`TOKEN_EXCHANGE_URL`].
    pub token_exchange_url: Option<String>,
}

impl WorkloadIdentity {
    /// Creates workload identity configuration.
    #[must_use]
    pub fn new(
        client_id: impl Into<String>,
        identity_provider_id: impl Into<String>,
        service_account_id: impl Into<String>,
        provider: Arc<dyn SubjectTokenProvider>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            identity_provider_id: identity_provider_id.into(),
            service_account_id: service_account_id.into(),
            provider,
            refresh_buffer: None,
            token_exchange_url: None,
        }
    }

    /// Sets the refresh buffer.
    #[must_use]
    pub fn with_refresh_buffer(mut self, refresh_buffer: Duration) -> Self {
        self.refresh_buffer = Some(refresh_buffer);
        self
    }

    /// Overrides the token exchange URL.
    #[must_use]
    pub fn with_token_exchange_url(mut self, token_exchange_url: impl Into<String>) -> Self {
        self.token_exchange_url = Some(token_exchange_url.into());
        self
    }
}

impl fmt::Debug for WorkloadIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkloadIdentity")
            .field("client_id", &self.client_id)
            .field("identity_provider_id", &self.identity_provider_id)
            .field("service_account_id", &self.service_account_id)
            .field("refresh_buffer", &self.refresh_buffer)
            .field("token_exchange_url", &self.token_exchange_url)
            .finish_non_exhaustive()
    }
}

/// Response returned by OAuth token exchange.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenExchangeResponse {
    /// Access token to use for API authentication.
    pub access_token: String,
    /// Issued token type.
    pub issued_token_type: Option<String>,
    /// Token type.
    pub token_type: Option<String>,
    /// Expiry in seconds.
    pub expires_in: Option<u64>,
}

#[derive(Debug, Serialize)]
struct TokenExchangeRequest<'a> {
    grant_type: &'static str,
    client_id: &'a str,
    subject_token: &'a str,
    subject_token_type: &'static str,
    identity_provider_id: &'a str,
    service_account_id: &'a str,
}

#[derive(Debug, Deserialize)]
struct OAuthErrorBody {
    error: OAuthErrorCode,
    #[serde(default)]
    error_description: String,
}

#[derive(Debug, Default)]
struct TokenState {
    cached_token: Option<String>,
    token_expiry: Option<Instant>,
    refresh_in_flight: bool,
}

/// Workload identity token exchanger/cache.
#[derive(Clone, Debug)]
pub struct WorkloadIdentityAuth {
    config: WorkloadIdentity,
    state: Arc<Mutex<TokenState>>,
    notify: Arc<Notify>,
}

impl WorkloadIdentityAuth {
    /// Creates workload identity auth after validating config.
    pub fn new(config: WorkloadIdentity) -> Result<Self, AuthError> {
        if config.client_id.is_empty() {
            return Err(AuthError::WorkloadIdentityConfig {
                field: "ClientID",
                message: "is required".to_owned(),
            });
        }
        if config.identity_provider_id.is_empty() {
            return Err(AuthError::WorkloadIdentityConfig {
                field: "IdentityProviderID",
                message: "is required".to_owned(),
            });
        }
        if config.service_account_id.is_empty() {
            return Err(AuthError::WorkloadIdentityConfig {
                field: "ServiceAccountID",
                message: "is required".to_owned(),
            });
        }

        Ok(Self {
            config,
            state: Arc::new(Mutex::new(TokenState::default())),
            notify: Arc::new(Notify::new()),
        })
    }

    /// Returns a cached token or exchanges a fresh token.
    pub async fn get_token(&self, http_client: &reqwest::Client) -> Result<String, AuthError> {
        loop {
            let mut state = self.state.lock().await;
            let now = Instant::now();

            if let (Some(token), Some(expiry)) = (state.cached_token.as_ref(), state.token_expiry) {
                if now < expiry {
                    let token = token.clone();
                    let refresh_buffer =
                        self.config.refresh_buffer.unwrap_or(DEFAULT_REFRESH_BUFFER);
                    let refresh_time = expiry.checked_sub(refresh_buffer).unwrap_or(now);
                    if now >= refresh_time && !state.refresh_in_flight {
                        state.refresh_in_flight = true;
                        let auth = self.clone();
                        let http_client = http_client.clone();
                        tokio::spawn(async move {
                            let _ = auth.refresh_and_store(&http_client).await;
                        });
                    }
                    return Ok(token);
                }
            }

            if state.refresh_in_flight {
                let notified = self.notify.notified();
                drop(state);
                notified.await;
                continue;
            }

            state.refresh_in_flight = true;
            drop(state);
            return self.refresh_and_store(http_client).await;
        }
    }

    /// Invalidates the cached access token.
    pub async fn invalidate_token(&self) {
        let mut state = self.state.lock().await;
        state.cached_token = None;
        state.token_expiry = None;
    }

    async fn refresh_and_store(&self, http_client: &reqwest::Client) -> Result<String, AuthError> {
        let result = self.refresh_token(http_client).await;
        let mut state = self.state.lock().await;
        state.refresh_in_flight = false;

        if let Ok(token) = &result {
            state.cached_token = Some(token.access_token.clone());
            state.token_expiry = Some(
                Instant::now()
                    + Duration::from_secs(
                        token.expires_in.unwrap_or(DEFAULT_TOKEN_EXPIRY.as_secs()),
                    ),
            );
        }

        self.notify.notify_waiters();
        result.map(|token| token.access_token)
    }

    async fn refresh_token(
        &self,
        http_client: &reqwest::Client,
    ) -> Result<TokenExchangeResponse, AuthError> {
        let subject_token = self.config.provider.get_token(http_client).await?;
        let subject_token_type = match self.config.provider.token_type() {
            SubjectTokenType::Jwt => JWT_TOKEN_TYPE,
            SubjectTokenType::Id => ID_TOKEN_TYPE,
        };

        let request = TokenExchangeRequest {
            grant_type: TOKEN_EXCHANGE_GRANT_TYPE,
            client_id: &self.config.client_id,
            subject_token: &subject_token,
            subject_token_type,
            identity_provider_id: &self.config.identity_provider_id,
            service_account_id: &self.config.service_account_id,
        };
        let token_exchange_url = self
            .config
            .token_exchange_url
            .as_deref()
            .unwrap_or(TOKEN_EXCHANGE_URL);

        let response = http_client
            .post(token_exchange_url)
            .json(&request)
            .send()
            .await
            .map_err(|err| AuthError::TokenExchange(err.to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|err| AuthError::TokenExchange(err.to_string()))?;

        if matches!(
            status,
            StatusCode::BAD_REQUEST | StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        ) {
            if let Ok(oauth_error) = serde_json::from_str::<OAuthErrorBody>(&body) {
                return Err(OAuthError {
                    status_code: status.as_u16(),
                    error_code: oauth_error.error,
                    error_description: oauth_error.error_description,
                }
                .into());
            }
        }

        if status != StatusCode::OK {
            return Err(AuthError::TokenExchange(format!(
                "token exchange failed with status {}: {}",
                status.as_u16(),
                body
            )));
        }

        let token: TokenExchangeResponse =
            serde_json::from_str(&body).map_err(|err| AuthError::TokenExchange(err.to_string()))?;
        if token.access_token.is_empty() {
            return Err(AuthError::TokenExchange(format!(
                "token exchange response missing 'access_token' field. Response: {body}"
            )));
        }
        Ok(token)
    }
}

/// Kubernetes service account token provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K8sServiceAccountTokenProvider {
    token_path: PathBuf,
}

impl K8sServiceAccountTokenProvider {
    /// Creates a provider reading from the given token path.
    #[must_use]
    pub fn new(token_path: impl Into<PathBuf>) -> Self {
        let token_path = token_path.into();
        if token_path.as_os_str().is_empty() {
            Self::default()
        } else {
            Self { token_path }
        }
    }

    /// Returns the configured token path.
    #[must_use]
    pub fn token_path(&self) -> &Path {
        &self.token_path
    }
}

impl Default for K8sServiceAccountTokenProvider {
    fn default() -> Self {
        Self {
            token_path: PathBuf::from(DEFAULT_K8S_TOKEN_PATH),
        }
    }
}

#[async_trait::async_trait]
impl SubjectTokenProvider for K8sServiceAccountTokenProvider {
    fn token_type(&self) -> SubjectTokenType {
        SubjectTokenType::Jwt
    }

    async fn get_token(&self, _http_client: &reqwest::Client) -> Result<String, AuthError> {
        let data = tokio::fs::read_to_string(&self.token_path)
            .await
            .map_err(|err| {
                SubjectTokenProviderError::new(
                    "kubernetes",
                    format!(
                        "failed to read service account token from {}: {err}",
                        self.token_path.display()
                    ),
                )
            })?;
        let token = data.trim().to_owned();
        if token.is_empty() {
            return Err(SubjectTokenProviderError::new(
                "kubernetes",
                "service account token is empty",
            )
            .into());
        }
        Ok(token)
    }
}

/// Azure managed identity token provider config.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AzureManagedIdentityTokenProviderConfig {
    /// Azure resource.
    pub resource: Option<String>,
    /// Azure object ID.
    pub object_id: Option<String>,
    /// Azure client ID.
    pub client_id: Option<String>,
    /// Azure MSI resource ID.
    pub msi_res_id: Option<String>,
    /// Azure IMDS API version.
    pub api_version: Option<String>,
}

/// Azure managed identity token provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AzureManagedIdentityTokenProvider {
    config: AzureManagedIdentityTokenProviderConfig,
}

impl AzureManagedIdentityTokenProvider {
    /// Creates an Azure managed identity token provider.
    #[must_use]
    pub fn new(config: AzureManagedIdentityTokenProviderConfig) -> Self {
        Self { config }
    }
}

impl Default for AzureManagedIdentityTokenProvider {
    fn default() -> Self {
        Self::new(AzureManagedIdentityTokenProviderConfig::default())
    }
}

#[async_trait::async_trait]
impl SubjectTokenProvider for AzureManagedIdentityTokenProvider {
    fn token_type(&self) -> SubjectTokenType {
        SubjectTokenType::Jwt
    }

    async fn get_token(&self, http_client: &reqwest::Client) -> Result<String, AuthError> {
        let mut endpoint = url::Url::parse("http://169.254.169.254/metadata/identity/oauth2/token")
            .map_err(|err| SubjectTokenProviderError::new("azure-imds", err.to_string()))?;
        endpoint
            .query_pairs_mut()
            .append_pair(
                "api-version",
                self.config
                    .api_version
                    .as_deref()
                    .unwrap_or(DEFAULT_AZURE_API_VERSION),
            )
            .append_pair(
                "resource",
                self.config
                    .resource
                    .as_deref()
                    .unwrap_or(DEFAULT_AZURE_RESOURCE),
            );
        if let Some(object_id) = self.config.object_id.as_deref() {
            endpoint
                .query_pairs_mut()
                .append_pair("object_id", object_id);
        }
        if let Some(client_id) = self.config.client_id.as_deref() {
            endpoint
                .query_pairs_mut()
                .append_pair("client_id", client_id);
        }
        if let Some(msi_res_id) = self.config.msi_res_id.as_deref() {
            endpoint
                .query_pairs_mut()
                .append_pair("msi_res_id", msi_res_id);
        }

        let response = http_client
            .get(endpoint)
            .header("Metadata", "true")
            .send()
            .await
            .map_err(|err| {
                SubjectTokenProviderError::new(
                    "azure-imds",
                    format!("failed to fetch token from IMDS: {err}"),
                )
            })?;
        let status = response.status();
        let body = response.text().await.map_err(|err| {
            SubjectTokenProviderError::new("azure-imds", format!("failed to read body: {err}"))
        })?;
        if status != StatusCode::OK {
            return Err(SubjectTokenProviderError::new(
                "azure-imds",
                format!("IMDS returned status {}: {body}", status.as_u16()),
            )
            .into());
        }

        #[derive(Deserialize)]
        struct AzureTokenResponse {
            access_token: String,
        }

        let token: AzureTokenResponse = serde_json::from_str(&body).map_err(|err| {
            SubjectTokenProviderError::new(
                "azure-imds",
                format!("failed to decode IMDS response: {err}"),
            )
        })?;
        if token.access_token.is_empty() {
            return Err(SubjectTokenProviderError::new(
                "azure-imds",
                "IMDS response missing 'access_token' field",
            )
            .into());
        }
        Ok(token.access_token)
    }
}

/// GCP ID token provider config.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GcpIdTokenProviderConfig {
    /// GCP ID token audience.
    pub audience: Option<String>,
}

/// GCP ID token provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcpIdTokenProvider {
    config: GcpIdTokenProviderConfig,
}

impl GcpIdTokenProvider {
    /// Creates a GCP ID token provider.
    #[must_use]
    pub fn new(config: GcpIdTokenProviderConfig) -> Self {
        Self { config }
    }
}

impl Default for GcpIdTokenProvider {
    fn default() -> Self {
        Self::new(GcpIdTokenProviderConfig::default())
    }
}

#[async_trait::async_trait]
impl SubjectTokenProvider for GcpIdTokenProvider {
    fn token_type(&self) -> SubjectTokenType {
        SubjectTokenType::Id
    }

    async fn get_token(&self, http_client: &reqwest::Client) -> Result<String, AuthError> {
        let mut endpoint = url::Url::parse(
            "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/identity",
        )
        .map_err(|err| SubjectTokenProviderError::new("gcp-metadata", err.to_string()))?;
        endpoint.query_pairs_mut().append_pair(
            "audience",
            self.config.audience.as_deref().unwrap_or(DEFAULT_AUDIENCE),
        );

        let response = http_client
            .get(endpoint)
            .header("Metadata-Flavor", "Google")
            .send()
            .await
            .map_err(|err| {
                SubjectTokenProviderError::new(
                    "gcp-metadata",
                    format!("failed to fetch token from metadata server: {err}"),
                )
            })?;
        let status = response.status();
        let body = response.text().await.map_err(|err| {
            SubjectTokenProviderError::new("gcp-metadata", format!("failed to read body: {err}"))
        })?;
        if status != StatusCode::OK {
            return Err(SubjectTokenProviderError::new(
                "gcp-metadata",
                format!(
                    "metadata server returned status {}: {body}",
                    status.as_u16()
                ),
            )
            .into());
        }

        let token = body.trim().to_owned();
        if token.is_empty() {
            return Err(SubjectTokenProviderError::new(
                "gcp-metadata",
                "metadata server returned empty token",
            )
            .into());
        }
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use wiremock::{
        matchers::{body_json, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[derive(Debug)]
    struct MockProvider {
        token: String,
        token_type: SubjectTokenType,
        call_count: AtomicUsize,
    }

    impl MockProvider {
        fn new(token: impl Into<String>, token_type: SubjectTokenType) -> Self {
            Self {
                token: token.into(),
                token_type,
                call_count: AtomicUsize::new(0),
            }
        }

        fn call_count(&self) -> usize {
            self.call_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl SubjectTokenProvider for MockProvider {
        fn token_type(&self) -> SubjectTokenType {
            self.token_type
        }

        async fn get_token(&self, _http_client: &reqwest::Client) -> Result<String, AuthError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(self.token.clone())
        }
    }

    #[test]
    fn provider_defaults_match_go_sdk() {
        let k8s = K8sServiceAccountTokenProvider::default();
        assert_eq!(k8s.token_path(), Path::new(DEFAULT_K8S_TOKEN_PATH));
        assert_eq!(k8s.token_type(), SubjectTokenType::Jwt);

        let azure = AzureManagedIdentityTokenProvider::default();
        assert_eq!(azure.token_type(), SubjectTokenType::Jwt);

        let gcp = GcpIdTokenProvider::default();
        assert_eq!(gcp.token_type(), SubjectTokenType::Id);
    }

    #[tokio::test]
    async fn k8s_provider_reads_trimmed_token() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("openai-rs-k8s-token-{}", std::process::id()));
        tokio::fs::write(&path, "  test-jwt-token\n")
            .await
            .expect("write token");

        let provider = K8sServiceAccountTokenProvider::new(path.clone());
        let token = provider
            .get_token(&reqwest::Client::new())
            .await
            .expect("get token");

        assert_eq!(token, "test-jwt-token");
        assert_eq!(provider.token_type(), SubjectTokenType::Jwt);
        let _ = tokio::fs::remove_file(path).await;
    }

    #[tokio::test]
    async fn workload_identity_caches_exchanged_token() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(body_json(serde_json::json!({
                "grant_type": TOKEN_EXCHANGE_GRANT_TYPE,
                "client_id": "client-id",
                "subject_token": "subject-token",
                "subject_token_type": JWT_TOKEN_TYPE,
                "identity_provider_id": "idp-id",
                "service_account_id": "sa-id"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "access-token",
                "expires_in": 3600
            })))
            .expect(1)
            .mount(&server)
            .await;

        let provider = Arc::new(MockProvider::new("subject-token", SubjectTokenType::Jwt));
        let auth = WorkloadIdentityAuth::new(
            WorkloadIdentity::new("client-id", "idp-id", "sa-id", provider.clone())
                .with_token_exchange_url(format!("{}/oauth/token", server.uri())),
        )
        .expect("auth config");
        let http_client = reqwest::Client::new();

        let first = auth.get_token(&http_client).await.expect("first token");
        let second = auth.get_token(&http_client).await.expect("second token");

        assert_eq!(first, "access-token");
        assert_eq!(second, "access-token");
        assert_eq!(provider.call_count(), 1);
    }

    #[tokio::test]
    async fn workload_identity_oauth_error_is_typed() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "Token exchange failed"
            })))
            .mount(&server)
            .await;

        let provider = Arc::new(MockProvider::new("subject-token", SubjectTokenType::Jwt));
        let auth = WorkloadIdentityAuth::new(
            WorkloadIdentity::new("client-id", "idp-id", "sa-id", provider)
                .with_token_exchange_url(format!("{}/oauth/token", server.uri())),
        )
        .expect("auth config");
        let err = auth
            .get_token(&reqwest::Client::new())
            .await
            .expect_err("token exchange should fail");

        match err {
            AuthError::OAuth(err) => {
                assert_eq!(err.status_code, 400);
                assert_eq!(err.error_code, OAuthErrorCode::InvalidGrant);
                assert_eq!(err.error_description, "Token exchange failed");
            }
            other => panic!("expected OAuth error, got {other:?}"),
        }
    }
}
