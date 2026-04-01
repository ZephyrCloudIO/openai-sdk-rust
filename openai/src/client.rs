//! Core API client.

use std::{sync::Arc, time::Duration};

use reqwest::{multipart::Form, Method, RequestBuilder, Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    config::{RequestConfig, RequestOptions},
    error::{ApiErrorEnvelope, Error, Result},
    options::ClientConfig,
};

/// Primary OpenAI API client.
#[derive(Clone, Debug)]
pub struct Client {
    inner: Arc<ClientInner>,
}

#[derive(Clone, Debug)]
struct ClientInner {
    http: reqwest::Client,
    config: RequestConfig,
}

impl Client {
    /// Creates a client from explicit config.
    pub fn new(config: ClientConfig) -> Result<Self> {
        let config = RequestConfig::from_client_config(config)?;
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(Error::Http)?;

        Ok(Self {
            inner: Arc::new(ClientInner { http, config }),
        })
    }

    /// Creates a client with only an API key.
    pub fn with_api_key(api_key: impl Into<String>) -> Result<Self> {
        Self::new(ClientConfig::default().with_api_key(api_key))
    }

    /// Returns chat service entrypoint.
    #[must_use]
    pub fn chat(&self) -> crate::chat::ChatService {
        crate::chat::ChatService::new(self.clone())
    }

    /// Returns audio service.
    #[must_use]
    pub fn audio(&self) -> crate::audio::AudioService {
        crate::audio::AudioService::new(self.clone())
    }

    /// Returns batches service.
    #[must_use]
    pub fn batches(&self) -> crate::batches::BatchService {
        crate::batches::BatchService::new(self.clone())
    }

    /// Returns beta Assistants API namespace.
    #[must_use]
    pub fn beta(&self) -> crate::beta::BetaService {
        crate::beta::BetaService::new(self.clone())
    }

    /// Returns legacy completions service.
    #[must_use]
    pub fn completions(&self) -> crate::completions::CompletionService {
        crate::completions::CompletionService::new(self.clone())
    }

    /// Returns embeddings service.
    #[must_use]
    pub fn embeddings(&self) -> crate::embeddings::EmbeddingService {
        crate::embeddings::EmbeddingService::new(self.clone())
    }

    /// Returns files service.
    #[must_use]
    pub fn files(&self) -> crate::files::FileService {
        crate::files::FileService::new(self.clone())
    }

    /// Returns fine-tuning service.
    #[must_use]
    pub fn fine_tuning(&self) -> crate::fine_tuning::FineTuningService {
        crate::fine_tuning::FineTuningService::new(self.clone())
    }

    /// Returns graders service namespace.
    #[must_use]
    pub fn graders(&self) -> crate::graders::GraderService {
        crate::graders::GraderService::new(self.clone())
    }

    /// Returns images service.
    #[must_use]
    pub fn images(&self) -> crate::images::ImageService {
        crate::images::ImageService::new(self.clone())
    }

    /// Returns models service.
    #[must_use]
    pub fn models(&self) -> crate::models::ModelService {
        crate::models::ModelService::new(self.clone())
    }

    /// Returns moderations service.
    #[must_use]
    pub fn moderations(&self) -> crate::moderations::ModerationService {
        crate::moderations::ModerationService::new(self.clone())
    }

    /// Returns uploads service.
    #[must_use]
    pub fn uploads(&self) -> crate::uploads::UploadService {
        crate::uploads::UploadService::new(self.clone())
    }

    /// Returns vector stores service.
    #[must_use]
    pub fn vector_stores(&self) -> crate::vector_stores::VectorStoreService {
        crate::vector_stores::VectorStoreService::new(self.clone())
    }

    pub(crate) async fn get_json<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .send_with_retry(|| self.request(Method::GET, path, &RequestOptions::default()))
            .await?;
        Self::parse_json_response(response).await
    }

    pub(crate) async fn post_json<B, T>(&self, path: &str, body: &B) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let response = self
            .send_with_retry(|| {
                Ok(self
                    .request(Method::POST, path, &RequestOptions::default())?
                    .json(body))
            })
            .await?;
        Self::parse_json_response(response).await
    }

    pub(crate) async fn delete_json<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .send_with_retry(|| self.request(Method::DELETE, path, &RequestOptions::default()))
            .await?;
        Self::parse_json_response(response).await
    }

    pub(crate) async fn post_raw_json<B>(&self, path: &str, body: &B) -> Result<Response>
    where
        B: Serialize + ?Sized,
    {
        self.send_with_retry(|| {
            Ok(self
                .request(Method::POST, path, &RequestOptions::default())?
                .json(body))
        })
        .await
    }

    pub(crate) async fn post_json_bytes<B>(&self, path: &str, body: &B) -> Result<Vec<u8>>
    where
        B: Serialize + ?Sized,
    {
        let response = self
            .send_with_retry(|| {
                Ok(self
                    .request(Method::POST, path, &RequestOptions::default())?
                    .json(body))
            })
            .await?;
        Self::parse_bytes_response(response).await
    }

    pub(crate) async fn post_multipart_json<F, T>(&self, path: &str, make_form: F) -> Result<T>
    where
        F: Fn() -> Form,
        T: DeserializeOwned,
    {
        let response = self
            .send_with_retry(|| {
                Ok(self
                    .request(Method::POST, path, &RequestOptions::default())?
                    .multipart(make_form()))
            })
            .await?;
        Self::parse_json_response(response).await
    }

    pub(crate) async fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        let response = self
            .send_with_retry(|| self.request(Method::GET, path, &RequestOptions::default()))
            .await?;
        Self::parse_bytes_response(response).await
    }

    fn request(
        &self,
        method: Method,
        path: &str,
        options: &RequestOptions,
    ) -> Result<RequestBuilder> {
        let config = self.inner.config.merge_request_options(options);
        let url = config.endpoint_url(path)?;
        let mut request = self.inner.http.request(method, url).timeout(config.timeout);

        if let Some(api_key) = config.api_key.as_deref() {
            request = request.bearer_auth(api_key);
        }
        if let Some(org) = config.organization.as_deref() {
            request = request.header("OpenAI-Organization", org);
        }
        if let Some(project) = config.project.as_deref() {
            request = request.header("OpenAI-Project", project);
        }
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }
        if !config.query_params.is_empty() {
            request = request.query(&config.query_params);
        }

        Ok(request)
    }

    async fn send_with_retry<F>(&self, make_request: F) -> Result<Response>
    where
        F: Fn() -> Result<RequestBuilder>,
    {
        let max_retries = self.inner.config.max_retries;
        let mut attempt = 0_u32;

        loop {
            let request = make_request()?;
            match request.send().await {
                Ok(response) => {
                    if !Self::is_retryable_status(response.status()) || attempt >= max_retries {
                        return Ok(response);
                    }
                }
                Err(err) => {
                    if attempt >= max_retries {
                        return Err(Error::Http(err));
                    }
                }
            }

            attempt = attempt.saturating_add(1);
            tokio::time::sleep(Self::backoff_delay(attempt)).await;
        }
    }

    fn backoff_delay(attempt: u32) -> Duration {
        let exp = 2_u64.saturating_pow(attempt.saturating_sub(1));
        let millis = 100_u64.saturating_mul(exp).min(2_000);
        Duration::from_millis(millis)
    }

    fn is_retryable_status(status: StatusCode) -> bool {
        matches!(status.as_u16(), 408 | 409 | 429) || status.is_server_error()
    }

    async fn parse_json_response<T>(response: Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let status = response.status();

        if status.is_success() {
            return response.json::<T>().await.map_err(Error::Http);
        }

        let body = response.text().await.map_err(Error::Http)?;
        Err(Self::parse_api_error(status, body))
    }

    async fn parse_bytes_response(response: Response) -> Result<Vec<u8>> {
        let status = response.status();
        if status.is_success() {
            return response
                .bytes()
                .await
                .map(|bytes| bytes.to_vec())
                .map_err(Error::Http);
        }

        let body = response.text().await.map_err(Error::Http)?;
        Err(Self::parse_api_error(status, body))
    }

    fn parse_api_error(status: StatusCode, body: String) -> Error {
        if let Ok(parsed) = serde_json::from_str::<ApiErrorEnvelope>(&body) {
            return Error::Api {
                status: status.as_u16(),
                code: parsed.error.code,
                message: parsed.error.message,
                param: parsed.error.param,
                error_type: parsed.error.error_type,
            };
        }

        Error::Api {
            status: status.as_u16(),
            code: None,
            message: body,
            param: None,
            error_type: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Client;
    use crate::options::ClientConfig;
    use reqwest::StatusCode;
    use std::time::Duration;

    #[test]
    fn backoff_increases_and_caps() {
        assert_eq!(Client::backoff_delay(1), Duration::from_millis(100));
        assert_eq!(Client::backoff_delay(2), Duration::from_millis(200));
        assert_eq!(Client::backoff_delay(3), Duration::from_millis(400));
        assert_eq!(Client::backoff_delay(10), Duration::from_millis(2_000));
    }

    #[test]
    fn client_constructs_with_defaults() {
        let cfg = ClientConfig::default().with_api_key("test");
        let client = Client::new(cfg);
        assert!(client.is_ok());
    }

    #[test]
    fn retryable_statuses_match_expectations() {
        assert!(Client::is_retryable_status(StatusCode::REQUEST_TIMEOUT));
        assert!(Client::is_retryable_status(StatusCode::CONFLICT));
        assert!(Client::is_retryable_status(StatusCode::TOO_MANY_REQUESTS));
        assert!(Client::is_retryable_status(
            StatusCode::INTERNAL_SERVER_ERROR
        ));
        assert!(!Client::is_retryable_status(StatusCode::BAD_REQUEST));
        assert!(!Client::is_retryable_status(StatusCode::UNAUTHORIZED));
    }
}
