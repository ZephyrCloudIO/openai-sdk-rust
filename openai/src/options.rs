//! Client options and builder helpers.

use std::time::Duration;

/// Builder-style configuration used to construct an API [`crate::Client`].
#[must_use]
#[derive(Clone, Debug, Default)]
pub struct ClientConfig {
    /// OpenAI API key used for Bearer auth.
    pub api_key: Option<String>,
    /// API base URL.
    pub base_url: Option<String>,
    /// Optional organization header.
    pub organization: Option<String>,
    /// Optional project header.
    pub project: Option<String>,
    /// Maximum number of retries for retryable errors.
    pub max_retries: Option<u32>,
    /// Per-request timeout.
    pub timeout: Option<Duration>,
    /// Additional headers applied to every request.
    pub headers: Vec<(String, String)>,
    /// Query params applied to every request.
    pub query_params: Vec<(String, String)>,
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

    /// Adds a static header to every request.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Adds a query parameter to every request.
    pub fn with_query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.push((key.into(), value.into()));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::ClientConfig;
    use std::time::Duration;

    #[test]
    fn builder_accumulates_values() {
        let cfg = ClientConfig::default()
            .with_api_key("k")
            .with_base_url("https://example.com/v1")
            .with_organization("org")
            .with_project("proj")
            .with_retries(5)
            .with_timeout(Duration::from_secs(20))
            .with_header("x-test", "v")
            .with_query_param("api-version", "2025-01-01");

        assert_eq!(cfg.api_key.as_deref(), Some("k"));
        assert_eq!(cfg.base_url.as_deref(), Some("https://example.com/v1"));
        assert_eq!(cfg.organization.as_deref(), Some("org"));
        assert_eq!(cfg.project.as_deref(), Some("proj"));
        assert_eq!(cfg.max_retries, Some(5));
        assert_eq!(cfg.timeout, Some(Duration::from_secs(20)));
        assert_eq!(cfg.headers, vec![("x-test".to_owned(), "v".to_owned())]);
        assert_eq!(
            cfg.query_params,
            vec![("api-version".to_owned(), "2025-01-01".to_owned())]
        );
    }
}
