//! Skill management APIs.

use reqwest::multipart::{Form, Part};

use crate::{pagination::CursorPage, Client, Result};

/// Skill service.
#[derive(Clone)]
pub struct SkillService {
    client: Client,
}

impl SkillService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a new skill.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: SkillCreateParams) -> Result<Skill> {
        let files = params.files;
        self.client
            .post_multipart_json("/skills", move || {
                let mut form = Form::new();
                for (i, file_data) in files.iter().enumerate() {
                    let part =
                        Part::bytes(file_data.bytes.clone()).file_name(file_data.filename.clone());
                    form = form.part(format!("files[{i}]"), part);
                }
                form
            })
            .await
    }

    /// Gets a skill by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, skill_id: impl AsRef<str>) -> Result<Skill> {
        self.client
            .get_json(&format!(
                "/skills/{}",
                urlencoding::encode(skill_id.as_ref())
            ))
            .await
    }

    /// Updates the default version pointer for a skill.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn update(
        &self,
        skill_id: impl AsRef<str>,
        params: SkillUpdateParams,
    ) -> Result<Skill> {
        self.client
            .post_json(
                &format!("/skills/{}", urlencoding::encode(skill_id.as_ref())),
                &params,
            )
            .await
    }

    /// Lists all skills for the current project.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(&self) -> Result<CursorPage<Skill>> {
        self.client.get_cursor_page("/skills").await
    }

    /// Deletes a skill by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(&self, skill_id: impl AsRef<str>) -> Result<DeletedSkill> {
        self.client
            .delete_json(&format!(
                "/skills/{}",
                urlencoding::encode(skill_id.as_ref())
            ))
            .await
    }

    /// Returns the skill content sub-service.
    #[must_use]
    pub fn content(&self) -> SkillContentService {
        SkillContentService::new(self.client.clone())
    }

    /// Returns the skill version sub-service.
    #[must_use]
    pub fn versions(&self) -> SkillVersionService {
        SkillVersionService::new(self.client.clone())
    }
}

/// Skill content sub-service for downloading skill bundles.
#[derive(Clone)]
pub struct SkillContentService {
    client: Client,
}

impl SkillContentService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Downloads a skill zip bundle by its ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, skill_id: impl AsRef<str>) -> Result<Vec<u8>> {
        self.client
            .get_bytes(&format!(
                "/skills/{}/content",
                urlencoding::encode(skill_id.as_ref())
            ))
            .await
    }
}

/// Skill version sub-service.
#[derive(Clone)]
pub struct SkillVersionService {
    client: Client,
}

impl SkillVersionService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a new immutable skill version.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(
        &self,
        skill_id: impl AsRef<str>,
        params: SkillVersionCreateParams,
    ) -> Result<SkillVersion> {
        let skill_id = skill_id.as_ref().to_owned();
        let set_default = params.default;
        let files = params.files;
        self.client
            .post_multipart_json(
                &format!("/skills/{}/versions", urlencoding::encode(&skill_id)),
                move || {
                    let mut form = Form::new();
                    if let Some(d) = set_default {
                        form = form.text("default", d.to_string());
                    }
                    for (i, file_data) in files.iter().enumerate() {
                        let part = Part::bytes(file_data.bytes.clone())
                            .file_name(file_data.filename.clone());
                        form = form.part(format!("files[{i}]"), part);
                    }
                    form
                },
            )
            .await
    }

    /// Gets a specific skill version.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(
        &self,
        skill_id: impl AsRef<str>,
        version: impl AsRef<str>,
    ) -> Result<SkillVersion> {
        self.client
            .get_json(&format!(
                "/skills/{}/versions/{}",
                urlencoding::encode(skill_id.as_ref()),
                urlencoding::encode(version.as_ref())
            ))
            .await
    }

    /// Lists skill versions for a skill.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(&self, skill_id: impl AsRef<str>) -> Result<CursorPage<SkillVersion>> {
        self.client
            .get_cursor_page(&format!(
                "/skills/{}/versions",
                urlencoding::encode(skill_id.as_ref())
            ))
            .await
    }

    /// Deletes a skill version.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(
        &self,
        skill_id: impl AsRef<str>,
        version: impl AsRef<str>,
    ) -> Result<DeletedSkillVersion> {
        self.client
            .delete_json(&format!(
                "/skills/{}/versions/{}",
                urlencoding::encode(skill_id.as_ref()),
                urlencoding::encode(version.as_ref())
            ))
            .await
    }

    /// Returns the skill version content sub-service.
    #[must_use]
    pub fn content(&self) -> SkillVersionContentService {
        SkillVersionContentService::new(self.client.clone())
    }
}

/// Skill version content sub-service for downloading version bundles.
#[derive(Clone)]
pub struct SkillVersionContentService {
    client: Client,
}

impl SkillVersionContentService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Downloads a skill version zip bundle.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(
        &self,
        skill_id: impl AsRef<str>,
        version: impl AsRef<str>,
    ) -> Result<Vec<u8>> {
        self.client
            .get_bytes(&format!(
                "/skills/{}/versions/{}/content",
                urlencoding::encode(skill_id.as_ref()),
                urlencoding::encode(version.as_ref())
            ))
            .await
    }
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

/// File data for skill upload.
#[derive(Debug, Clone)]
pub struct SkillFileUpload {
    /// File bytes.
    pub bytes: Vec<u8>,
    /// File name presented to API.
    pub filename: String,
}

impl SkillFileUpload {
    /// Creates a file upload from bytes.
    #[must_use]
    pub fn from_bytes(bytes: impl Into<Vec<u8>>, filename: impl Into<String>) -> Self {
        Self {
            bytes: bytes.into(),
            filename: filename.into(),
        }
    }
}

/// Skill create request.
#[derive(Debug, Clone)]
pub struct SkillCreateParams {
    /// Skill files to upload (directory upload) or a single zip file.
    pub files: Vec<SkillFileUpload>,
}

/// Skill update request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillUpdateParams {
    /// The skill version number to set as default.
    pub default_version: String,
}

/// Skill version create request.
#[derive(Debug, Clone)]
pub struct SkillVersionCreateParams {
    /// Whether to set this version as the default.
    pub default: Option<bool>,
    /// Skill files to upload (directory upload) or a single zip file.
    pub files: Vec<SkillFileUpload>,
}

/// Sort order for list operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

/// Skill object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Skill {
    /// Unique identifier for the skill.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Unix timestamp (seconds) for when the skill was created.
    pub created_at: i64,
    /// Default version for the skill.
    pub default_version: String,
    /// Description of the skill.
    pub description: String,
    /// Latest version for the skill.
    pub latest_version: String,
    /// Name of the skill.
    pub name: String,
}

/// Skill delete response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeletedSkill {
    /// Skill ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Deletion flag.
    pub deleted: bool,
}

/// Skill version object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillVersion {
    /// Unique identifier for the skill version.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Unix timestamp (seconds) for when the version was created.
    pub created_at: i64,
    /// Description of the skill version.
    pub description: String,
    /// Name of the skill version.
    pub name: String,
    /// Identifier of the skill for this version.
    pub skill_id: String,
    /// Version number for this skill.
    pub version: String,
}

/// Skill version delete response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeletedSkillVersion {
    /// Skill version ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Deletion flag.
    pub deleted: bool,
    /// The deleted skill version.
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::{
        DeletedSkill, DeletedSkillVersion, Skill, SkillFileUpload, SkillUpdateParams, SkillVersion,
        SortOrder,
    };

    #[test]
    fn skill_deserializes_all_fields() {
        let json = r#"{
            "id":"skill_1",
            "object":"skill",
            "created_at":1700000000,
            "default_version":"v1",
            "description":"A test skill",
            "latest_version":"v2",
            "name":"my-skill"
        }"#;

        let skill: Skill = serde_json::from_str(json).expect("deserialize skill");
        assert_eq!(skill.id, "skill_1");
        assert_eq!(skill.object, "skill");
        assert_eq!(skill.created_at, 1_700_000_000);
        assert_eq!(skill.default_version, "v1");
        assert_eq!(skill.description, "A test skill");
        assert_eq!(skill.latest_version, "v2");
        assert_eq!(skill.name, "my-skill");
    }

    #[test]
    fn deleted_skill_deserializes_flag() {
        let json = r#"{
            "id":"skill_1",
            "object":"skill.deleted",
            "deleted":true
        }"#;

        let deleted: DeletedSkill = serde_json::from_str(json).expect("deserialize deleted skill");
        assert_eq!(deleted.id, "skill_1");
        assert!(deleted.deleted);
    }

    #[test]
    fn skill_version_deserializes_all_fields() {
        let json = r#"{
            "id":"skillver_1",
            "object":"skill.version",
            "created_at":1700000000,
            "description":"First version",
            "name":"my-skill",
            "skill_id":"skill_1",
            "version":"v1"
        }"#;

        let ver: SkillVersion = serde_json::from_str(json).expect("deserialize skill version");
        assert_eq!(ver.id, "skillver_1");
        assert_eq!(ver.skill_id, "skill_1");
        assert_eq!(ver.version, "v1");
    }

    #[test]
    fn deleted_skill_version_deserializes() {
        let json = r#"{
            "id":"skill_1",
            "object":"skill.version.deleted",
            "deleted":true,
            "version":"v1"
        }"#;

        let deleted: DeletedSkillVersion =
            serde_json::from_str(json).expect("deserialize deleted skill version");
        assert_eq!(deleted.id, "skill_1");
        assert!(deleted.deleted);
        assert_eq!(deleted.version, "v1");
    }

    #[test]
    fn skill_update_params_serializes() {
        let params = SkillUpdateParams {
            default_version: "v2".to_owned(),
        };

        let value = serde_json::to_value(params).expect("serialize update params");
        assert_eq!(
            value.get("default_version"),
            Some(&serde_json::Value::String("v2".to_owned()))
        );
    }

    #[test]
    fn sort_order_serializes_lowercase() {
        let asc = serde_json::to_string(&SortOrder::Asc).expect("serialize asc");
        assert_eq!(asc, "\"asc\"");

        let desc = serde_json::to_string(&SortOrder::Desc).expect("serialize desc");
        assert_eq!(desc, "\"desc\"");
    }

    #[test]
    fn skill_file_upload_from_bytes() {
        let upload = SkillFileUpload::from_bytes(vec![1_u8, 2, 3], "skill.zip");
        assert_eq!(upload.filename, "skill.zip");
        assert_eq!(upload.bytes, vec![1_u8, 2, 3]);
    }
}
