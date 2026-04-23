//! Vector stores APIs.

use std::collections::HashMap;

use crate::{
    pagination::{CursorPage, Page},
    Client, Result,
};

/// Vector store service.
#[derive(Clone)]
pub struct VectorStoreService {
    client: Client,
}

impl VectorStoreService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns a file sub-service scoped to the given vector store.
    #[must_use]
    pub fn files(&self, vector_store_id: impl Into<String>) -> VectorStoreFileService {
        VectorStoreFileService::new(self.client.clone(), vector_store_id.into())
    }

    /// Returns a file-batch sub-service scoped to the given vector store.
    #[must_use]
    pub fn file_batches(&self, vector_store_id: impl Into<String>) -> VectorStoreFileBatchService {
        VectorStoreFileBatchService::new(self.client.clone(), vector_store_id.into())
    }

    /// Creates a vector store.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: VectorStoreCreateParams) -> Result<VectorStore> {
        self.client.post_json("/vector_stores", &params).await
    }

    /// Gets a vector store by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, vector_store_id: impl AsRef<str>) -> Result<VectorStore> {
        self.client
            .get_json(&format!(
                "/vector_stores/{}",
                urlencoding::encode(vector_store_id.as_ref())
            ))
            .await
    }

    /// Updates a vector store.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn update(
        &self,
        vector_store_id: impl AsRef<str>,
        params: VectorStoreUpdateParams,
    ) -> Result<VectorStore> {
        self.client
            .post_json(
                &format!(
                    "/vector_stores/{}",
                    urlencoding::encode(vector_store_id.as_ref())
                ),
                &params,
            )
            .await
    }

    /// Lists vector stores with optional query parameters.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(
        &self,
        params: Option<VectorStoreListParams>,
    ) -> Result<CursorPage<VectorStore>> {
        match params {
            Some(p) => {
                self.client
                    .get_cursor_page_query("/vector_stores", &p)
                    .await
            }
            None => self.client.get_cursor_page("/vector_stores").await,
        }
    }

    /// Deletes a vector store.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(&self, vector_store_id: impl AsRef<str>) -> Result<VectorStoreDeleted> {
        self.client
            .delete_json(&format!(
                "/vector_stores/{}",
                urlencoding::encode(vector_store_id.as_ref())
            ))
            .await
    }

    /// Searches a vector store, returning a paginated list of search results.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn search(
        &self,
        vector_store_id: impl AsRef<str>,
        params: VectorStoreSearchParams,
    ) -> Result<Page<VectorStoreSearchResult>> {
        self.client
            .post_json(
                &format!(
                    "/vector_stores/{}/search",
                    urlencoding::encode(vector_store_id.as_ref())
                ),
                &params,
            )
            .await
    }
}

// ---------------------------------------------------------------------------
// Vector store file service
// ---------------------------------------------------------------------------

/// Service for managing individual files within a specific vector store.
#[derive(Clone)]
pub struct VectorStoreFileService {
    client: Client,
    vector_store_id: String,
}

impl VectorStoreFileService {
    pub(crate) fn new(client: Client, vector_store_id: String) -> Self {
        Self {
            client,
            vector_store_id,
        }
    }

    /// Attaches a file to the vector store.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: VectorStoreFileCreateParams) -> Result<VectorStoreFile> {
        self.client
            .post_json(
                &format!(
                    "/vector_stores/{}/files",
                    urlencoding::encode(&self.vector_store_id)
                ),
                &params,
            )
            .await
    }

    /// Attaches a file and polls until processing reaches a terminal status.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create_and_poll(
        &self,
        params: VectorStoreFileCreateParams,
        interval: std::time::Duration,
    ) -> Result<VectorStoreFile> {
        let file = self.create(params).await?;
        self.poll_status(&file.id, interval).await
    }

    /// Alias for [`VectorStoreFileService::create_and_poll`].
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn new_and_poll(
        &self,
        params: VectorStoreFileCreateParams,
        interval: std::time::Duration,
    ) -> Result<VectorStoreFile> {
        self.create_and_poll(params, interval).await
    }

    /// Uploads a file through the Files API and attaches it to the vector store.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn upload(&self, params: crate::files::FileCreateParams) -> Result<VectorStoreFile> {
        let file = self.client.files().create(params).await?;
        self.create(VectorStoreFileCreateParams {
            file_id: file.id,
            attributes: None,
            chunking_strategy: None,
        })
        .await
    }

    /// Uploads a file, attaches it, and polls until processing reaches a terminal status.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn upload_and_poll(
        &self,
        params: crate::files::FileCreateParams,
        interval: std::time::Duration,
    ) -> Result<VectorStoreFile> {
        let file = self.upload(params).await?;
        self.poll_status(&file.id, interval).await
    }

    /// Retrieves a vector store file.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, file_id: impl AsRef<str>) -> Result<VectorStoreFile> {
        self.client
            .get_json(&format!(
                "/vector_stores/{}/files/{}",
                urlencoding::encode(&self.vector_store_id),
                urlencoding::encode(file_id.as_ref()),
            ))
            .await
    }

    /// Updates attributes on a vector store file.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn update(
        &self,
        file_id: impl AsRef<str>,
        params: VectorStoreFileUpdateParams,
    ) -> Result<VectorStoreFile> {
        self.client
            .post_json(
                &format!(
                    "/vector_stores/{}/files/{}",
                    urlencoding::encode(&self.vector_store_id),
                    urlencoding::encode(file_id.as_ref()),
                ),
                &params,
            )
            .await
    }

    /// Lists files in the vector store with optional query parameters.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(
        &self,
        params: Option<VectorStoreFileListParams>,
    ) -> Result<CursorPage<VectorStoreFile>> {
        let path = format!(
            "/vector_stores/{}/files",
            urlencoding::encode(&self.vector_store_id)
        );
        match params {
            Some(p) => self.client.get_cursor_page_query(&path, &p).await,
            None => self.client.get_cursor_page(&path).await,
        }
    }

    /// Deletes a file from the vector store (does not delete the underlying file).
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(&self, file_id: impl AsRef<str>) -> Result<VectorStoreFileDeleted> {
        self.client
            .delete_json(&format!(
                "/vector_stores/{}/files/{}",
                urlencoding::encode(&self.vector_store_id),
                urlencoding::encode(file_id.as_ref()),
            ))
            .await
    }

    /// Retrieves the parsed content of a vector store file.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn content(
        &self,
        file_id: impl AsRef<str>,
    ) -> Result<Page<VectorStoreFileContentResponse>> {
        self.client
            .get_page(&format!(
                "/vector_stores/{}/files/{}/content",
                urlencoding::encode(&self.vector_store_id),
                urlencoding::encode(file_id.as_ref()),
            ))
            .await
    }

    /// Polls until the file reaches a terminal status (`completed`, `failed`,
    /// or `cancelled`), sleeping for `interval` between each check.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures during polling.
    pub async fn poll_status(
        &self,
        file_id: impl AsRef<str>,
        interval: std::time::Duration,
    ) -> Result<VectorStoreFile> {
        let file_id = file_id.as_ref();
        loop {
            let file = self.get(file_id).await?;
            match file.status.as_str() {
                "completed" | "failed" | "cancelled" => return Ok(file),
                _ => tokio::time::sleep(interval).await,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Vector store file batch service
// ---------------------------------------------------------------------------

/// Service for managing file batches within a specific vector store.
#[derive(Clone)]
pub struct VectorStoreFileBatchService {
    client: Client,
    vector_store_id: String,
}

impl VectorStoreFileBatchService {
    pub(crate) fn new(client: Client, vector_store_id: String) -> Self {
        Self {
            client,
            vector_store_id,
        }
    }

    /// Creates a file batch in the vector store.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(
        &self,
        params: VectorStoreFileBatchCreateParams,
    ) -> Result<VectorStoreFileBatch> {
        self.client
            .post_json(
                &format!(
                    "/vector_stores/{}/file_batches",
                    urlencoding::encode(&self.vector_store_id)
                ),
                &params,
            )
            .await
    }

    /// Creates a file batch and polls until processing reaches a terminal status.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create_and_poll(
        &self,
        params: VectorStoreFileBatchCreateParams,
        interval: std::time::Duration,
    ) -> Result<VectorStoreFileBatch> {
        let batch = self.create(params).await?;
        self.poll_status(&batch.id, interval).await
    }

    /// Alias for [`VectorStoreFileBatchService::create_and_poll`].
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn new_and_poll(
        &self,
        params: VectorStoreFileBatchCreateParams,
        interval: std::time::Duration,
    ) -> Result<VectorStoreFileBatch> {
        self.create_and_poll(params, interval).await
    }

    /// Uploads files through the Files API, combines them with existing file IDs,
    /// creates a file batch, and polls until processing reaches a terminal status.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn upload_and_poll(
        &self,
        files: Vec<crate::files::FileCreateParams>,
        file_ids: Vec<String>,
        interval: std::time::Duration,
    ) -> Result<VectorStoreFileBatch> {
        let mut all_file_ids = file_ids;
        for params in files {
            let file = self.client.files().create(params).await?;
            all_file_ids.push(file.id);
        }

        self.create_and_poll(
            VectorStoreFileBatchCreateParams {
                file_ids: Some(all_file_ids),
                attributes: None,
                chunking_strategy: None,
                files: None,
            },
            interval,
        )
        .await
    }

    /// Retrieves a file batch.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, batch_id: impl AsRef<str>) -> Result<VectorStoreFileBatch> {
        self.client
            .get_json(&format!(
                "/vector_stores/{}/file_batches/{}",
                urlencoding::encode(&self.vector_store_id),
                urlencoding::encode(batch_id.as_ref()),
            ))
            .await
    }

    /// Cancels a file batch.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn cancel(&self, batch_id: impl AsRef<str>) -> Result<VectorStoreFileBatch> {
        let body = serde_json::json!({});
        self.client
            .post_json(
                &format!(
                    "/vector_stores/{}/file_batches/{}/cancel",
                    urlencoding::encode(&self.vector_store_id),
                    urlencoding::encode(batch_id.as_ref()),
                ),
                &body,
            )
            .await
    }

    /// Lists files in a batch with optional query parameters.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list_files(
        &self,
        batch_id: impl AsRef<str>,
        params: Option<VectorStoreFileBatchListFilesParams>,
    ) -> Result<CursorPage<VectorStoreFile>> {
        let path = format!(
            "/vector_stores/{}/file_batches/{}/files",
            urlencoding::encode(&self.vector_store_id),
            urlencoding::encode(batch_id.as_ref()),
        );
        match params {
            Some(p) => self.client.get_cursor_page_query(&path, &p).await,
            None => self.client.get_cursor_page(&path).await,
        }
    }

    /// Polls until the file batch reaches a terminal status (`completed`, `failed`,
    /// or `cancelled`), sleeping for `interval` between each check.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures during polling.
    pub async fn poll_status(
        &self,
        batch_id: impl AsRef<str>,
        interval: std::time::Duration,
    ) -> Result<VectorStoreFileBatch> {
        let batch_id = batch_id.as_ref();
        loop {
            let batch = self.get(batch_id).await?;
            match batch.status.as_str() {
                "completed" | "failed" | "cancelled" => return Ok(batch),
                _ => tokio::time::sleep(interval).await,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Filter for file list by status.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorStoreFileListFilter {
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Sort order for listing.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorStoreListOrder {
    Asc,
    Desc,
}

/// Chunking strategy for request params.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FileChunkingStrategyParam {
    /// Auto chunking (default).
    Auto,
    /// Static chunking with configurable parameters.
    Static {
        /// Static chunking configuration.
        #[serde(rename = "static")]
        config: StaticChunkingStrategy,
    },
}

/// Static chunking strategy parameters.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StaticChunkingStrategy {
    /// Number of tokens that overlap between chunks (default 400).
    pub chunk_overlap_tokens: i64,
    /// Max tokens per chunk (default 800, min 100, max 4096).
    pub max_chunk_size_tokens: i64,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Vector store create request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreCreateParams {
    /// Optional name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional seed files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Vector store update request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreUpdateParams {
    /// Optional updated name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional metadata replacement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Query parameters for listing vector stores.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreListParams {
    /// Pagination cursor (after).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Pagination cursor (before).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Maximum number of results (1..100, default 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<VectorStoreListOrder>,
}

/// Parameters for creating a file batch.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreFileBatchCreateParams {
    /// File IDs to include in the batch. Mutually exclusive with `files`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    /// Optional key-value attributes applied to all files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, serde_json::Value>>,
    /// Optional chunking strategy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: Option<FileChunkingStrategyParam>,
    /// Per-file overrides with `file_id`, `attributes`, and `chunking_strategy`.
    /// Mutually exclusive with `file_ids`. Maximum batch size: 2000.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<VectorStoreFileBatchNewParamsFile>>,
}

/// Per-file override for creating a file batch.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreFileBatchNewParamsFile {
    /// File ID to attach to the vector store.
    pub file_id: String,
    /// Optional key-value attributes for this file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, serde_json::Value>>,
    /// Optional chunking strategy for this file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: Option<FileChunkingStrategyParam>,
}

/// Query parameters for listing files in a file batch.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreFileBatchListFilesParams {
    /// Pagination cursor (after).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Pagination cursor (before).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Maximum number of results (1..100, default 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Filter by file status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<VectorStoreFileListFilter>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<VectorStoreListOrder>,
}

/// Parameters for creating (attaching) a file to a vector store.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreFileCreateParams {
    /// The file ID to attach.
    pub file_id: String,
    /// Optional key-value attributes for this file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, serde_json::Value>>,
    /// Optional chunking strategy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: Option<FileChunkingStrategyParam>,
}

/// Parameters for updating attributes on a vector store file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreFileUpdateParams {
    /// Key-value attributes to set on the file.
    pub attributes: HashMap<String, serde_json::Value>,
}

/// Query parameters for listing files in a vector store.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreFileListParams {
    /// Pagination cursor (after).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Pagination cursor (before).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Maximum number of results (1..100, default 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Filter by file status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<VectorStoreFileListFilter>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<VectorStoreListOrder>,
}

/// Vector store search request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreSearchParams {
    /// Search query text.
    pub query: String,
    /// Optional max number of returned chunks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_num_results: Option<u32>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Vector store object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStore {
    pub id: String,
    pub object: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_counts: Option<VectorStoreFileCounts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_active_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_bytes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<VectorStoreExpiresAfter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
}

/// File processing counts for a vector store.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreFileCounts {
    #[serde(default)]
    pub cancelled: i64,
    #[serde(default)]
    pub completed: i64,
    #[serde(default)]
    pub failed: i64,
    #[serde(default)]
    pub in_progress: i64,
    #[serde(default)]
    pub total: i64,
}

/// Expiration policy for a vector store.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreExpiresAfter {
    pub anchor: String,
    pub days: i64,
}

/// Vector store delete response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreDeleted {
    /// Deleted vector store ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Deletion flag.
    pub deleted: bool,
}

/// Vector store search match.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreSearchResult {
    /// Matching file ID.
    pub file_id: String,
    /// The filename of the matched file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// Similarity score.
    pub score: f64,
    /// Content chunks from the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<VectorStoreSearchResultContent>>,
    /// Optional attributes on the matched file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, serde_json::Value>>,
}

/// A content chunk returned from a vector store search.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreSearchResultContent {
    /// The text content.
    pub text: String,
    /// The content type (currently only `"text"`).
    #[serde(rename = "type")]
    pub content_type: String,
}

/// A file attached to a vector store.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreFile {
    /// File identifier.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Unix timestamp for when the file was created.
    pub created_at: i64,
    /// Status of the file processing (`in_progress`, `completed`, `failed`, `cancelled`).
    pub status: String,
    /// Vector store ID the file belongs to.
    pub vector_store_id: String,
    /// Total vector store usage in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_bytes: Option<i64>,
    /// Optional last error information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<VectorStoreFileLastError>,
}

/// Last error associated with a vector store file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreFileLastError {
    /// Error code (e.g. `server_error`, `unsupported_file`, `invalid_file`).
    pub code: String,
    /// A human-readable error description.
    pub message: String,
}

/// Response for deleting a vector store file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreFileDeleted {
    /// Deleted file ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Deletion flag.
    pub deleted: bool,
}

/// Parsed content of a vector store file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreFileContentResponse {
    /// The text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// The content type (currently only `"text"`).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

/// A batch of files attached to a vector store.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreFileBatch {
    /// Batch identifier.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Unix timestamp for when the batch was created.
    pub created_at: i64,
    /// Status of the batch (`in_progress`, `completed`, `failed`, `cancelled`).
    pub status: String,
    /// Vector store ID.
    pub vector_store_id: String,
    /// File count breakdown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_counts: Option<VectorStoreFileBatchFileCounts>,
}

/// File count breakdown for a vector store file batch.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreFileBatchFileCounts {
    /// Files cancelled.
    pub cancelled: i64,
    /// Files completed.
    pub completed: i64,
    /// Files failed.
    pub failed: i64,
    /// Files in progress.
    pub in_progress: i64,
    /// Total files.
    pub total: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_params_omit_optional_fields() {
        let params = VectorStoreCreateParams {
            name: None,
            file_ids: None,
            metadata: None,
        };

        let value = serde_json::to_value(params).expect("serialize vector store create params");
        assert!(value.get("name").is_none());
        assert!(value.get("file_ids").is_none());
        assert!(value.get("metadata").is_none());
    }

    #[test]
    fn vector_store_deserializes_status() {
        let json = r#"{
            "id":"vs_1",
            "object":"vector_store",
            "status":"completed"
        }"#;

        let store: VectorStore = serde_json::from_str(json).expect("deserialize vector store");
        assert_eq!(store.status.as_deref(), Some("completed"));
    }

    #[test]
    fn vector_store_search_params_omit_optional_max_results() {
        let params = VectorStoreSearchParams {
            query: "hello".to_owned(),
            max_num_results: None,
        };

        let value = serde_json::to_value(params).expect("serialize search params");
        assert!(value.get("max_num_results").is_none());
    }

    #[test]
    fn file_batch_create_params_with_files() {
        let mut attrs = HashMap::new();
        attrs.insert("category".to_owned(), serde_json::json!("docs"));

        let params = VectorStoreFileBatchCreateParams {
            file_ids: None,
            attributes: None,
            chunking_strategy: None,
            files: Some(vec![
                VectorStoreFileBatchNewParamsFile {
                    file_id: "file-1".to_owned(),
                    attributes: Some(attrs),
                    chunking_strategy: Some(FileChunkingStrategyParam::Auto),
                },
                VectorStoreFileBatchNewParamsFile {
                    file_id: "file-2".to_owned(),
                    attributes: None,
                    chunking_strategy: None,
                },
            ]),
        };

        let value = serde_json::to_value(&params).expect("serialize with files");
        assert!(value.get("file_ids").is_none());
        let files = value["files"].as_array().unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0]["file_id"], "file-1");
        assert_eq!(files[0]["attributes"]["category"], "docs");
        assert_eq!(files[0]["chunking_strategy"]["type"], "auto");
        assert_eq!(files[1]["file_id"], "file-2");
    }

    #[test]
    fn file_batch_list_files_params_default() {
        let params = VectorStoreFileBatchListFilesParams::default();
        let value = serde_json::to_value(&params).expect("serialize");
        assert!(value.get("after").is_none());
        assert!(value.get("limit").is_none());
        assert!(value.get("filter").is_none());
        assert!(value.get("order").is_none());
    }

    #[test]
    fn file_batch_list_files_params_serialize_all() {
        let params = VectorStoreFileBatchListFilesParams {
            after: Some("file-abc".to_owned()),
            before: Some("file-xyz".to_owned()),
            limit: Some(25),
            filter: Some(VectorStoreFileListFilter::Completed),
            order: Some(VectorStoreListOrder::Desc),
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["after"], "file-abc");
        assert_eq!(value["limit"], 25);
        assert_eq!(value["filter"], "completed");
        assert_eq!(value["order"], "desc");
    }

    #[test]
    fn vector_store_list_params_serialize_all() {
        let params = VectorStoreListParams {
            after: Some("vs_abc".to_owned()),
            before: Some("vs_xyz".to_owned()),
            limit: Some(50),
            order: Some(VectorStoreListOrder::Desc),
        };
        let value = serde_json::to_value(params).expect("serialize");
        assert_eq!(value["after"], "vs_abc");
        assert_eq!(value["limit"], 50);
        assert_eq!(value["order"], "desc");
    }

    // -----------------------------------------------------------------------
    // VectorStoreFileService types
    // -----------------------------------------------------------------------

    #[test]
    fn file_create_params_serialize() {
        let params = VectorStoreFileCreateParams {
            file_id: "file-abc123".to_owned(),
            attributes: None,
            chunking_strategy: None,
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["file_id"], "file-abc123");
        assert!(value.get("attributes").is_none());
        assert!(value.get("chunking_strategy").is_none());
    }

    #[test]
    fn file_create_params_serialize_with_attributes() {
        let mut attrs = HashMap::new();
        attrs.insert("category".to_owned(), serde_json::json!("manual"));

        let params = VectorStoreFileCreateParams {
            file_id: "file-xyz".to_owned(),
            attributes: Some(attrs),
            chunking_strategy: Some(FileChunkingStrategyParam::Auto),
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["file_id"], "file-xyz");
        assert_eq!(value["attributes"]["category"], "manual");
        assert_eq!(value["chunking_strategy"]["type"], "auto");
    }

    #[test]
    fn file_update_params_serialize() {
        let mut attrs = HashMap::new();
        attrs.insert("priority".to_owned(), serde_json::json!(1));

        let params = VectorStoreFileUpdateParams { attributes: attrs };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["attributes"]["priority"], 1);
    }

    #[test]
    fn file_list_params_default_omits_all() {
        let params = VectorStoreFileListParams::default();
        let value = serde_json::to_value(&params).expect("serialize");
        assert!(value.get("after").is_none());
        assert!(value.get("before").is_none());
        assert!(value.get("limit").is_none());
        assert!(value.get("filter").is_none());
        assert!(value.get("order").is_none());
    }

    #[test]
    fn file_list_params_serialize_all() {
        let params = VectorStoreFileListParams {
            after: Some("file-a".to_owned()),
            before: Some("file-z".to_owned()),
            limit: Some(10),
            filter: Some(VectorStoreFileListFilter::InProgress),
            order: Some(VectorStoreListOrder::Asc),
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["after"], "file-a");
        assert_eq!(value["before"], "file-z");
        assert_eq!(value["limit"], 10);
        assert_eq!(value["filter"], "in_progress");
        assert_eq!(value["order"], "asc");
    }

    #[test]
    fn vector_store_file_deserializes() {
        let json = r#"{
            "id": "file-abc",
            "object": "vector_store.file",
            "created_at": 1700000000,
            "status": "completed",
            "vector_store_id": "vs_123",
            "usage_bytes": 4096
        }"#;
        let file: VectorStoreFile = serde_json::from_str(json).expect("deserialize");
        assert_eq!(file.id, "file-abc");
        assert_eq!(file.status, "completed");
        assert_eq!(file.usage_bytes, Some(4096));
        assert!(file.last_error.is_none());
    }

    #[test]
    fn vector_store_file_deserializes_with_error() {
        let json = r#"{
            "id": "file-bad",
            "object": "vector_store.file",
            "created_at": 1700000000,
            "status": "failed",
            "vector_store_id": "vs_123",
            "last_error": {
                "code": "unsupported_file",
                "message": "File type not supported"
            }
        }"#;
        let file: VectorStoreFile = serde_json::from_str(json).expect("deserialize");
        assert_eq!(file.status, "failed");
        let err = file.last_error.unwrap();
        assert_eq!(err.code, "unsupported_file");
        assert_eq!(err.message, "File type not supported");
    }

    #[test]
    fn vector_store_file_deleted_deserializes() {
        let json = r#"{
            "id": "file-abc",
            "object": "vector_store.file.deleted",
            "deleted": true
        }"#;
        let deleted: VectorStoreFileDeleted = serde_json::from_str(json).expect("deserialize");
        assert_eq!(deleted.id, "file-abc");
        assert!(deleted.deleted);
    }

    #[test]
    fn vector_store_file_content_response_deserializes() {
        let json = r#"{
            "text": "Hello world",
            "type": "text"
        }"#;
        let content: VectorStoreFileContentResponse =
            serde_json::from_str(json).expect("deserialize");
        assert_eq!(content.text.as_deref(), Some("Hello world"));
        assert_eq!(content.content_type.as_deref(), Some("text"));
    }

    // -----------------------------------------------------------------------
    // Search result types
    // -----------------------------------------------------------------------

    #[test]
    fn search_result_deserializes() {
        let json = r#"{
            "file_id": "file-abc",
            "filename": "doc.pdf",
            "score": 0.95,
            "content": [{"text": "relevant chunk", "type": "text"}],
            "attributes": {"category": "docs"}
        }"#;
        let result: VectorStoreSearchResult = serde_json::from_str(json).expect("deserialize");
        assert_eq!(result.file_id, "file-abc");
        assert_eq!(result.filename.as_deref(), Some("doc.pdf"));
        assert!((result.score - 0.95).abs() < f64::EPSILON);
        let content = result.content.unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0].text, "relevant chunk");
        assert_eq!(content[0].content_type, "text");
    }

    #[test]
    fn search_result_page_deserializes() {
        let json = r#"{
            "object": "list",
            "data": [
                {"file_id": "file-1", "score": 0.9},
                {"file_id": "file-2", "score": 0.8}
            ]
        }"#;
        let page: crate::pagination::Page<VectorStoreSearchResult> =
            serde_json::from_str(json).expect("deserialize");
        assert_eq!(page.object, "list");
        assert_eq!(page.data.len(), 2);
        assert_eq!(page.data[0].file_id, "file-1");
        assert_eq!(page.data[1].file_id, "file-2");
    }

    // -----------------------------------------------------------------------
    // File batch types
    // -----------------------------------------------------------------------

    #[test]
    fn file_batch_deserializes_with_counts() {
        let json = r#"{
            "id": "vsfb_123",
            "object": "vector_store.files_batch",
            "created_at": 1700000000,
            "status": "in_progress",
            "vector_store_id": "vs_456",
            "file_counts": {
                "cancelled": 0,
                "completed": 3,
                "failed": 1,
                "in_progress": 2,
                "total": 6
            }
        }"#;
        let batch: VectorStoreFileBatch = serde_json::from_str(json).expect("deserialize");
        assert_eq!(batch.id, "vsfb_123");
        assert_eq!(batch.status, "in_progress");
        let counts = batch.file_counts.unwrap();
        assert_eq!(counts.completed, 3);
        assert_eq!(counts.failed, 1);
        assert_eq!(counts.in_progress, 2);
        assert_eq!(counts.total, 6);
    }

    #[test]
    fn file_batch_deserializes_without_counts() {
        let json = r#"{
            "id": "vsfb_123",
            "object": "vector_store.files_batch",
            "created_at": 1700000000,
            "status": "completed",
            "vector_store_id": "vs_456"
        }"#;
        let batch: VectorStoreFileBatch = serde_json::from_str(json).expect("deserialize");
        assert_eq!(batch.status, "completed");
        assert!(batch.file_counts.is_none());
    }
}
