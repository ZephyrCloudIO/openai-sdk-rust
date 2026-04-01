//! Models APIs.

use crate::{shared::ModelId, Client, Result};

/// Models service.
#[derive(Clone)]
pub struct ModelService {
    client: Client,
}

impl ModelService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Lists models.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(&self) -> Result<ModelList> {
        self.client.get_json("/models").await
    }

    /// Gets one model by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, model_id: impl AsRef<str>) -> Result<Model> {
        self.client
            .get_json(&format!(
                "/models/{}",
                urlencoding::encode(model_id.as_ref())
            ))
            .await
    }

    /// Deletes a fine-tuned model.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(&self, model_id: impl AsRef<str>) -> Result<DeletedModel> {
        self.client
            .delete_json(&format!(
                "/models/{}",
                urlencoding::encode(model_id.as_ref())
            ))
            .await
    }
}

/// Model list response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelList {
    /// Object type.
    pub object: String,
    /// Model records.
    pub data: Vec<Model>,
}

/// Model record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Model {
    /// Model ID.
    pub id: ModelId,
    /// Object type.
    pub object: String,
    /// Owner org.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owned_by: Option<String>,
}

/// Model delete response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeletedModel {
    /// Deleted model ID.
    pub id: ModelId,
    /// Object type.
    pub object: String,
    /// Deletion flag.
    pub deleted: bool,
}

#[cfg(test)]
mod tests {
    use super::{DeletedModel, ModelList};

    #[test]
    fn model_list_deserializes_model_ids() {
        let json = r#"{
            "object":"list",
            "data":[{"id":"gpt-4o-mini","object":"model","owned_by":"openai"}]
        }"#;

        let list: ModelList = serde_json::from_str(json).expect("deserialize model list");
        assert_eq!(list.data.len(), 1);
        assert_eq!(list.data[0].id.as_ref(), "gpt-4o-mini");
    }

    #[test]
    fn deleted_model_serializes_with_string_id() {
        let deleted = DeletedModel {
            id: "ft:gpt-4o:custom".into(),
            object: "model".to_owned(),
            deleted: true,
        };

        let value = serde_json::to_value(deleted).expect("serialize deleted model");
        assert_eq!(
            value.get("id"),
            Some(&serde_json::Value::String("ft:gpt-4o:custom".to_owned()))
        );
        assert_eq!(value.get("deleted"), Some(&serde_json::Value::Bool(true)));
    }
}
