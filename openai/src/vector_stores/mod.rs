//! Vector stores APIs.

use std::collections::HashMap;

use crate::{pagination::CursorPage, Client, Result};

/// Vector store service.
#[derive(Clone)]
pub struct VectorStoreService {
    client: Client,
}

impl VectorStoreService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
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

    /// Lists vector stores.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(&self) -> Result<CursorPage<VectorStore>> {
        self.client.get_json("/vector_stores").await
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

    /// Searches a vector store.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn search(
        &self,
        vector_store_id: impl AsRef<str>,
        params: VectorStoreSearchParams,
    ) -> Result<VectorStoreSearchResponse> {
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

/// Vector store object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStore {
    /// Vector store ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Optional display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Current status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
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

/// Vector store search request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreSearchParams {
    /// Search query text.
    pub query: String,
    /// Optional max number of returned chunks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_num_results: Option<u32>,
}

/// Vector store search response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreSearchResponse {
    /// Object type.
    pub object: String,
    /// Search matches.
    pub data: Vec<VectorStoreSearchResult>,
}

/// Vector store search match.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreSearchResult {
    /// Matching file ID.
    pub file_id: String,
    /// Similarity score.
    pub score: f32,
    /// Optional matching snippet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{VectorStore, VectorStoreCreateParams, VectorStoreSearchParams};

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
}
