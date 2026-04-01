//! Resolved request configuration.

use std::time::Duration;

use url::Url;

use crate::{
    error::{Error, Result},
    options::ClientConfig,
};

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1/";
const DEFAULT_MAX_RETRIES: u32 = 2;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Per-request overrides merged on top of [`RequestConfig`].
#[derive(Clone, Debug, Default)]
pub struct RequestOptions {
    /// Override max retries for retryable responses.
    pub max_retries: Option<u32>,
    /// Override request timeout.
    pub timeout: Option<Duration>,
    /// Extra headers for this request.
    pub headers: Vec<(String, String)>,
    /// Extra query params for this request.
    pub query_params: Vec<(String, String)>,
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

    /// Adds a request-local header.
    #[must_use]
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Adds a request-local query parameter.
    #[must_use]
    pub fn with_query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.push((key.into(), value.into()));
        self
    }
}

/// Fully-resolved request configuration derived from [`ClientConfig`].
#[derive(Clone, Debug)]
pub struct RequestConfig {
    /// API key used for bearer auth.
    pub api_key: Option<String>,
    /// Normalized base URL with a trailing slash.
    pub base_url: Url,
    /// Optional OpenAI org ID.
    pub organization: Option<String>,
    /// Optional OpenAI project ID.
    pub project: Option<String>,
    /// Max retries for retryable responses.
    pub max_retries: u32,
    /// Request timeout.
    pub timeout: Duration,
    /// Additional request headers.
    pub headers: Vec<(String, String)>,
    /// Additional request query params.
    pub query_params: Vec<(String, String)>,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            base_url: Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid"),
            organization: None,
            project: None,
            max_retries: DEFAULT_MAX_RETRIES,
            timeout: DEFAULT_TIMEOUT,
            headers: Vec::new(),
            query_params: Vec::new(),
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

        merged.api_key = value.api_key.clone().or_else(|| self.api_key.clone());
        merged.base_url = match value.base_url.as_deref() {
            Some(base_url) => Self::parse_base_url(base_url)?,
            None => self.base_url.clone(),
        };
        merged.organization = value
            .organization
            .clone()
            .or_else(|| self.organization.clone());
        merged.project = value.project.clone().or_else(|| self.project.clone());
        merged.max_retries = value.max_retries.unwrap_or(self.max_retries);
        merged.timeout = value.timeout.unwrap_or(self.timeout);
        merged.headers.extend(value.headers.iter().cloned());
        merged
            .query_params
            .extend(value.query_params.iter().cloned());

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
