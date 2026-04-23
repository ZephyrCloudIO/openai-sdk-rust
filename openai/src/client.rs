//! Core API client.

use std::{sync::Arc, time::Duration};

use futures::future::BoxFuture;
use reqwest::{
    header::{HeaderName, HeaderValue, CONTENT_TYPE},
    multipart::Form,
    Method, Request, RequestBuilder, Response, StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::{
    config::{
        CapturedResponse, HeaderAction, JsonAction, JsonOperation, QueryAction, RequestConfig,
        RequestMiddleware, RequestOptions,
    },
    error::{ApiErrorEnvelope, Error, Result},
    options::ClientConfig,
    pagination::{path_with_query, ConversationCursorPage, CursorPage, Page},
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
        let http = match config.http_client.clone() {
            Some(client) => client,
            None => reqwest::Client::builder()
                .timeout(config.timeout)
                .build()
                .map_err(Error::Http)?,
        };

        Ok(Self {
            inner: Arc::new(ClientInner { http, config }),
        })
    }

    /// Creates a client with only an API key.
    pub fn with_api_key(api_key: impl Into<String>) -> Result<Self> {
        Self::new(ClientConfig::default().with_api_key(api_key))
    }

    /// Returns a client clone with request options merged into its default config.
    ///
    /// This mirrors Go's ability to attach request options at service scope:
    /// `client.with_options(opts).responses().create(...)` applies `opts` to
    /// every request issued by the returned client.
    #[must_use]
    pub fn with_options(&self, options: RequestOptions) -> Self {
        let config = self.inner.config.merge_request_options(&options);
        let http = config
            .http_client
            .clone()
            .unwrap_or_else(|| self.inner.http.clone());

        Self {
            inner: Arc::new(ClientInner { http, config }),
        }
    }

    /// Alias for [`Client::with_options`].
    #[must_use]
    pub fn with_request_options(&self, options: RequestOptions) -> Self {
        self.with_options(options)
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

    /// Returns containers service.
    #[must_use]
    pub fn containers(&self) -> crate::containers::ContainerService {
        crate::containers::ContainerService::new(self.clone())
    }

    /// Returns conversations service.
    #[must_use]
    pub fn conversations(&self) -> crate::conversations::ConversationService {
        crate::conversations::ConversationService::new(self.clone())
    }

    /// Returns realtime service.
    #[must_use]
    pub fn realtime(&self) -> crate::realtime::RealtimeService {
        crate::realtime::RealtimeService::new(self.clone())
    }

    /// Returns responses service.
    #[must_use]
    pub fn responses(&self) -> crate::responses::ResponseService {
        crate::responses::ResponseService::new(self.clone())
    }

    /// Returns skills service.
    #[must_use]
    pub fn skills(&self) -> crate::skills::SkillService {
        crate::skills::SkillService::new(self.clone())
    }

    /// Returns videos service.
    #[must_use]
    pub fn videos(&self) -> crate::videos::VideoService {
        crate::videos::VideoService::new(self.clone())
    }

    /// Creates a webhook verification service with the given signing secret.
    #[must_use]
    pub fn webhooks(&self, secret: impl Into<String>) -> crate::webhooks::WebhookService {
        crate::webhooks::WebhookService::new(secret)
    }

    /// Creates a webhook verification service from the configured webhook secret.
    #[must_use]
    pub fn webhooks_from_config(&self) -> crate::webhooks::WebhookService {
        crate::webhooks::WebhookService::new(
            self.inner.config.webhook_secret.clone().unwrap_or_default(),
        )
    }

    /// Executes a raw JSON request and deserializes the response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or JSON failures.
    pub async fn execute<B, T>(&self, method: Method, path: &str, body: Option<&B>) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned + Serialize,
    {
        self.execute_with_options(method, path, body, RequestOptions::default())
            .await
    }

    /// Executes a raw JSON request with request-local options.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or JSON failures.
    pub async fn execute_with_options<B, T>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        options: RequestOptions,
    ) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned + Serialize,
    {
        let config = self.inner.config.merge_request_options(&options);
        let response = self
            .execute_raw_with_options(method, path, body, options)
            .await?;
        Self::parse_json_response_with_config(response, Some(&config)).await
    }

    /// Executes a GET request and deserializes the JSON response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or JSON failures.
    pub async fn get<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let response = self
            .send_with_retry(|| self.request(Method::GET, path, &RequestOptions::default()))
            .await?;
        Self::parse_json_response(response).await
    }

    /// Executes a GET request with request-local options and deserializes the JSON response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or JSON failures.
    pub async fn get_with_options<T>(&self, path: &str, options: RequestOptions) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        self.get_json_with_options(path, &options).await
    }

    /// Executes a POST request and deserializes the JSON response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or JSON failures.
    pub async fn post<B, T>(&self, path: &str, body: &B) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned + Serialize,
    {
        self.execute(Method::POST, path, Some(body)).await
    }

    /// Executes a POST request with request-local options and deserializes the JSON response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or JSON failures.
    pub async fn post_with_options<B, T>(
        &self,
        path: &str,
        body: &B,
        options: RequestOptions,
    ) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned + Serialize,
    {
        self.execute_with_options(Method::POST, path, Some(body), options)
            .await
    }

    /// Executes a PUT request and deserializes the JSON response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or JSON failures.
    pub async fn put<B, T>(&self, path: &str, body: &B) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned + Serialize,
    {
        self.execute(Method::PUT, path, Some(body)).await
    }

    /// Executes a PUT request with request-local options and deserializes the JSON response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or JSON failures.
    pub async fn put_with_options<B, T>(
        &self,
        path: &str,
        body: &B,
        options: RequestOptions,
    ) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned + Serialize,
    {
        self.execute_with_options(Method::PUT, path, Some(body), options)
            .await
    }

    /// Executes a PATCH request and deserializes the JSON response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or JSON failures.
    pub async fn patch<B, T>(&self, path: &str, body: &B) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned + Serialize,
    {
        self.execute(Method::PATCH, path, Some(body)).await
    }

    /// Executes a PATCH request with request-local options and deserializes the JSON response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or JSON failures.
    pub async fn patch_with_options<B, T>(
        &self,
        path: &str,
        body: &B,
        options: RequestOptions,
    ) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned + Serialize,
    {
        self.execute_with_options(Method::PATCH, path, Some(body), options)
            .await
    }

    /// Executes a DELETE request and deserializes the JSON response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or JSON failures.
    pub async fn delete<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let response = self
            .send_with_retry(|| self.request(Method::DELETE, path, &RequestOptions::default()))
            .await?;
        Self::parse_json_response(response).await
    }

    /// Executes a DELETE request with request-local options and deserializes the JSON response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport, API, or JSON failures.
    pub async fn delete_with_options<T>(&self, path: &str, options: RequestOptions) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let config = self.inner.config.merge_request_options(&options);
        let response = self
            .send_with_retry_options(&options, || self.request(Method::DELETE, path, &options))
            .await?;
        Self::parse_json_response_with_config(response, Some(&config)).await
    }

    /// Executes a raw request and returns the HTTP response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures during send.
    pub async fn execute_raw<B>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<Response>
    where
        B: Serialize + ?Sized,
    {
        self.execute_raw_with_options(method, path, body, RequestOptions::default())
            .await
    }

    /// Executes a raw request with request-local options and returns the HTTP response.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures during send.
    pub async fn execute_raw_with_options<B>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        options: RequestOptions,
    ) -> Result<Response>
    where
        B: Serialize + ?Sized,
    {
        self.send_with_retry_options(&options, || {
            let request = self.request(method.clone(), path, &options)?;
            Ok(match body {
                Some(body) => request.json(body),
                None => request,
            })
        })
        .await
    }

    pub(crate) async fn get_json<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let response = self
            .send_with_retry(|| self.request(Method::GET, path, &RequestOptions::default()))
            .await?;
        Self::parse_json_response(response).await
    }

    pub(crate) async fn get_json_with_options<T>(
        &self,
        path: &str,
        options: &RequestOptions,
    ) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let config = self.inner.config.merge_request_options(options);
        let response = self
            .send_with_retry_options(options, || self.request(Method::GET, path, options))
            .await?;
        Self::parse_json_response_with_config(response, Some(&config)).await
    }

    pub(crate) async fn get_json_query<Q, T>(&self, path: &str, query: &Q) -> Result<T>
    where
        Q: Serialize + ?Sized,
        T: DeserializeOwned + Serialize,
    {
        let response = self
            .send_with_retry(|| {
                Ok(self
                    .request(Method::GET, path, &RequestOptions::default())?
                    .query(query))
            })
            .await?;
        Self::parse_json_response(response).await
    }

    pub(crate) async fn get_cursor_page<T>(&self, path: &str) -> Result<CursorPage<T>>
    where
        T: Clone + DeserializeOwned + Serialize + Send + 'static,
    {
        let page: CursorPage<T> = self.get_json(path).await?;
        Ok(page.with_context(self.clone(), path.to_owned()))
    }

    pub(crate) async fn get_cursor_page_options<T>(
        &self,
        path: &str,
        options: RequestOptions,
    ) -> Result<CursorPage<T>>
    where
        T: Clone + DeserializeOwned + Serialize + Send + 'static,
    {
        let page: CursorPage<T> = self.get_json_with_options(path, &options).await?;
        Ok(page.with_request_options(self.clone(), path.to_owned(), options))
    }

    pub(crate) async fn get_cursor_page_query<Q, T>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<CursorPage<T>>
    where
        Q: Serialize + ?Sized,
        T: Clone + DeserializeOwned + Serialize + Send + 'static,
    {
        let request_path = path_with_query(path, query)?;
        let page: CursorPage<T> = self.get_json_query(path, query).await?;
        Ok(page.with_context(self.clone(), request_path))
    }

    pub(crate) async fn get_cursor_page_query_options<Q, T>(
        &self,
        path: &str,
        query: &Q,
        options: RequestOptions,
    ) -> Result<CursorPage<T>>
    where
        Q: Serialize + ?Sized,
        T: Clone + DeserializeOwned + Serialize + Send + 'static,
    {
        let request_path = path_with_query(path, query)?;
        let config = self.inner.config.merge_request_options(&options);
        let response = self
            .send_with_retry_options(&options, || {
                Ok(self.request(Method::GET, path, &options)?.query(query))
            })
            .await?;
        let page: CursorPage<T> =
            Self::parse_json_response_with_config(response, Some(&config)).await?;
        Ok(page.with_request_options(self.clone(), request_path, options))
    }

    #[allow(dead_code)]
    pub(crate) async fn get_conversation_cursor_page<T>(
        &self,
        path: &str,
    ) -> Result<ConversationCursorPage<T>>
    where
        T: Clone + DeserializeOwned + Serialize + Send + 'static,
    {
        let page: ConversationCursorPage<T> = self.get_json(path).await?;
        Ok(page.with_context(self.clone(), path.to_owned()))
    }

    pub(crate) async fn get_conversation_cursor_page_options<T>(
        &self,
        path: &str,
        options: RequestOptions,
    ) -> Result<ConversationCursorPage<T>>
    where
        T: Clone + DeserializeOwned + Serialize + Send + 'static,
    {
        let page: ConversationCursorPage<T> = self.get_json_with_options(path, &options).await?;
        Ok(page.with_request_options(self.clone(), path.to_owned(), options))
    }

    pub(crate) async fn get_conversation_cursor_page_query<Q, T>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<ConversationCursorPage<T>>
    where
        Q: Serialize + ?Sized,
        T: Clone + DeserializeOwned + Serialize + Send + 'static,
    {
        let request_path = path_with_query(path, query)?;
        let page: ConversationCursorPage<T> = self.get_json_query(path, query).await?;
        Ok(page.with_context(self.clone(), request_path))
    }

    pub(crate) async fn get_conversation_cursor_page_query_options<Q, T>(
        &self,
        path: &str,
        query: &Q,
        options: RequestOptions,
    ) -> Result<ConversationCursorPage<T>>
    where
        Q: Serialize + ?Sized,
        T: Clone + DeserializeOwned + Serialize + Send + 'static,
    {
        let request_path = path_with_query(path, query)?;
        let config = self.inner.config.merge_request_options(&options);
        let response = self
            .send_with_retry_options(&options, || {
                Ok(self.request(Method::GET, path, &options)?.query(query))
            })
            .await?;
        let page: ConversationCursorPage<T> =
            Self::parse_json_response_with_config(response, Some(&config)).await?;
        Ok(page.with_request_options(self.clone(), request_path, options))
    }

    pub(crate) async fn get_page<T>(&self, path: &str) -> Result<Page<T>>
    where
        T: Clone + DeserializeOwned + Serialize + Send + 'static,
    {
        let page: Page<T> = self.get_json(path).await?;
        Ok(page.with_client(self.clone()))
    }

    #[allow(dead_code)]
    pub(crate) async fn get_page_query<Q, T>(&self, path: &str, query: &Q) -> Result<Page<T>>
    where
        Q: Serialize + ?Sized,
        T: Clone + DeserializeOwned + Serialize + Send + 'static,
    {
        let page: Page<T> = self.get_json_query(path, query).await?;
        Ok(page.with_client(self.clone()))
    }

    pub(crate) async fn post_json<B, T>(&self, path: &str, body: &B) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned + Serialize,
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
        T: DeserializeOwned + Serialize,
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
        T: DeserializeOwned + Serialize,
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

    pub(crate) async fn post_raw_multipart<F>(&self, path: &str, make_form: F) -> Result<Response>
    where
        F: Fn() -> Form,
    {
        self.send_with_retry(|| {
            Ok(self
                .request(Method::POST, path, &RequestOptions::default())?
                .multipart(make_form()))
        })
        .await
    }

    // ------------------------------------------------------------------
    // Beta helpers — inject `OpenAI-Beta: assistants=v2` on every call.
    // ------------------------------------------------------------------

    pub(crate) async fn get_json_beta<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let response = self
            .send_with_retry(|| {
                Ok(self
                    .request(Method::GET, path, &RequestOptions::default())?
                    .header("OpenAI-Beta", "assistants=v2"))
            })
            .await?;
        Self::parse_json_response(response).await
    }

    #[allow(dead_code)]
    pub(crate) async fn get_json_beta_query<Q, T>(&self, path: &str, query: &Q) -> Result<T>
    where
        Q: Serialize + ?Sized,
        T: DeserializeOwned + Serialize,
    {
        let response = self
            .send_with_retry(|| {
                Ok(self
                    .request(Method::GET, path, &RequestOptions::default())?
                    .header("OpenAI-Beta", "assistants=v2")
                    .query(query))
            })
            .await?;
        Self::parse_json_response(response).await
    }

    pub(crate) async fn post_json_beta<B, T>(&self, path: &str, body: &B) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned + Serialize,
    {
        let response = self
            .send_with_retry(|| {
                Ok(self
                    .request(Method::POST, path, &RequestOptions::default())?
                    .header("OpenAI-Beta", "assistants=v2")
                    .json(body))
            })
            .await?;
        Self::parse_json_response(response).await
    }

    pub(crate) async fn delete_json_beta<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let response = self
            .send_with_retry(|| {
                Ok(self
                    .request(Method::DELETE, path, &RequestOptions::default())?
                    .header("OpenAI-Beta", "assistants=v2"))
            })
            .await?;
        Self::parse_json_response(response).await
    }

    pub(crate) async fn post_raw_json_beta<B>(&self, path: &str, body: &B) -> Result<Response>
    where
        B: Serialize + ?Sized,
    {
        self.send_with_retry(|| {
            Ok(self
                .request(Method::POST, path, &RequestOptions::default())?
                .header("OpenAI-Beta", "assistants=v2")
                .json(body))
        })
        .await
    }

    // ------------------------------------------------------------------
    // ChatKit helpers — inject `OpenAI-Beta: chatkit_beta=v1`.
    // ------------------------------------------------------------------

    pub(crate) async fn get_json_chatkit_beta<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let response = self
            .send_with_retry(|| {
                Ok(self
                    .request(Method::GET, path, &RequestOptions::default())?
                    .header("OpenAI-Beta", "chatkit_beta=v1"))
            })
            .await?;
        Self::parse_json_response(response).await
    }

    #[allow(dead_code)]
    pub(crate) async fn get_json_chatkit_beta_query<Q, T>(&self, path: &str, query: &Q) -> Result<T>
    where
        Q: Serialize + ?Sized,
        T: DeserializeOwned + Serialize,
    {
        let response = self
            .send_with_retry(|| {
                Ok(self
                    .request(Method::GET, path, &RequestOptions::default())?
                    .header("OpenAI-Beta", "chatkit_beta=v1")
                    .query(query))
            })
            .await?;
        Self::parse_json_response(response).await
    }

    pub(crate) async fn post_json_chatkit_beta<B, T>(&self, path: &str, body: &B) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned + Serialize,
    {
        let response = self
            .send_with_retry(|| {
                Ok(self
                    .request(Method::POST, path, &RequestOptions::default())?
                    .header("OpenAI-Beta", "chatkit_beta=v1")
                    .json(body))
            })
            .await?;
        Self::parse_json_response(response).await
    }

    pub(crate) async fn delete_json_chatkit_beta<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let response = self
            .send_with_retry(|| {
                Ok(self
                    .request(Method::DELETE, path, &RequestOptions::default())?
                    .header("OpenAI-Beta", "chatkit_beta=v1"))
            })
            .await?;
        Self::parse_json_response(response).await
    }

    // ------------------------------------------------------------------
    // Other helpers
    // ------------------------------------------------------------------

    /// Returns a reference to the resolved request configuration.
    #[allow(dead_code)]
    pub(crate) fn config(&self) -> &RequestConfig {
        &self.inner.config
    }

    pub(crate) async fn post_empty_json<B>(&self, path: &str, body: &B) -> Result<()>
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
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(crate::error::Error::Api {
                status: status.as_u16(),
                code: None,
                message: body,
                param: None,
                error_type: None,
            })
        }
    }

    pub(crate) async fn get_raw_query<Q>(&self, path: &str, query: &Q) -> Result<Response>
    where
        Q: Serialize + ?Sized,
    {
        self.send_with_retry(|| {
            Ok(self
                .request(Method::GET, path, &RequestOptions::default())?
                .query(query))
        })
        .await
    }

    #[allow(dead_code)]
    pub(crate) async fn post_empty(&self, path: &str) -> Result<()> {
        let response = self
            .send_with_retry(|| {
                Ok(self
                    .request(Method::POST, path, &RequestOptions::default())?
                    .header("Content-Length", "0"))
            })
            .await?;
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(crate::error::Error::Api {
                status: status.as_u16(),
                code: None,
                message: body,
                param: None,
                error_type: None,
            })
        }
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
        let mut url = config.endpoint_url(path)?;
        Self::apply_query_params(&mut url, &config.query_params);
        let http = config.http_client.as_ref().unwrap_or(&self.inner.http);
        let mut request = http.request(method, url).timeout(config.timeout);

        request = request
            .header(
                "User-Agent",
                format!("OpenAI/Rust {}", env!("CARGO_PKG_VERSION")),
            )
            .header("X-Stainless-Lang", "rust")
            .header("X-Stainless-Package-Version", env!("CARGO_PKG_VERSION"))
            .header("X-Stainless-OS", std::env::consts::OS)
            .header("X-Stainless-Arch", std::env::consts::ARCH)
            .header("X-Stainless-Runtime", "rust")
            .header(
                "X-Stainless-Runtime-Version",
                option_env!("RUSTC_VERSION").unwrap_or("unknown"),
            )
            .header("X-Stainless-Timeout", config.timeout.as_secs().to_string());

        if config.workload_identity.is_none() {
            if let Some(api_key) = config.api_key.as_deref() {
                request = request.bearer_auth(api_key);
            }
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
        Ok(request)
    }

    async fn apply_auth(
        &self,
        request: RequestBuilder,
        config: &RequestConfig,
    ) -> Result<RequestBuilder> {
        if let Some(workload_identity) = &config.workload_identity {
            let http = config.http_client.as_ref().unwrap_or(&self.inner.http);
            let token = workload_identity.get_token(http).await?;
            Ok(request.bearer_auth(token))
        } else {
            Ok(request)
        }
    }

    async fn send_with_retry<F>(&self, make_request: F) -> Result<Response>
    where
        F: Fn() -> Result<RequestBuilder>,
    {
        self.send_with_retry_options(&RequestOptions::default(), make_request)
            .await
    }

    async fn send_with_retry_options<F>(
        &self,
        options: &RequestOptions,
        make_request: F,
    ) -> Result<Response>
    where
        F: Fn() -> Result<RequestBuilder>,
    {
        let config = self.inner.config.merge_request_options(options);
        let max_retries = config.max_retries;
        let http = config
            .http_client
            .clone()
            .unwrap_or_else(|| self.inner.http.clone());
        let mut attempt = 0_u32;
        let mut workload_identity_auth_retry_done = false;

        loop {
            let request = self
                .apply_auth(
                    make_request()?.header("X-Stainless-Retry-Count", attempt.to_string()),
                    &config,
                )
                .await?;
            let mut request = request.build().map_err(Error::Http)?;
            Self::finalize_request(&mut request, &config)?;

            match Self::execute_request(http.clone(), Arc::new(config.middlewares.clone()), request)
                .await
            {
                Ok(response) => {
                    if response.status() == StatusCode::UNAUTHORIZED
                        && config.workload_identity.is_some()
                        && !workload_identity_auth_retry_done
                    {
                        if let Some(workload_identity) = &config.workload_identity {
                            workload_identity.invalidate_token().await;
                        }
                        workload_identity_auth_retry_done = true;
                        continue;
                    }

                    if !Self::is_retryable_status(response.status()) || attempt >= max_retries {
                        Self::capture_response_metadata(&response, &config);
                        return Ok(response);
                    }
                }
                Err(err) => {
                    if attempt >= max_retries {
                        return Err(err);
                    }
                }
            }

            attempt = attempt.saturating_add(1);
            tokio::time::sleep(Self::backoff_delay(attempt)).await;
        }
    }

    async fn execute_request(
        http: reqwest::Client,
        middlewares: Arc<Vec<RequestMiddleware>>,
        request: Request,
    ) -> Result<Response> {
        Self::call_middleware(http, middlewares, 0, request).await
    }

    fn call_middleware(
        http: reqwest::Client,
        middlewares: Arc<Vec<RequestMiddleware>>,
        index: usize,
        request: Request,
    ) -> BoxFuture<'static, Result<Response>> {
        if index >= middlewares.len() {
            return Box::pin(async move { http.execute(request).await.map_err(Error::Http) });
        }

        let middleware = middlewares[index].clone();
        let next_http = http.clone();
        let next_middlewares = middlewares.clone();
        let next = Arc::new(move |request| {
            Self::call_middleware(
                next_http.clone(),
                next_middlewares.clone(),
                index + 1,
                request,
            )
        });

        middleware(request, next)
    }

    fn finalize_request(request: &mut Request, config: &RequestConfig) -> Result<()> {
        Self::apply_query_ops(request.url_mut(), &config.query_ops);
        Self::apply_request_body_override(request, config)?;
        Self::apply_json_ops(request, &config.json_ops)?;
        Self::apply_header_ops(request, config)?;
        Ok(())
    }

    fn apply_query_params(url: &mut url::Url, params: &[(String, String)]) {
        if params.is_empty() {
            return;
        }

        let mut query = url
            .query_pairs()
            .into_owned()
            .collect::<Vec<(String, String)>>();
        query.extend(params.iter().cloned());
        Self::replace_query(url, &query);
    }

    fn apply_query_ops(url: &mut url::Url, ops: &[crate::config::QueryOperation]) {
        if ops.is_empty() {
            return;
        }

        let mut query = url
            .query_pairs()
            .into_owned()
            .collect::<Vec<(String, String)>>();

        for op in ops {
            match op.action {
                QueryAction::Set => {
                    query.retain(|(key, _)| key != &op.key);
                    if let Some(value) = &op.value {
                        query.push((op.key.clone(), value.clone()));
                    }
                }
                QueryAction::Add => {
                    if let Some(value) = &op.value {
                        query.push((op.key.clone(), value.clone()));
                    }
                }
                QueryAction::Delete => query.retain(|(key, _)| key != &op.key),
            }
        }

        Self::replace_query(url, &query);
    }

    fn replace_query(url: &mut url::Url, query: &[(String, String)]) {
        url.set_query(None);
        if query.is_empty() {
            return;
        }

        let mut pairs = url.query_pairs_mut();
        for (key, value) in query {
            pairs.append_pair(key, value);
        }
    }

    fn apply_request_body_override(request: &mut Request, config: &RequestConfig) -> Result<()> {
        let Some(body) = &config.request_body else {
            return Ok(());
        };

        *request.body_mut() = Some(body.body.clone().into());
        let value = HeaderValue::from_str(&body.content_type).map_err(|err| Error::Config {
            field: "request_body.content_type",
            message: err.to_string(),
        })?;
        request.headers_mut().insert(CONTENT_TYPE, value);
        Ok(())
    }

    fn apply_json_ops(request: &mut Request, ops: &[JsonOperation]) -> Result<()> {
        if ops.is_empty() {
            return Ok(());
        }

        let bytes = request
            .body()
            .and_then(reqwest::Body::as_bytes)
            .ok_or_else(|| Error::Config {
                field: "json_body",
                message: "cannot mutate a streaming or multipart request body".to_owned(),
            })?;
        let mut value = if bytes.is_empty() {
            Value::Object(serde_json::Map::new())
        } else {
            serde_json::from_slice(bytes).map_err(Error::Json)?
        };

        for op in ops {
            match op.action {
                JsonAction::Set => {
                    let Some(value_to_set) = op.value.clone() else {
                        continue;
                    };
                    Self::json_set(&mut value, &op.path, value_to_set);
                }
                JsonAction::Delete => Self::json_del(&mut value, &op.path),
            }
        }

        *request.body_mut() = Some(serde_json::to_vec(&value).map_err(Error::Json)?.into());
        request
            .headers_mut()
            .entry(CONTENT_TYPE)
            .or_insert(HeaderValue::from_static("application/json"));
        Ok(())
    }

    fn json_set(root: &mut Value, path: &str, value: Value) {
        let parts = path
            .split('.')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if parts.is_empty() {
            *root = value;
            return;
        }

        let mut cursor = root;
        for part in &parts[..parts.len() - 1] {
            cursor = ensure_json_child(cursor, part);
        }

        set_json_child(cursor, parts[parts.len() - 1], value);
    }

    fn json_del(root: &mut Value, path: &str) {
        let parts = path
            .split('.')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if parts.is_empty() {
            *root = Value::Null;
            return;
        }

        let mut cursor = root;
        for part in &parts[..parts.len() - 1] {
            let Some(next) = get_json_child_mut(cursor, part) else {
                return;
            };
            cursor = next;
        }

        delete_json_child(cursor, parts[parts.len() - 1]);
    }

    fn apply_header_ops(request: &mut Request, config: &RequestConfig) -> Result<()> {
        for op in &config.header_ops {
            let name = HeaderName::from_bytes(op.key.as_bytes()).map_err(|err| Error::Config {
                field: "headers",
                message: err.to_string(),
            })?;
            match op.action {
                HeaderAction::Set => {
                    let Some(value) = &op.value else {
                        continue;
                    };
                    let value = HeaderValue::from_str(value).map_err(|err| Error::Config {
                        field: "headers",
                        message: err.to_string(),
                    })?;
                    request.headers_mut().insert(name, value);
                }
                HeaderAction::Add => {
                    let Some(value) = &op.value else {
                        continue;
                    };
                    let value = HeaderValue::from_str(value).map_err(|err| Error::Config {
                        field: "headers",
                        message: err.to_string(),
                    })?;
                    request.headers_mut().append(name, value);
                }
                HeaderAction::Delete => {
                    request.headers_mut().remove(name);
                }
            }
        }

        Ok(())
    }

    fn capture_response_metadata(response: &Response, config: &RequestConfig) {
        let Some(capture) = &config.response_capture else {
            return;
        };

        let headers = response
            .headers()
            .iter()
            .map(|(key, value)| {
                (
                    key.as_str().to_owned(),
                    value.to_str().unwrap_or_default().to_owned(),
                )
            })
            .collect::<Vec<_>>();

        if let Ok(mut captured) = capture.lock() {
            *captured = Some(CapturedResponse {
                status: response.status().as_u16(),
                url: response.url().to_string(),
                headers,
            });
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
        T: DeserializeOwned + Serialize,
    {
        Self::parse_json_response_with_config(response, None).await
    }

    async fn parse_json_response_with_config<T>(
        response: Response,
        config: Option<&RequestConfig>,
    ) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let status = response.status();
        let body = response.text().await.map_err(Error::Http)?;
        if let Some(capture) = config.and_then(|config| config.response_body_capture.as_ref()) {
            if let Ok(mut captured) = capture.lock() {
                *captured = Some(body.as_bytes().to_vec());
            }
        }

        if status.is_success() {
            let parsed = serde_json::from_str::<T>(&body).map_err(Error::Json)?;
            crate::raw_json::register_raw_json(&parsed, &body);
            return Ok(parsed);
        }

        Err(Self::parse_api_error(status, body))
    }

    async fn parse_bytes_response(response: Response) -> Result<Vec<u8>> {
        Self::parse_bytes_response_with_config(response, None).await
    }

    async fn parse_bytes_response_with_config(
        response: Response,
        config: Option<&RequestConfig>,
    ) -> Result<Vec<u8>> {
        let status = response.status();
        let body = response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(Error::Http)?;
        if let Some(capture) = config.and_then(|config| config.response_body_capture.as_ref()) {
            if let Ok(mut captured) = capture.lock() {
                *captured = Some(body.clone());
            }
        }

        if status.is_success() {
            return Ok(body);
        }

        let body = String::from_utf8_lossy(&body).into_owned();
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

fn ensure_json_child<'a>(value: &'a mut Value, part: &str) -> &'a mut Value {
    if let Ok(index) = part.parse::<usize>() {
        if !value.is_array() {
            *value = Value::Array(Vec::new());
        }
        let Value::Array(items) = value else {
            unreachable!("value was converted to an array");
        };
        while items.len() <= index {
            items.push(Value::Null);
        }
        return &mut items[index];
    }

    if !value.is_object() {
        *value = Value::Object(serde_json::Map::new());
    }
    let Value::Object(object) = value else {
        unreachable!("value was converted to an object");
    };
    object.entry(part.to_owned()).or_insert(Value::Null)
}

fn set_json_child(value: &mut Value, part: &str, child: Value) {
    if let Ok(index) = part.parse::<usize>() {
        if !value.is_array() {
            *value = Value::Array(Vec::new());
        }
        let Value::Array(items) = value else {
            return;
        };
        while items.len() <= index {
            items.push(Value::Null);
        }
        items[index] = child;
        return;
    }

    if !value.is_object() {
        *value = Value::Object(serde_json::Map::new());
    }
    if let Value::Object(object) = value {
        object.insert(part.to_owned(), child);
    }
}

fn get_json_child_mut<'a>(value: &'a mut Value, part: &str) -> Option<&'a mut Value> {
    if let Ok(index) = part.parse::<usize>() {
        return value.as_array_mut()?.get_mut(index);
    }

    value.as_object_mut()?.get_mut(part)
}

fn delete_json_child(value: &mut Value, part: &str) {
    if let Ok(index) = part.parse::<usize>() {
        if let Some(items) = value.as_array_mut() {
            if index < items.len() {
                items.remove(index);
            }
        }
        return;
    }

    if let Some(object) = value.as_object_mut() {
        object.remove(part);
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
