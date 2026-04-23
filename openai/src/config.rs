//! Resolved request configuration.

use std::{
    fmt,
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};

use futures::future::BoxFuture;
use serde::Serialize;
use serde_json::Value;
use url::Url;

use crate::{
    auth::WorkloadIdentityAuth,
    error::{Error, Result},
    options::ClientConfig,
};

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1/";
const DEFAULT_MAX_RETRIES: u32 = 2;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Continuation passed to request middleware.
pub type MiddlewareNext =
    Arc<dyn Fn(reqwest::Request) -> BoxFuture<'static, Result<reqwest::Response>> + Send + Sync>;

/// Middleware that can inspect or replace the outgoing request and response.
pub type RequestMiddleware = Arc<
    dyn Fn(reqwest::Request, MiddlewareNext) -> BoxFuture<'static, Result<reqwest::Response>>
        + Send
        + Sync,
>;

/// Operation applied to a request header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderOperation {
    /// Operation kind.
    pub action: HeaderAction,
    /// Header name.
    pub key: String,
    /// Header value for set/add operations.
    pub value: Option<String>,
}

/// Header operation kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeaderAction {
    /// Replace any existing value.
    Set,
    /// Append an additional value.
    Add,
    /// Delete all values.
    Delete,
}

/// Operation applied to a request query parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryOperation {
    /// Operation kind.
    pub action: QueryAction,
    /// Query parameter name.
    pub key: String,
    /// Query parameter value for set/add operations.
    pub value: Option<String>,
}

/// Query operation kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryAction {
    /// Replace any existing value.
    Set,
    /// Append an additional value.
    Add,
    /// Delete all values.
    Delete,
}

/// Operation applied to a serialized JSON request body.
#[derive(Clone, Debug, PartialEq)]
pub struct JsonOperation {
    /// Operation kind.
    pub action: JsonAction,
    /// Dot-separated JSON path, matching the common `sjson` request-option form.
    pub path: String,
    /// JSON value for set operations.
    pub value: Option<Value>,
}

/// JSON mutation operation kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonAction {
    /// Set or create a value.
    Set,
    /// Delete an existing value.
    Delete,
}

/// Custom serialized request body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestBodyOverride {
    /// Content type header value.
    pub content_type: String,
    /// Serialized request body.
    pub body: Vec<u8>,
}

/// Captured HTTP response metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedResponse {
    /// HTTP status code.
    pub status: u16,
    /// Final request URL.
    pub url: String,
    /// Response headers as UTF-8 lossy strings.
    pub headers: Vec<(String, String)>,
}

/// Shared destination for response metadata capture.
pub type ResponseCapture = Arc<Mutex<Option<CapturedResponse>>>;

/// Shared destination for response body capture.
pub type ResponseBodyCapture = Arc<Mutex<Option<Vec<u8>>>>;

/// Per-request overrides merged on top of [`RequestConfig`].
#[derive(Clone, Default)]
pub struct RequestOptions {
    /// Override max retries for retryable responses.
    pub max_retries: Option<u32>,
    /// Override request timeout.
    pub timeout: Option<Duration>,
    /// Extra headers for this request.
    pub headers: Vec<(String, String)>,
    /// Extra query params for this request.
    pub query_params: Vec<(String, String)>,
    /// Header operations applied after defaults and static headers.
    pub header_ops: Vec<HeaderOperation>,
    /// Query operations applied after static query params.
    pub query_ops: Vec<QueryOperation>,
    /// JSON body mutations applied after serialization.
    pub json_ops: Vec<JsonOperation>,
    /// Custom serialized request body override.
    pub request_body: Option<RequestBodyOverride>,
    /// Destination for response metadata capture.
    pub response_capture: Option<ResponseCapture>,
    /// Destination for response body capture.
    pub response_body_capture: Option<ResponseBodyCapture>,
    /// Request-local HTTP client override.
    pub http_client: Option<reqwest::Client>,
    /// Request middleware chain.
    pub middlewares: Vec<RequestMiddleware>,
}

impl fmt::Debug for RequestOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequestOptions")
            .field("max_retries", &self.max_retries)
            .field("timeout", &self.timeout)
            .field("headers", &self.headers)
            .field("query_params", &self.query_params)
            .field("header_ops", &self.header_ops)
            .field("query_ops", &self.query_ops)
            .field("json_ops", &self.json_ops)
            .field("request_body", &self.request_body)
            .field("response_capture", &self.response_capture.is_some())
            .field(
                "response_body_capture",
                &self.response_body_capture.is_some(),
            )
            .field("http_client", &self.http_client.is_some())
            .field("middlewares", &self.middlewares.len())
            .finish()
    }
}

impl RequestOptions {
    /// Creates empty request options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides max retries for this request.
    #[must_use]
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Overrides timeout for this request.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Overrides timeout for each request attempt.
    #[must_use]
    pub fn with_request_timeout(self, timeout: Duration) -> Self {
        self.with_timeout(timeout)
    }

    /// Adds a request-local header.
    #[must_use]
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Appends a request-local header value.
    #[must_use]
    pub fn with_header_add(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.header_ops.push(HeaderOperation {
            action: HeaderAction::Add,
            key: key.into(),
            value: Some(value.into()),
        });
        self
    }

    /// Deletes a request-local header.
    #[must_use]
    pub fn with_header_del(mut self, key: impl Into<String>) -> Self {
        self.header_ops.push(HeaderOperation {
            action: HeaderAction::Delete,
            key: key.into(),
            value: None,
        });
        self
    }

    /// Sets a request-local query parameter.
    #[must_use]
    pub fn with_query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_ops.push(QueryOperation {
            action: QueryAction::Set,
            key: key.into(),
            value: Some(value.into()),
        });
        self
    }

    /// Adds a request-local query parameter.
    #[must_use]
    pub fn with_query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.push((key.into(), value.into()));
        self
    }

    /// Appends a request-local query parameter.
    #[must_use]
    pub fn with_query_add(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_ops.push(QueryOperation {
            action: QueryAction::Add,
            key: key.into(),
            value: Some(value.into()),
        });
        self
    }

    /// Deletes a request-local query parameter.
    #[must_use]
    pub fn with_query_del(mut self, key: impl Into<String>) -> Self {
        self.query_ops.push(QueryOperation {
            action: QueryAction::Delete,
            key: key.into(),
            value: None,
        });
        self
    }

    /// Sets a value on the serialized JSON body.
    #[must_use]
    pub fn with_json_set(mut self, path: impl Into<String>, value: impl Serialize) -> Self {
        let value = serde_json::to_value(value).expect("request JSON override value serializes");
        self.json_ops.push(JsonOperation {
            action: JsonAction::Set,
            path: path.into(),
            value: Some(value),
        });
        self
    }

    /// Deletes a value from the serialized JSON body.
    #[must_use]
    pub fn with_json_del(mut self, path: impl Into<String>) -> Self {
        self.json_ops.push(JsonOperation {
            action: JsonAction::Delete,
            path: path.into(),
            value: None,
        });
        self
    }

    /// Overrides the serialized request body.
    #[must_use]
    pub fn with_request_body(
        mut self,
        content_type: impl Into<String>,
        body: impl Into<Vec<u8>>,
    ) -> Self {
        self.request_body = Some(RequestBodyOverride {
            content_type: content_type.into(),
            body: body.into(),
        });
        self
    }

    /// Captures response metadata into the given shared destination.
    #[must_use]
    pub fn with_response_into(mut self, dst: ResponseCapture) -> Self {
        self.response_capture = Some(dst);
        self
    }

    /// Captures response body bytes into the given shared destination.
    #[must_use]
    pub fn with_response_body_into(mut self, dst: ResponseBodyCapture) -> Self {
        self.response_body_capture = Some(dst);
        self
    }

    /// Overrides the HTTP client for this request.
    #[must_use]
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Adds request middleware.
    #[must_use]
    pub fn with_middleware<F, Fut>(mut self, middleware: F) -> Self
    where
        F: Fn(reqwest::Request, MiddlewareNext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<reqwest::Response>> + Send + 'static,
    {
        self.middlewares.push(Arc::new(move |request, next| {
            Box::pin(middleware(request, next))
        }));
        self
    }
}

/// Fully-resolved request configuration derived from [`ClientConfig`].
#[derive(Clone)]
pub struct RequestConfig {
    /// API key used for bearer auth.
    pub api_key: Option<String>,
    /// Workload identity auth state.
    pub workload_identity: Option<WorkloadIdentityAuth>,
    /// Normalized base URL with a trailing slash.
    pub base_url: Url,
    /// Optional OpenAI org ID.
    pub organization: Option<String>,
    /// Optional OpenAI project ID.
    pub project: Option<String>,
    /// Optional webhook signing secret.
    pub webhook_secret: Option<String>,
    /// Max retries for retryable responses.
    pub max_retries: u32,
    /// Request timeout.
    pub timeout: Duration,
    /// Additional request headers.
    pub headers: Vec<(String, String)>,
    /// Additional request query params.
    pub query_params: Vec<(String, String)>,
    /// Header operations applied after defaults and static headers.
    pub header_ops: Vec<HeaderOperation>,
    /// Query operations applied after static query params.
    pub query_ops: Vec<QueryOperation>,
    /// JSON body mutations applied after serialization.
    pub json_ops: Vec<JsonOperation>,
    /// Custom serialized request body override.
    pub request_body: Option<RequestBodyOverride>,
    /// Destination for response metadata capture.
    pub response_capture: Option<ResponseCapture>,
    /// Destination for response body capture.
    pub response_body_capture: Option<ResponseBodyCapture>,
    /// HTTP client override.
    pub http_client: Option<reqwest::Client>,
    /// Request middleware chain.
    pub middlewares: Vec<RequestMiddleware>,
}

impl fmt::Debug for RequestConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequestConfig")
            .field("api_key", &self.api_key.as_ref().map(|_| "<redacted>"))
            .field("workload_identity", &self.workload_identity.is_some())
            .field("base_url", &self.base_url)
            .field("organization", &self.organization)
            .field("project", &self.project)
            .field(
                "webhook_secret",
                &self.webhook_secret.as_ref().map(|_| "<redacted>"),
            )
            .field("max_retries", &self.max_retries)
            .field("timeout", &self.timeout)
            .field("headers", &self.headers)
            .field("query_params", &self.query_params)
            .field("header_ops", &self.header_ops)
            .field("query_ops", &self.query_ops)
            .field("json_ops", &self.json_ops)
            .field("request_body", &self.request_body)
            .field("response_capture", &self.response_capture.is_some())
            .field(
                "response_body_capture",
                &self.response_body_capture.is_some(),
            )
            .field("http_client", &self.http_client.is_some())
            .field("middlewares", &self.middlewares.len())
            .finish()
    }
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            workload_identity: None,
            base_url: std::env::var("OPENAI_BASE_URL")
                .ok()
                .and_then(|url| Self::parse_base_url(&url).ok())
                .unwrap_or_else(|| {
                    Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid")
                }),
            organization: std::env::var("OPENAI_ORG_ID").ok(),
            project: std::env::var("OPENAI_PROJECT_ID").ok(),
            webhook_secret: std::env::var("OPENAI_WEBHOOK_SECRET").ok(),
            max_retries: DEFAULT_MAX_RETRIES,
            timeout: DEFAULT_TIMEOUT,
            headers: Vec::new(),
            query_params: Vec::new(),
            header_ops: Vec::new(),
            query_ops: Vec::new(),
            json_ops: Vec::new(),
            request_body: None,
            response_capture: None,
            response_body_capture: None,
            http_client: None,
            middlewares: Vec::new(),
        }
    }
}

impl RequestConfig {
    /// Builds a resolved config from user-provided options.
    pub fn from_client_config(value: ClientConfig) -> Result<Self> {
        Self::default().merge_client_config(&value)
    }

    /// Merges client-level overrides onto this request config.
    pub fn merge_client_config(&self, value: &ClientConfig) -> Result<Self> {
        let mut merged = self.clone();

        if let Some(workload_identity) = value.workload_identity.clone() {
            merged.api_key = None;
            merged.workload_identity = Some(WorkloadIdentityAuth::new(workload_identity)?);
        } else {
            merged.api_key = value.api_key.clone().or_else(|| self.api_key.clone());
            merged.workload_identity = self.workload_identity.clone();
        }
        merged.base_url = match value.base_url.as_deref() {
            Some(base_url) => Self::parse_base_url(base_url)?,
            None => self.base_url.clone(),
        };
        merged.organization = value
            .organization
            .clone()
            .or_else(|| self.organization.clone());
        merged.project = value.project.clone().or_else(|| self.project.clone());
        merged.webhook_secret = value
            .webhook_secret
            .clone()
            .or_else(|| self.webhook_secret.clone());
        merged.max_retries = value.max_retries.unwrap_or(self.max_retries);
        merged.timeout = value.timeout.unwrap_or(self.timeout);
        merged.headers.extend(value.headers.iter().cloned());
        merged
            .query_params
            .extend(value.query_params.iter().cloned());
        merged.header_ops.extend(value.header_ops.iter().cloned());
        merged.query_ops.extend(value.query_ops.iter().cloned());
        merged.json_ops.extend(value.json_ops.iter().cloned());
        if value.request_body.is_some() {
            merged.request_body = value.request_body.clone();
        }
        if value.response_capture.is_some() {
            merged.response_capture = value.response_capture.clone();
        }
        if value.response_body_capture.is_some() {
            merged.response_body_capture = value.response_body_capture.clone();
        }
        if value.http_client.is_some() {
            merged.http_client = value.http_client.clone();
        }
        merged.middlewares.extend(value.middlewares.iter().cloned());

        Ok(merged)
    }

    /// Merges request-local overrides with this config.
    #[must_use]
    pub fn merge_request_options(&self, options: &RequestOptions) -> Self {
        let mut merged = self.clone();
        merged.max_retries = options.max_retries.unwrap_or(self.max_retries);
        merged.timeout = options.timeout.unwrap_or(self.timeout);
        merged.headers.extend(options.headers.iter().cloned());
        merged
            .query_params
            .extend(options.query_params.iter().cloned());
        merged.header_ops.extend(options.header_ops.iter().cloned());
        merged.query_ops.extend(options.query_ops.iter().cloned());
        merged.json_ops.extend(options.json_ops.iter().cloned());
        if options.request_body.is_some() {
            merged.request_body = options.request_body.clone();
        }
        if options.response_capture.is_some() {
            merged.response_capture = options.response_capture.clone();
        }
        if options.response_body_capture.is_some() {
            merged.response_body_capture = options.response_body_capture.clone();
        }
        if options.http_client.is_some() {
            merged.http_client = options.http_client.clone();
        }
        merged
            .middlewares
            .extend(options.middlewares.iter().cloned());
        merged
    }

    /// Resolves an endpoint path against the base URL.
    pub fn endpoint_url(&self, path: &str) -> Result<Url> {
        self.base_url
            .join(path.trim_start_matches('/'))
            .map_err(|err| Error::Config {
                field: "path",
                message: err.to_string(),
            })
    }

    fn parse_base_url(base_url: &str) -> Result<Url> {
        let mut parsed = Url::parse(base_url).map_err(|err| Error::Config {
            field: "base_url",
            message: err.to_string(),
        })?;
        if !parsed.path().ends_with('/') {
            let mut path = parsed.path().to_owned();
            path.push('/');
            parsed.set_path(&path);
        }
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::{RequestConfig, RequestOptions};
    use crate::options::ClientConfig;
    use std::time::Duration;

    #[test]
    fn client_config_defaults_are_applied() {
        let cfg = RequestConfig::from_client_config(ClientConfig::default())
            .expect("default config should resolve");

        assert_eq!(cfg.base_url.as_str(), "https://api.openai.com/v1/");
        assert_eq!(cfg.max_retries, 2);
        assert_eq!(cfg.timeout, Duration::from_secs(60));
        assert!(cfg.headers.is_empty());
        assert!(cfg.query_params.is_empty());
    }

    #[test]
    fn merge_client_config_overrides_values_and_appends_collections() {
        let base = RequestConfig::default();
        let cfg = ClientConfig::default()
            .with_api_key("k")
            .with_base_url("https://example.com/v1")
            .with_organization("org")
            .with_project("proj")
            .with_webhook_secret("whsec_test")
            .with_max_retries(7)
            .with_timeout(Duration::from_secs(15))
            .with_header("x-global", "1")
            .with_query_param("version", "2025-01-01");

        let merged = base
            .merge_client_config(&cfg)
            .expect("merge should produce config");

        assert_eq!(merged.api_key.as_deref(), Some("k"));
        assert_eq!(merged.base_url.as_str(), "https://example.com/v1/");
        assert_eq!(merged.organization.as_deref(), Some("org"));
        assert_eq!(merged.project.as_deref(), Some("proj"));
        assert_eq!(merged.webhook_secret.as_deref(), Some("whsec_test"));
        assert_eq!(merged.max_retries, 7);
        assert_eq!(merged.timeout, Duration::from_secs(15));
        assert_eq!(
            merged.headers,
            vec![("x-global".to_owned(), "1".to_owned())]
        );
        assert_eq!(
            merged.query_params,
            vec![("version".to_owned(), "2025-01-01".to_owned())]
        );
    }

    #[test]
    fn merge_request_options_overrides_retry_timeout_and_extends_headers() {
        let base = RequestConfig::from_client_config(
            ClientConfig::default()
                .with_header("x-global", "1")
                .with_query_param("global", "yes"),
        )
        .expect("client config should resolve");
        let options = RequestOptions::new()
            .with_max_retries(5)
            .with_timeout(Duration::from_secs(5))
            .with_header("x-request", "2")
            .with_query_param("request", "yes");

        let merged = base.merge_request_options(&options);

        assert_eq!(merged.max_retries, 5);
        assert_eq!(merged.timeout, Duration::from_secs(5));
        assert_eq!(
            merged.headers,
            vec![
                ("x-global".to_owned(), "1".to_owned()),
                ("x-request".to_owned(), "2".to_owned())
            ]
        );
        assert_eq!(
            merged.query_params,
            vec![
                ("global".to_owned(), "yes".to_owned()),
                ("request".to_owned(), "yes".to_owned())
            ]
        );
    }

    #[test]
    fn endpoint_url_trims_leading_slash() {
        let cfg = RequestConfig::default();
        let url = cfg.endpoint_url("/models").expect("valid path");
        assert_eq!(url.as_str(), "https://api.openai.com/v1/models");
    }
}
