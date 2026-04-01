//! Upload APIs for multi-part large file ingestion.

use reqwest::multipart::{Form, Part};

use crate::Client;
use crate::Result;

/// Upload service.
#[derive(Clone)]
pub struct UploadService {
    client: Client,
}

impl UploadService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates an upload session.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: UploadCreateParams) -> Result<Upload> {
        self.client.post_json("/uploads", &params).await
    }

    /// Gets an upload session by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, upload_id: impl AsRef<str>) -> Result<Upload> {
        self.client
            .get_json(&format!(
                "/uploads/{}",
                urlencoding::encode(upload_id.as_ref())
            ))
            .await
    }

    /// Uploads one part in a session.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create_part(
        &self,
        upload_id: impl AsRef<str>,
        params: UploadPartCreateParams,
    ) -> Result<UploadPart> {
        let bytes = params.data;
        let filename = params.filename.unwrap_or_else(|| "part.bin".to_owned());
        let content_type = params.content_type;

        self.client
            .post_multipart_json(
                &format!("/uploads/{}/parts", urlencoding::encode(upload_id.as_ref())),
                move || {
                    let base_part = Part::bytes(bytes.clone()).file_name(filename.clone());
                    let part = if let Some(content_type) = content_type.as_deref() {
                        match base_part.mime_str(content_type) {
                            Ok(part) => part,
                            Err(_) => Part::bytes(bytes.clone()).file_name(filename.clone()),
                        }
                    } else {
                        base_part
                    };
                    Form::new().part("data", part)
                },
            )
            .await
    }

    /// Completes an upload after all parts are uploaded.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn complete(
        &self,
        upload_id: impl AsRef<str>,
        params: UploadCompleteParams,
    ) -> Result<Upload> {
        self.client
            .post_json(
                &format!(
                    "/uploads/{}/complete",
                    urlencoding::encode(upload_id.as_ref())
                ),
                &params,
            )
            .await
    }

    /// Cancels an upload session.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn cancel(&self, upload_id: impl AsRef<str>) -> Result<Upload> {
        let body = serde_json::json!({});
        self.client
            .post_json(
                &format!(
                    "/uploads/{}/cancel",
                    urlencoding::encode(upload_id.as_ref())
                ),
                &body,
            )
            .await
    }
}

/// Upload session create request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadCreateParams {
    /// Total byte count expected across parts.
    pub bytes: u64,
    /// Display filename for resulting file.
    pub filename: String,
    /// Purpose for the eventual file object.
    pub purpose: String,
    /// Optional MIME type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Upload part create request.
#[derive(Debug, Clone)]
pub struct UploadPartCreateParams {
    /// Raw part bytes.
    pub data: Vec<u8>,
    /// Optional filename hint for part.
    pub filename: Option<String>,
    /// Optional MIME type.
    pub content_type: Option<String>,
}

/// Upload complete request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadCompleteParams {
    /// Ordered part IDs to assemble.
    pub part_ids: Vec<String>,
}

/// Upload session response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Upload {
    /// Upload ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Byte count.
    pub bytes: u64,
    /// Filename.
    pub filename: String,
    /// Purpose.
    pub purpose: String,
    /// Status when provided by API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Upload part response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadPart {
    /// Upload part ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Upload session ID when provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{Upload, UploadCompleteParams, UploadCreateParams};

    #[test]
    fn upload_create_params_omit_optional_mime_type() {
        let params = UploadCreateParams {
            bytes: 42,
            filename: "data.jsonl".to_owned(),
            purpose: "fine-tune".to_owned(),
            mime_type: None,
        };

        let value = serde_json::to_value(params).expect("serialize upload create params");
        assert_eq!(
            value.get("bytes"),
            Some(&serde_json::Value::Number(42_u64.into()))
        );
        assert!(value.get("mime_type").is_none());
    }

    #[test]
    fn upload_deserializes_optional_status() {
        let json = r#"{
            "id":"upload_1",
            "object":"upload",
            "bytes":5,
            "filename":"input.bin",
            "purpose":"assistants",
            "status":"cancelled"
        }"#;

        let upload: Upload = serde_json::from_str(json).expect("deserialize upload");
        assert_eq!(upload.id, "upload_1");
        assert_eq!(upload.status.as_deref(), Some("cancelled"));
    }

    #[test]
    fn upload_complete_params_serialize_part_ids() {
        let params = UploadCompleteParams {
            part_ids: vec!["part_1".to_owned(), "part_2".to_owned()],
        };

        let value = serde_json::to_value(params).expect("serialize upload complete params");
        let ids = value
            .get("part_ids")
            .and_then(serde_json::Value::as_array)
            .expect("part_ids array");
        assert_eq!(ids.len(), 2);
    }
}
