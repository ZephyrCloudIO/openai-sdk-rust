//! File upload and management APIs.

use reqwest::multipart::{Form, Part};

use crate::{pagination::CursorPage, Client, Result};

/// Files service.
#[derive(Clone)]
pub struct FileService {
    client: Client,
}

impl FileService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Uploads a file.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: FileCreateParams) -> Result<FileObject> {
        let purpose = serde_json::to_value(&params.purpose)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        let bytes = params.file.bytes;
        let filename = params.file.filename;
        let content_type = params.file.content_type;
        let expires_after = params.expires_after;

        self.client
            .post_multipart_json("/files", move || {
                let base_part = Part::bytes(bytes.clone()).file_name(filename.clone());
                let part = if let Some(content_type) = content_type.as_deref() {
                    match base_part.mime_str(content_type) {
                        Ok(part) => part,
                        Err(_) => Part::bytes(bytes.clone()).file_name(filename.clone()),
                    }
                } else {
                    base_part
                };

                let mut form = Form::new()
                    .text("purpose", purpose.clone())
                    .part("file", part);

                if let Some(ref ea) = expires_after {
                    form = form.text("expires_after[anchor]", ea.anchor.clone());
                    form = form.text("expires_after[seconds]", ea.seconds.to_string());
                }

                form
            })
            .await
    }

    /// Lists files with optional query parameters.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(&self, params: Option<FileListParams>) -> Result<CursorPage<FileObject>> {
        match params {
            Some(p) => self.client.get_cursor_page_query("/files", &p).await,
            None => self.client.get_cursor_page("/files").await,
        }
    }

    /// Gets one file by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, file_id: impl AsRef<str>) -> Result<FileObject> {
        self.client
            .get_json(&format!("/files/{}", urlencoding::encode(file_id.as_ref())))
            .await
    }

    /// Deletes a file by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(&self, file_id: impl AsRef<str>) -> Result<DeletedFile> {
        self.client
            .delete_json(&format!("/files/{}", urlencoding::encode(file_id.as_ref())))
            .await
    }

    /// Downloads raw file content.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn content(&self, file_id: impl AsRef<str>) -> Result<Vec<u8>> {
        self.client.get_bytes(&content_path(file_id.as_ref())).await
    }
}

fn content_path(file_id: &str) -> String {
    format!("/files/{}/content", urlencoding::encode(file_id))
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// The intended purpose of the file as returned by the API.
///
/// Supported values: `assistants`, `assistants_output`, `batch`, `batch_output`,
/// `fine-tune`, `fine-tune-results`, `vision`, and `user_data`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileObjectPurpose {
    Assistants,
    AssistantsOutput,
    Batch,
    BatchOutput,
    #[serde(rename = "fine-tune")]
    FineTune,
    #[serde(rename = "fine-tune-results")]
    FineTuneResults,
    Vision,
    UserData,
}

/// The current status of a file object.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileObjectStatus {
    Uploaded,
    Processed,
    Error,
}

/// The purpose field used for upload requests.
///
/// Supported values: `assistants`, `batch`, `fine-tune`, `vision`, `user_data`,
/// `evals`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilePurpose {
    Assistants,
    Batch,
    #[serde(rename = "fine-tune")]
    FineTune,
    Vision,
    UserData,
    Evals,
}

/// Sort order for listing files.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileListOrder {
    Asc,
    Desc,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// File upload request.
#[derive(Debug, Clone)]
pub struct FileCreateParams {
    /// File usage purpose.
    pub purpose: FilePurpose,
    /// File payload.
    pub file: FileUploadPart,
    /// Optional expiration policy.
    pub expires_after: Option<FileNewParamsExpiresAfter>,
}

/// Expiration policy for uploaded files.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileNewParamsExpiresAfter {
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

/// Query parameters for listing files.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FileListParams {
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Maximum number of results (1..10000, default 10000).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Only return files with the given purpose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    /// Sort order by `created_at` timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<FileListOrder>,
}

/// In-memory file upload part.
#[derive(Debug, Clone)]
pub struct FileUploadPart {
    /// File bytes.
    pub bytes: Vec<u8>,
    /// File name presented to API.
    pub filename: String,
    /// Optional MIME content type.
    pub content_type: Option<String>,
}

impl FileUploadPart {
    /// Creates a file part from bytes.
    #[must_use]
    pub fn from_bytes(bytes: impl Into<Vec<u8>>, filename: impl Into<String>) -> Self {
        Self {
            bytes: bytes.into(),
            filename: filename.into(),
            content_type: None,
        }
    }

    /// Sets MIME type for uploaded part.
    #[must_use]
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// File list response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileList {
    /// Object type.
    pub object: String,
    /// File data.
    pub data: Vec<FileObject>,
}

/// File object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileObject {
    /// File ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Byte size.
    pub bytes: u64,
    /// Unix timestamp for creation.
    pub created_at: u64,
    /// Filename.
    pub filename: String,
    /// Purpose (typed enum).
    pub purpose: FileObjectPurpose,
    /// File status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<FileObjectStatus>,
    /// Unix timestamp for expiration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    /// Status details (deprecated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_details: Option<String>,
}

/// File delete response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeletedFile {
    /// File ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Deletion flag.
    pub deleted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_upload_part_sets_content_type() {
        let part = FileUploadPart::from_bytes(vec![1_u8, 2, 3], "sample.txt")
            .with_content_type("text/plain");

        assert_eq!(part.filename, "sample.txt");
        assert_eq!(part.content_type.as_deref(), Some("text/plain"));
        assert_eq!(part.bytes, vec![1_u8, 2, 3]);
    }

    #[test]
    fn file_list_deserializes_file_object() {
        let json = r#"{
            "object":"list",
            "data":[{
                "id":"file_1",
                "object":"file",
                "bytes":5,
                "created_at":123,
                "filename":"input.txt",
                "purpose":"assistants"
            }]
        }"#;

        let list: FileList = serde_json::from_str(json).expect("deserialize file list");
        assert_eq!(list.data.len(), 1);
        assert_eq!(list.data[0].id, "file_1");
        assert_eq!(list.data[0].filename, "input.txt");
        assert_eq!(list.data[0].purpose, FileObjectPurpose::Assistants);
    }

    #[test]
    fn file_object_purpose_enum_round_trips() {
        let purposes = vec![
            (FileObjectPurpose::Assistants, "\"assistants\""),
            (FileObjectPurpose::AssistantsOutput, "\"assistants_output\""),
            (FileObjectPurpose::Batch, "\"batch\""),
            (FileObjectPurpose::BatchOutput, "\"batch_output\""),
            (FileObjectPurpose::FineTune, "\"fine-tune\""),
            (FileObjectPurpose::FineTuneResults, "\"fine-tune-results\""),
            (FileObjectPurpose::Vision, "\"vision\""),
            (FileObjectPurpose::UserData, "\"user_data\""),
        ];
        for (variant, expected_json) in purposes {
            let serialized = serde_json::to_string(&variant).expect("serialize");
            assert_eq!(serialized, expected_json);
            let deserialized: FileObjectPurpose =
                serde_json::from_str(&serialized).expect("deserialize");
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn file_object_status_enum_round_trips() {
        for variant in [
            FileObjectStatus::Uploaded,
            FileObjectStatus::Processed,
            FileObjectStatus::Error,
        ] {
            let serialized = serde_json::to_string(&variant).expect("serialize");
            let deserialized: FileObjectStatus =
                serde_json::from_str(&serialized).expect("deserialize");
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn file_purpose_enum_round_trips() {
        for variant in [
            FilePurpose::Assistants,
            FilePurpose::Batch,
            FilePurpose::FineTune,
            FilePurpose::Vision,
            FilePurpose::UserData,
            FilePurpose::Evals,
        ] {
            let serialized = serde_json::to_string(&variant).expect("serialize");
            let deserialized: FilePurpose = serde_json::from_str(&serialized).expect("deserialize");
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn file_object_with_optional_fields() {
        let json = r#"{
            "id":"file_1",
            "object":"file",
            "bytes":5,
            "created_at":123,
            "filename":"input.txt",
            "purpose":"batch",
            "status":"processed",
            "expires_at":999,
            "status_details":"some detail"
        }"#;

        let file: FileObject = serde_json::from_str(json).expect("deserialize");
        assert_eq!(file.purpose, FileObjectPurpose::Batch);
        assert_eq!(file.status, Some(FileObjectStatus::Processed));
        assert_eq!(file.expires_at, Some(999));
        assert_eq!(file.status_details.as_deref(), Some("some detail"));
    }

    #[test]
    fn file_list_params_omit_optional_fields() {
        let params = FileListParams::default();
        let value = serde_json::to_value(params).expect("serialize");
        assert!(value.get("after").is_none());
        assert!(value.get("limit").is_none());
        assert!(value.get("purpose").is_none());
        assert!(value.get("order").is_none());
    }

    #[test]
    fn file_list_params_serialize_all() {
        let params = FileListParams {
            after: Some("file_abc".to_owned()),
            limit: Some(50),
            purpose: Some("assistants".to_owned()),
            order: Some(FileListOrder::Desc),
        };
        let value = serde_json::to_value(params).expect("serialize");
        assert_eq!(value["after"], "file_abc");
        assert_eq!(value["limit"], 50);
        assert_eq!(value["purpose"], "assistants");
        assert_eq!(value["order"], "desc");
    }

    #[test]
    fn deleted_file_deserializes_flag() {
        let json = r#"{
            "id":"file_1",
            "object":"file",
            "deleted":true
        }"#;

        let deleted: DeletedFile = serde_json::from_str(json).expect("deserialize deleted file");
        assert_eq!(deleted.id, "file_1");
        assert!(deleted.deleted);
    }

    #[test]
    fn content_path_encodes_file_id() {
        let path = content_path("file:abc/123");
        assert_eq!(path, "/files/file%3Aabc%2F123/content");
    }

    #[test]
    fn file_new_params_expires_after_serializes() {
        let ea = FileNewParamsExpiresAfter {
            seconds: 3600,
            anchor: "created_at".to_owned(),
        };
        let value = serde_json::to_value(&ea).expect("serialize");
        assert_eq!(value["seconds"], 3600);
        assert_eq!(value["anchor"], "created_at");
    }
}
