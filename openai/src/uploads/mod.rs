//! Upload APIs for multi-part large file ingestion.

use reqwest::multipart::{Form, Part};

use crate::files::FileObject;
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

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Status of an upload session.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UploadStatus {
    Pending,
    Completed,
    Cancelled,
    Expired,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Upload session create request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadCreateParams {
    /// Total byte count expected across parts.
    pub bytes: u64,
    /// Display filename for resulting file.
    pub filename: String,
    /// Purpose for the eventual file object.
    pub purpose: String,
    /// MIME type of the file (required).
    pub mime_type: String,
    /// Optional expiration policy for the resulting file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<UploadExpiresAfter>,
}

/// Expiration policy for an upload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadExpiresAfter {
    /// Seconds after anchor before the file expires.
    /// Must be between 3600 (1 hour) and 2592000 (30 days).
    pub seconds: i64,
    /// Anchor timestamp. Supported: `created_at`.
    #[serde(default = "default_created_at_anchor")]
    pub anchor: String,
}

fn default_created_at_anchor() -> String {
    "created_at".to_owned()
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
    /// Optional md5 checksum to verify uploaded bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

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
    /// Unix timestamp (in seconds) for when the upload was created.
    #[serde(default)]
    pub created_at: i64,
    /// Unix timestamp (in seconds) for when the upload will expire.
    #[serde(default)]
    pub expires_at: i64,
    /// Status (typed enum).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<UploadStatus>,
    /// The resulting file object (populated on completion).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FileObject>,
}

/// Upload part response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadPart {
    /// Upload part ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Upload session ID.
    #[serde(default)]
    pub upload_id: String,
    /// Unix timestamp (in seconds) for when the part was created.
    #[serde(default)]
    pub created_at: i64,
}

#[cfg(test)]
mod tests {
    use super::{
        Upload, UploadCompleteParams, UploadCreateParams, UploadExpiresAfter, UploadPart,
        UploadStatus,
    };

    #[test]
    fn upload_create_params_requires_mime_type() {
        let params = UploadCreateParams {
            bytes: 42,
            filename: "data.jsonl".to_owned(),
            purpose: "fine-tune".to_owned(),
            mime_type: "application/jsonl".to_owned(),
            expires_after: None,
        };

        let value = serde_json::to_value(params).expect("serialize upload create params");
        assert_eq!(
            value.get("bytes"),
            Some(&serde_json::Value::Number(42_u64.into()))
        );
        assert_eq!(
            value.get("mime_type").and_then(|v| v.as_str()),
            Some("application/jsonl")
        );
        assert!(value.get("expires_after").is_none());
    }

    #[test]
    fn upload_create_params_with_expires_after() {
        let params = UploadCreateParams {
            bytes: 100,
            filename: "data.jsonl".to_owned(),
            purpose: "fine-tune".to_owned(),
            mime_type: "application/jsonl".to_owned(),
            expires_after: Some(UploadExpiresAfter {
                seconds: 7200,
                anchor: "created_at".to_owned(),
            }),
        };

        let value = serde_json::to_value(params).expect("serialize");
        assert_eq!(value["expires_after"]["seconds"], 7200);
        assert_eq!(value["expires_after"]["anchor"], "created_at");
    }

    #[test]
    fn upload_expires_after_default_anchor() {
        let ea: UploadExpiresAfter =
            serde_json::from_str(r#"{"seconds":3600}"#).expect("deserialize");
        assert_eq!(ea.seconds, 3600);
        assert_eq!(ea.anchor, "created_at");
    }

    #[test]
    fn upload_deserializes_typed_status() {
        let json = r#"{
            "id":"upload_1",
            "object":"upload",
            "bytes":5,
            "filename":"input.bin",
            "purpose":"assistants",
            "status":"cancelled",
            "created_at":1700000000,
            "expires_at":1700003600
        }"#;

        let upload: Upload = serde_json::from_str(json).expect("deserialize upload");
        assert_eq!(upload.id, "upload_1");
        assert_eq!(upload.status, Some(UploadStatus::Cancelled));
        assert_eq!(upload.created_at, 1_700_000_000);
        assert_eq!(upload.expires_at, 1_700_003_600);
    }

    #[test]
    fn upload_defaults_timestamps_when_missing() {
        let json = r#"{
            "id":"upload_1",
            "object":"upload",
            "bytes":5,
            "filename":"input.bin",
            "purpose":"assistants"
        }"#;

        let upload: Upload = serde_json::from_str(json).expect("deserialize upload");
        assert_eq!(upload.created_at, 0);
        assert_eq!(upload.expires_at, 0);
    }

    #[test]
    fn upload_status_enum_round_trips() {
        for variant in [
            UploadStatus::Pending,
            UploadStatus::Completed,
            UploadStatus::Cancelled,
            UploadStatus::Expired,
        ] {
            let serialized = serde_json::to_string(&variant).expect("serialize");
            let deserialized: UploadStatus =
                serde_json::from_str(&serialized).expect("deserialize");
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn upload_complete_params_serialize_with_md5() {
        let params = UploadCompleteParams {
            part_ids: vec!["part_1".to_owned(), "part_2".to_owned()],
            md5: Some("abc123".to_owned()),
        };

        let value = serde_json::to_value(params).expect("serialize upload complete params");
        let ids = value
            .get("part_ids")
            .and_then(serde_json::Value::as_array)
            .expect("part_ids array");
        assert_eq!(ids.len(), 2);
        assert_eq!(value["md5"], "abc123");
    }

    #[test]
    fn upload_complete_params_omit_optional_md5() {
        let params = UploadCompleteParams {
            part_ids: vec!["part_1".to_owned()],
            md5: None,
        };
        let value = serde_json::to_value(params).expect("serialize");
        assert!(value.get("md5").is_none());
    }

    #[test]
    fn upload_with_file_field() {
        let json = r#"{
            "id":"upload_1",
            "object":"upload",
            "bytes":5,
            "filename":"input.bin",
            "purpose":"assistants",
            "status":"completed",
            "created_at":1700000000,
            "expires_at":1700003600,
            "file":{"id":"file-abc","object":"file","bytes":5,"created_at":123,"filename":"input.bin","purpose":"assistants"}
        }"#;

        let upload: Upload = serde_json::from_str(json).expect("deserialize upload with file");
        let file = upload.file.expect("file present");
        assert_eq!(file.id, "file-abc");
    }

    #[test]
    fn upload_part_with_created_at() {
        let json = r#"{
            "id":"part_1",
            "object":"upload.part",
            "upload_id":"upload_1",
            "created_at":1700000000
        }"#;
        let part: UploadPart = serde_json::from_str(json).expect("deserialize upload part");
        assert_eq!(part.created_at, 1_700_000_000);
        assert_eq!(part.upload_id, "upload_1");
    }

    #[test]
    fn upload_part_defaults_when_fields_missing() {
        let json = r#"{
            "id":"part_1",
            "object":"upload.part"
        }"#;
        let part: UploadPart = serde_json::from_str(json).expect("deserialize upload part");
        assert_eq!(part.upload_id, "");
        assert_eq!(part.created_at, 0);
    }
}
