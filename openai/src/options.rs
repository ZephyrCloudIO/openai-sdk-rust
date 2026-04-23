//! Client options and builder helpers.

use std::{fmt, future::Future, sync::Arc, time::Duration};

use crate::auth::WorkloadIdentity;
use crate::config::{
    HeaderAction, HeaderOperation, JsonAction, JsonOperation, MiddlewareNext, QueryAction,
    QueryOperation, RequestBodyOverride, RequestMiddleware, ResponseBodyCapture, ResponseCapture,
};
use crate::Result;
use serde::Serialize;

/// Builder-style configuration used to construct an API [`crate::Client`].
#[must_use]
#[derive(Clone, Default)]
pub struct ClientConfig {
    /// OpenAI API key used for Bearer auth.
    pub api_key: Option<String>,
    /// Workload identity auth configuration for short-lived token auth.
    pub workload_identity: Option<WorkloadIdentity>,
    /// API base URL.
    pub base_url: Option<String>,
    /// Optional organization header.
    pub organization: Option<String>,
    /// Optional project header.
    pub project: Option<String>,
    /// Optional webhook signing secret.
    pub webhook_secret: Option<String>,
    /// Maximum number of retries for retryable errors.
    pub max_retries: Option<u32>,
    /// Per-request timeout.
    pub timeout: Option<Duration>,
    /// Additional headers applied to every request.
    pub headers: Vec<(String, String)>,
    /// Query params applied to every request.
    pub query_params: Vec<(String, String)>,
    /// Header operations applied to every request.
    pub header_ops: Vec<HeaderOperation>,
    /// Query operations applied to every request.
    pub query_ops: Vec<QueryOperation>,
    /// JSON body mutations applied to every serialized JSON request.
    pub json_ops: Vec<JsonOperation>,
    /// Custom serialized request body override.
    pub request_body: Option<RequestBodyOverride>,
    /// Destination for response metadata capture.
    pub response_capture: Option<ResponseCapture>,
    /// Destination for response body capture.
    pub response_body_capture: Option<ResponseBodyCapture>,
    /// Custom HTTP client.
    pub http_client: Option<reqwest::Client>,
    /// Request middleware chain.
    pub middlewares: Vec<RequestMiddleware>,
}

impl fmt::Debug for ClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientConfig")
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

impl ClientConfig {
    /// Creates a default client config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets API key used for bearer auth.
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Sets workload identity auth and disables static API key auth.
    pub fn with_workload_identity(mut self, workload_identity: WorkloadIdentity) -> Self {
        self.api_key = None;
        self.workload_identity = Some(workload_identity);
        self
    }

    /// Sets base URL, e.g. `https://api.openai.com/v1`.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Sets organization header.
    pub fn with_organization(mut self, organization: impl Into<String>) -> Self {
        self.organization = Some(organization.into());
        self
    }

    /// Sets project header.
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Sets the webhook signing secret used by [`crate::Client::webhooks_from_config`].
    pub fn with_webhook_secret(mut self, webhook_secret: impl Into<String>) -> Self {
        self.webhook_secret = Some(webhook_secret.into());
        self
    }

    /// Sets max retries for retryable statuses (408/409/429/5xx).
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Sets max retries for retryable statuses (408/409/429/5xx).
    pub fn with_retries(self, retries: u32) -> Self {
        self.with_max_retries(retries)
    }

    /// Sets request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets request timeout for each request attempt.
    pub fn with_request_timeout(self, timeout: Duration) -> Self {
        self.with_timeout(timeout)
    }

    /// Adds a static header to every request.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Appends a static header value to every request.
    pub fn with_header_add(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.header_ops.push(HeaderOperation {
            action: HeaderAction::Add,
            key: key.into(),
            value: Some(value.into()),
        });
        self
    }

    /// Deletes a static header from every request.
    pub fn with_header_del(mut self, key: impl Into<String>) -> Self {
        self.header_ops.push(HeaderOperation {
            action: HeaderAction::Delete,
            key: key.into(),
            value: None,
        });
        self
    }

    /// Sets a query parameter on every request.
    pub fn with_query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_ops.push(QueryOperation {
            action: QueryAction::Set,
            key: key.into(),
            value: Some(value.into()),
        });
        self
    }

    /// Adds a query parameter to every request.
    pub fn with_query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.push((key.into(), value.into()));
        self
    }

    /// Appends a query parameter to every request.
    pub fn with_query_add(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_ops.push(QueryOperation {
            action: QueryAction::Add,
            key: key.into(),
            value: Some(value.into()),
        });
        self
    }

    /// Deletes a query parameter from every request.
    pub fn with_query_del(mut self, key: impl Into<String>) -> Self {
        self.query_ops.push(QueryOperation {
            action: QueryAction::Delete,
            key: key.into(),
            value: None,
        });
        self
    }

    /// Sets a value on every serialized JSON request body.
    pub fn with_json_set(mut self, path: impl Into<String>, value: impl Serialize) -> Self {
        let value = serde_json::to_value(value).expect("request JSON override value serializes");
        self.json_ops.push(JsonOperation {
            action: JsonAction::Set,
            path: path.into(),
            value: Some(value),
        });
        self
    }

    /// Deletes a value from every serialized JSON request body.
    pub fn with_json_del(mut self, path: impl Into<String>) -> Self {
        self.json_ops.push(JsonOperation {
            action: JsonAction::Delete,
            path: path.into(),
            value: None,
        });
        self
    }

    /// Overrides the serialized request body.
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
    pub fn with_response_into(mut self, dst: ResponseCapture) -> Self {
        self.response_capture = Some(dst);
        self
    }

    /// Captures response body bytes into the given shared destination.
    pub fn with_response_body_into(mut self, dst: ResponseBodyCapture) -> Self {
        self.response_body_capture = Some(dst);
        self
    }

    /// Sets a custom HTTP client.
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Adds request middleware.
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

#[cfg(test)]
mod tests {
    use super::ClientConfig;
    use crate::auth::{SubjectTokenProvider, SubjectTokenType, WorkloadIdentity};
    use std::sync::Arc;
    use std::time::Duration;

    #[derive(Debug)]
    struct StaticProvider;

    #[async_trait::async_trait]
    impl SubjectTokenProvider for StaticProvider {
        fn token_type(&self) -> SubjectTokenType {
            SubjectTokenType::Jwt
        }

        async fn get_token(
            &self,
            _http_client: &reqwest::Client,
        ) -> std::result::Result<String, crate::auth::AuthError> {
            Ok("subject-token".to_owned())
        }
    }

    #[test]
    fn builder_accumulates_values() {
        let cfg = ClientConfig::default()
            .with_api_key("k")
            .with_base_url("https://example.com/v1")
            .with_organization("org")
            .with_project("proj")
            .with_webhook_secret("whsec_test")
            .with_retries(5)
            .with_timeout(Duration::from_secs(20))
            .with_header("x-test", "v")
            .with_query_param("api-version", "2025-01-01");

        assert_eq!(cfg.api_key.as_deref(), Some("k"));
        assert_eq!(cfg.base_url.as_deref(), Some("https://example.com/v1"));
        assert_eq!(cfg.organization.as_deref(), Some("org"));
        assert_eq!(cfg.project.as_deref(), Some("proj"));
        assert_eq!(cfg.webhook_secret.as_deref(), Some("whsec_test"));
        assert_eq!(cfg.max_retries, Some(5));
        assert_eq!(cfg.timeout, Some(Duration::from_secs(20)));
        assert_eq!(cfg.headers, vec![("x-test".to_owned(), "v".to_owned())]);
        assert_eq!(
            cfg.query_params,
            vec![("api-version".to_owned(), "2025-01-01".to_owned())]
        );
    }

    #[test]
    fn workload_identity_clears_static_key() {
        let cfg = ClientConfig::default()
            .with_api_key("k")
            .with_workload_identity(WorkloadIdentity::new(
                "client",
                "idp",
                "sa",
                Arc::new(StaticProvider),
            ));

        assert!(cfg.api_key.is_none());
        assert!(cfg.workload_identity.is_some());
    }
}
