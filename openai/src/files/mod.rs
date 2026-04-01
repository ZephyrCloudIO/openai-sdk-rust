//! File upload and management APIs.

use reqwest::multipart::{Form, Part};

use crate::{Client, Result};

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
        let purpose = params.purpose;
        let bytes = params.file.bytes;
        let filename = params.file.filename;
        let content_type = params.file.content_type;

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

                Form::new()
                    .text("purpose", purpose.clone())
                    .part("file", part)
            })
            .await
    }

    /// Lists files.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(&self) -> Result<FileList> {
        self.client.get_json("/files").await
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

/// File upload request.
#[derive(Debug, Clone)]
pub struct FileCreateParams {
    /// File usage purpose.
    pub purpose: String,
    /// File payload.
    pub file: FileUploadPart,
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
    /// Purpose.
    pub purpose: String,
    /// File status if returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
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
    use super::{content_path, DeletedFile, FileList, FileUploadPart};

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
}
