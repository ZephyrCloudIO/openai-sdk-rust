//! Containers, container files, and container file content APIs.

use crate::{pagination::CursorPage, Client, Result};

/// Container service.
#[derive(Clone)]
pub struct ContainerService {
    client: Client,
}

impl ContainerService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a container.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: ContainerCreateParams) -> Result<Container> {
        self.client.post_json("/containers", &params).await
    }

    /// Gets a container by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, container_id: impl AsRef<str>) -> Result<Container> {
        self.client
            .get_json(&format!(
                "/containers/{}",
                urlencoding::encode(container_id.as_ref())
            ))
            .await
    }

    /// Lists containers.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(&self, params: ContainerListParams) -> Result<CursorPage<Container>> {
        self.client
            .get_cursor_page_query("/containers", &params)
            .await
    }

    /// Deletes a container.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(&self, container_id: impl AsRef<str>) -> Result<ContainerDeleted> {
        self.client
            .delete_json(&format!(
                "/containers/{}",
                urlencoding::encode(container_id.as_ref())
            ))
            .await
    }

    /// Returns the container files sub-service.
    #[must_use]
    pub fn files(&self) -> ContainerFileService {
        ContainerFileService::new(self.client.clone())
    }
}

/// Container file service (nested under containers).
#[derive(Clone)]
pub struct ContainerFileService {
    client: Client,
}

impl ContainerFileService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a file in a container (by file ID reference).
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(
        &self,
        container_id: impl AsRef<str>,
        params: ContainerFileCreateParams,
    ) -> Result<ContainerFile> {
        self.client
            .post_json(
                &format!(
                    "/containers/{}/files",
                    urlencoding::encode(container_id.as_ref())
                ),
                &params,
            )
            .await
    }

    /// Gets a file in a container.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(
        &self,
        container_id: impl AsRef<str>,
        file_id: impl AsRef<str>,
    ) -> Result<ContainerFile> {
        self.client
            .get_json(&format!(
                "/containers/{}/files/{}",
                urlencoding::encode(container_id.as_ref()),
                urlencoding::encode(file_id.as_ref())
            ))
            .await
    }

    /// Lists files in a container.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(
        &self,
        container_id: impl AsRef<str>,
        params: ContainerFileListParams,
    ) -> Result<CursorPage<ContainerFile>> {
        let path = format!(
            "/containers/{}/files",
            urlencoding::encode(container_id.as_ref())
        );
        self.client.get_cursor_page_query(&path, &params).await
    }

    /// Deletes a file in a container.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(
        &self,
        container_id: impl AsRef<str>,
        file_id: impl AsRef<str>,
    ) -> Result<ContainerFileDeleted> {
        self.client
            .delete_json(&format!(
                "/containers/{}/files/{}",
                urlencoding::encode(container_id.as_ref()),
                urlencoding::encode(file_id.as_ref())
            ))
            .await
    }

    /// Returns the container file content sub-service.
    #[must_use]
    pub fn content(&self) -> ContainerFileContentService {
        ContainerFileContentService::new(self.client.clone())
    }
}

/// Container file content service (nested under container files).
#[derive(Clone)]
pub struct ContainerFileContentService {
    client: Client,
}

impl ContainerFileContentService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Downloads the raw content of a file in a container.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(
        &self,
        container_id: impl AsRef<str>,
        file_id: impl AsRef<str>,
    ) -> Result<Vec<u8>> {
        self.client
            .get_bytes(&format!(
                "/containers/{}/files/{}/content",
                urlencoding::encode(container_id.as_ref()),
                urlencoding::encode(file_id.as_ref())
            ))
            .await
    }
}

// ---------------------------------------------------------------------------
// Request params
// ---------------------------------------------------------------------------

/// Network access policy for a container.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerNetworkPolicy {
    /// The network policy mode. One of `disabled` or `allowlist`.
    #[serde(rename = "type")]
    pub policy_type: String,
    /// Allowed outbound domains when `type` is `allowlist`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
}

/// A skill reference for a container, either by ID or inline.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerSkillRef {
    /// Skill ID reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Inline skill type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub skill_type: Option<String>,
}

/// Container create request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerCreateParams {
    /// Name of the container.
    pub name: String,
    /// Expiration configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<ContainerExpiresAfter>,
    /// IDs of files to copy into the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    /// Memory limit for the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit: Option<String>,
    /// Network access policy for the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_policy: Option<ContainerNetworkPolicy>,
    /// Optional list of skills referenced by id or inline data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<ContainerSkillRef>>,
}

/// Container expiration configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerExpiresAfter {
    /// Reference point for expiration (e.g. "last_active_at").
    pub anchor: String,
    /// Minutes after anchor before the container expires.
    pub minutes: i64,
}

/// Parameters for listing containers.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ContainerListParams {
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Maximum number of items (1-100, default 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Filter by container name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Sort order: "asc" or "desc".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
}

impl ContainerListParams {
    /// Serializes non-None fields into a URL query string.
    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref after) = self.after {
            parts.push(format!("after={}", urlencoding::encode(after)));
        }
        if let Some(limit) = self.limit {
            parts.push(format!("limit={limit}"));
        }
        if let Some(ref name) = self.name {
            parts.push(format!("name={}", urlencoding::encode(name)));
        }
        if let Some(ref order) = self.order {
            parts.push(format!("order={}", urlencoding::encode(order)));
        }
        parts.join("&")
    }
}

/// Container file create request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerFileCreateParams {
    /// ID of an existing file to add to the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}

/// Parameters for listing container files.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ContainerFileListParams {
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Maximum number of items (1-100, default 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Sort order: "asc" or "desc".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
}

impl ContainerFileListParams {
    /// Serializes non-None fields into a URL query string.
    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref after) = self.after {
            parts.push(format!("after={}", urlencoding::encode(after)));
        }
        if let Some(limit) = self.limit {
            parts.push(format!("limit={limit}"));
        }
        if let Some(ref order) = self.order {
            parts.push(format!("order={}", urlencoding::encode(order)));
        }
        parts.join("&")
    }
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Container object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Container {
    /// Unique container ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Container name.
    pub name: String,
    /// Container status (e.g. "active", "deleted").
    pub status: String,
    /// Unix timestamp when created.
    pub created_at: i64,
    /// Expiration configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<ContainerExpiresAfter>,
    /// Unix timestamp when last active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_active_at: Option<i64>,
    /// Memory limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit: Option<String>,
    /// Network access policy for the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_policy: Option<ContainerNetworkPolicy>,
}

/// Container delete response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerDeleted {
    /// Deleted container ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Deletion flag.
    pub deleted: bool,
}

/// Container file object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerFile {
    /// Unique file ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Size of the file in bytes.
    pub bytes: i64,
    /// Parent container ID.
    pub container_id: String,
    /// Unix timestamp when created.
    pub created_at: i64,
    /// File path in the container.
    pub path: String,
    /// Source of the file (e.g. "user", "assistant").
    pub source: String,
}

/// Container file delete response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerFileDeleted {
    /// Deleted file ID.
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
    fn container_create_params_omit_optional_fields() {
        let params = ContainerCreateParams {
            name: "my-container".to_owned(),
            expires_after: None,
            file_ids: None,
            memory_limit: None,
            network_policy: None,
            skills: None,
        };
        let value = serde_json::to_value(params).expect("serialize container create params");
        assert_eq!(value["name"], "my-container");
        assert!(value.get("expires_after").is_none());
        assert!(value.get("file_ids").is_none());
        assert!(value.get("memory_limit").is_none());
        assert!(value.get("network_policy").is_none());
        assert!(value.get("skills").is_none());
    }

    #[test]
    fn container_create_params_with_all_fields() {
        let params = ContainerCreateParams {
            name: "my-container".to_owned(),
            expires_after: Some(ContainerExpiresAfter {
                anchor: "last_active_at".to_owned(),
                minutes: 60,
            }),
            file_ids: Some(vec!["file_1".to_owned()]),
            memory_limit: Some("4g".to_owned()),
            network_policy: Some(ContainerNetworkPolicy {
                policy_type: "allowlist".to_owned(),
                allowed_domains: Some(vec!["example.com".to_owned()]),
            }),
            skills: Some(vec![ContainerSkillRef {
                id: Some("skill_abc".to_owned()),
                skill_type: None,
            }]),
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["name"], "my-container");
        assert_eq!(value["expires_after"]["anchor"], "last_active_at");
        assert_eq!(value["expires_after"]["minutes"], 60);
        assert_eq!(value["file_ids"][0], "file_1");
        assert_eq!(value["memory_limit"], "4g");
        assert_eq!(value["network_policy"]["type"], "allowlist");
        assert_eq!(value["network_policy"]["allowed_domains"][0], "example.com");
        assert_eq!(value["skills"][0]["id"], "skill_abc");
    }

    #[test]
    fn container_network_policy_disabled_round_trip() {
        let policy = ContainerNetworkPolicy {
            policy_type: "disabled".to_owned(),
            allowed_domains: None,
        };
        let json = serde_json::to_string(&policy).expect("serialize");
        let back: ContainerNetworkPolicy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.policy_type, "disabled");
        assert!(back.allowed_domains.is_none());
    }

    #[test]
    fn container_network_policy_allowlist_round_trip() {
        let policy = ContainerNetworkPolicy {
            policy_type: "allowlist".to_owned(),
            allowed_domains: Some(vec!["api.example.com".to_owned()]),
        };
        let json = serde_json::to_string(&policy).expect("serialize");
        let back: ContainerNetworkPolicy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.policy_type, "allowlist");
        assert_eq!(back.allowed_domains.as_ref().unwrap()[0], "api.example.com");
    }

    #[test]
    fn container_response_with_network_policy_deserializes() {
        let json = r#"{
            "id":"ctr_1",
            "object":"container",
            "name":"my-container",
            "status":"active",
            "created_at":1234567890,
            "network_policy":{"type":"allowlist","allowed_domains":["example.com"]}
        }"#;
        let container: Container = serde_json::from_str(json).expect("deserialize");
        let policy = container.network_policy.unwrap();
        assert_eq!(policy.policy_type, "allowlist");
        assert_eq!(policy.allowed_domains.unwrap()[0], "example.com");
    }

    #[test]
    fn container_deserializes() {
        let json = r#"{
            "id":"ctr_1",
            "object":"container",
            "name":"my-container",
            "status":"active",
            "created_at":1234567890,
            "memory_limit":"1g"
        }"#;

        let container: Container = serde_json::from_str(json).expect("deserialize container");
        assert_eq!(container.id, "ctr_1");
        assert_eq!(container.name, "my-container");
        assert_eq!(container.status, "active");
        assert_eq!(container.memory_limit.as_deref(), Some("1g"));
    }

    #[test]
    fn container_deleted_deserializes() {
        let json = r#"{
            "id":"ctr_1",
            "object":"container.deleted",
            "deleted":true
        }"#;

        let deleted: ContainerDeleted =
            serde_json::from_str(json).expect("deserialize container deleted");
        assert_eq!(deleted.id, "ctr_1");
        assert!(deleted.deleted);
    }

    #[test]
    fn container_file_deserializes() {
        let json = r#"{
            "id":"file_1",
            "object":"container.file",
            "bytes":1024,
            "container_id":"ctr_1",
            "created_at":1234567890,
            "path":"/data/input.txt",
            "source":"user"
        }"#;

        let file: ContainerFile = serde_json::from_str(json).expect("deserialize container file");
        assert_eq!(file.id, "file_1");
        assert_eq!(file.container_id, "ctr_1");
        assert_eq!(file.bytes, 1024);
        assert_eq!(file.path, "/data/input.txt");
        assert_eq!(file.source, "user");
    }

    #[test]
    fn container_file_deleted_deserializes() {
        let json = r#"{
            "id":"file_1",
            "object":"container.file.deleted",
            "deleted":true
        }"#;

        let deleted: ContainerFileDeleted =
            serde_json::from_str(json).expect("deserialize container file deleted");
        assert_eq!(deleted.id, "file_1");
        assert!(deleted.deleted);
    }

    #[test]
    fn container_list_params_query_string() {
        let params = ContainerListParams {
            after: Some("ctr_abc".to_owned()),
            limit: Some(10),
            name: Some("my-container".to_owned()),
            order: Some("asc".to_owned()),
        };
        let qs = params.to_query_string();
        assert!(qs.contains("after=ctr_abc"));
        assert!(qs.contains("limit=10"));
        assert!(qs.contains("name=my-container"));
        assert!(qs.contains("order=asc"));
    }

    #[test]
    fn container_list_params_empty_query_string() {
        let params = ContainerListParams::default();
        assert!(params.to_query_string().is_empty());
    }

    #[test]
    fn container_file_create_params_omit_optional() {
        let params = ContainerFileCreateParams { file_id: None };
        let value = serde_json::to_value(params).expect("serialize");
        assert!(value.get("file_id").is_none());
    }

    #[test]
    fn container_file_create_params_with_file_id() {
        let params = ContainerFileCreateParams {
            file_id: Some("file_abc".to_owned()),
        };
        let value = serde_json::to_value(&params).expect("serialize");
        assert_eq!(value["file_id"], "file_abc");
    }

    #[test]
    fn container_file_list_params_query_string() {
        let params = ContainerFileListParams {
            after: Some("file_abc".to_owned()),
            limit: Some(5),
            order: Some("desc".to_owned()),
        };
        let qs = params.to_query_string();
        assert!(qs.contains("after=file_abc"));
        assert!(qs.contains("limit=5"));
        assert!(qs.contains("order=desc"));
    }

    #[test]
    fn container_expires_after_round_trips() {
        let ea = ContainerExpiresAfter {
            anchor: "last_active_at".to_owned(),
            minutes: 120,
        };
        let json = serde_json::to_string(&ea).expect("serialize");
        let back: ContainerExpiresAfter = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.anchor, "last_active_at");
        assert_eq!(back.minutes, 120);
    }
}
