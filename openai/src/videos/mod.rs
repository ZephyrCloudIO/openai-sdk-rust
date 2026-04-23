//! Video generation APIs.

use reqwest::multipart::{Form, Part};

use crate::{pagination::ConversationCursorPage, Client, Result};

/// Video service.
#[derive(Clone)]
pub struct VideoService {
    client: Client,
}

impl VideoService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a new video generation job from a prompt and optional reference assets.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create(&self, params: VideoCreateParams) -> Result<Video> {
        let prompt = params.prompt;
        let model = params.model;
        let seconds = params.seconds;
        let size = params.size;
        let input_reference = params.input_reference;

        self.client
            .post_multipart_json("/videos", move || {
                let mut form = Form::new().text("prompt", prompt.clone());

                if let Some(ref m) = model {
                    form = form.text("model", m.clone());
                }
                if let Some(ref s) = seconds {
                    form = form.text("seconds", s.clone());
                }
                if let Some(ref sz) = size {
                    form = form.text("size", sz.clone());
                }
                if let Some(ref input_ref) = input_reference {
                    match input_ref {
                        VideoInputReference::File(file_data) => {
                            let part = Part::bytes(file_data.bytes.clone())
                                .file_name(file_data.filename.clone());
                            form = form.part("input_reference", part);
                        }
                        VideoInputReference::Image(image_ref) => {
                            if let Some(ref file_id) = image_ref.file_id {
                                form = form.text("input_reference[file_id]", file_id.clone());
                            }
                            if let Some(ref image_url) = image_ref.image_url {
                                form = form.text("input_reference[image_url]", image_url.clone());
                            }
                        }
                    }
                }
                form
            })
            .await
    }

    /// Creates a video generation job and polls until it reaches a terminal status.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures during creation or polling.
    pub async fn create_and_poll(
        &self,
        params: VideoCreateParams,
        interval: std::time::Duration,
    ) -> Result<Video> {
        let video = self.create(params).await?;
        self.poll_status(&video.id, interval).await
    }

    /// Alias for [`VideoService::create_and_poll`].
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures during creation or polling.
    pub async fn new_and_poll(
        &self,
        params: VideoCreateParams,
        interval: std::time::Duration,
    ) -> Result<Video> {
        self.create_and_poll(params, interval).await
    }

    /// Fetches the latest metadata for a generated video.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get(&self, video_id: impl AsRef<str>) -> Result<Video> {
        self.client
            .get_json(&format!(
                "/videos/{}",
                urlencoding::encode(video_id.as_ref())
            ))
            .await
    }

    /// Polls until a video reaches a terminal status (`completed` or `failed`).
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures during polling.
    pub async fn poll_status(
        &self,
        video_id: impl AsRef<str>,
        interval: std::time::Duration,
    ) -> Result<Video> {
        let video_id = video_id.as_ref();
        loop {
            let video = self.get(video_id).await?;
            match video.status {
                VideoStatus::Queued | VideoStatus::InProgress => {
                    tokio::time::sleep(interval).await;
                }
                VideoStatus::Completed | VideoStatus::Failed => return Ok(video),
            }
        }
    }

    /// Lists recently generated videos for the current project.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn list(&self, params: VideoListParams) -> Result<ConversationCursorPage<Video>> {
        self.client
            .get_conversation_cursor_page_query("/videos", &params)
            .await
    }

    /// Permanently deletes a completed or failed video and its stored assets.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn delete(&self, video_id: impl AsRef<str>) -> Result<VideoDeleteResponse> {
        self.client
            .delete_json(&format!(
                "/videos/{}",
                urlencoding::encode(video_id.as_ref())
            ))
            .await
    }

    /// Creates a character from an uploaded video.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn create_character(
        &self,
        params: VideoNewCharacterParams,
    ) -> Result<VideoCharacter> {
        let name = params.name;
        let video = params.video;

        self.client
            .post_multipart_json("/videos/characters", move || {
                let part = Part::bytes(video.bytes.clone()).file_name(video.filename.clone());
                Form::new().text("name", name.clone()).part("video", part)
            })
            .await
    }

    /// Fetches a character by ID.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn get_character(&self, character_id: impl AsRef<str>) -> Result<VideoCharacter> {
        self.client
            .get_json(&format!(
                "/videos/characters/{}",
                urlencoding::encode(character_id.as_ref())
            ))
            .await
    }

    /// Downloads the generated video bytes or a derived preview asset.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn download_content(
        &self,
        video_id: impl AsRef<str>,
        params: VideoDownloadContentParams,
    ) -> Result<Vec<u8>> {
        let mut path = format!("/videos/{}/content", urlencoding::encode(video_id.as_ref()));
        if let Some(ref variant) = params.variant {
            path.push_str(&format!("?variant={variant}"));
        }
        self.client.get_bytes(&path).await
    }

    /// Creates a new video generation job by editing a source video.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn edit(&self, params: VideoEditParams) -> Result<Video> {
        let prompt = params.prompt;
        let video = params.video;

        self.client
            .post_multipart_json("/videos/edits", move || {
                let mut form = Form::new().text("prompt", prompt.clone());
                match &video {
                    VideoReference::File(file_data) => {
                        let part = Part::bytes(file_data.bytes.clone())
                            .file_name(file_data.filename.clone());
                        form = form.part("video", part);
                    }
                    VideoReference::Id(id) => {
                        form = form.text("video[id]", id.clone());
                    }
                }
                form
            })
            .await
    }

    /// Creates an extension of a completed video.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn extend(&self, params: VideoExtendParams) -> Result<Video> {
        let prompt = params.prompt;
        let seconds = params.seconds;
        let video = params.video;

        self.client
            .post_multipart_json("/videos/extensions", move || {
                let mut form = Form::new()
                    .text("prompt", prompt.clone())
                    .text("seconds", seconds.clone());
                match &video {
                    VideoReference::File(file_data) => {
                        let part = Part::bytes(file_data.bytes.clone())
                            .file_name(file_data.filename.clone());
                        form = form.part("video", part);
                    }
                    VideoReference::Id(id) => {
                        form = form.text("video[id]", id.clone());
                    }
                }
                form
            })
            .await
    }

    /// Creates a remix of a completed video using a refreshed prompt.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn remix(
        &self,
        video_id: impl AsRef<str>,
        params: VideoRemixParams,
    ) -> Result<Video> {
        self.client
            .post_json(
                &format!("/videos/{}/remix", urlencoding::encode(video_id.as_ref())),
                &params,
            )
            .await
    }
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

/// File data for video upload.
#[derive(Debug, Clone)]
pub struct VideoFileUpload {
    /// File bytes.
    pub bytes: Vec<u8>,
    /// File name presented to API.
    pub filename: String,
}

impl VideoFileUpload {
    /// Creates a file upload from bytes.
    #[must_use]
    pub fn from_bytes(bytes: impl Into<Vec<u8>>, filename: impl Into<String>) -> Self {
        Self {
            bytes: bytes.into(),
            filename: filename.into(),
        }
    }
}

/// Image input reference for video generation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageInputReference {
    /// Optional file ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// A fully qualified URL or base64-encoded data URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

/// Input reference for video creation -- either a file or an image reference.
#[derive(Debug, Clone)]
pub enum VideoInputReference {
    /// Upload a file as input reference.
    File(VideoFileUpload),
    /// Reference an image by file ID or URL.
    Image(ImageInputReference),
}

/// Reference to a video -- either a file upload or a video ID.
#[derive(Debug, Clone)]
pub enum VideoReference {
    /// Upload a video file.
    File(VideoFileUpload),
    /// Reference a completed video by ID.
    Id(String),
}

/// Video create request.
#[derive(Debug, Clone)]
pub struct VideoCreateParams {
    /// Text prompt that describes the video to generate.
    pub prompt: String,
    /// Optional reference asset upload or reference object.
    pub input_reference: Option<VideoInputReference>,
    /// The video generation model to use.
    pub model: Option<String>,
    /// Clip duration in seconds (allowed values: "4", "8", "12").
    pub seconds: Option<String>,
    /// Output resolution formatted as width x height.
    pub size: Option<String>,
}

/// Video list query parameters.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VideoListParams {
    /// Identifier for the last item from the previous pagination request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Number of items to retrieve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Sort order of results by timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
}

/// Character creation request.
#[derive(Debug, Clone)]
pub struct VideoNewCharacterParams {
    /// Display name for this API character.
    pub name: String,
    /// Video file used to create a character.
    pub video: VideoFileUpload,
}

/// Download content parameters.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VideoDownloadContentParams {
    /// Which downloadable asset to return (video, thumbnail, spritesheet).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

/// Video edit request.
#[derive(Debug, Clone)]
pub struct VideoEditParams {
    /// Text prompt that describes how to edit the source video.
    pub prompt: String,
    /// Reference to the completed video to edit.
    pub video: VideoReference,
}

/// Video extend request.
#[derive(Debug, Clone)]
pub struct VideoExtendParams {
    /// Updated text prompt that directs the extension generation.
    pub prompt: String,
    /// Length of the newly generated extension segment in seconds.
    pub seconds: String,
    /// Reference to the completed video to extend.
    pub video: VideoReference,
}

/// Video remix request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VideoRemixParams {
    /// Updated text prompt that directs the remix generation.
    pub prompt: String,
}

/// Video model constants.
pub mod video_model {
    /// Sora 2 model.
    pub const SORA_2: &str = "sora-2";
    /// Sora 2 Pro model.
    pub const SORA_2_PRO: &str = "sora-2-pro";
    /// Sora 2 2025-10-06 model.
    pub const SORA_2_2025_10_06: &str = "sora-2-2025-10-06";
    /// Sora 2 Pro 2025-10-06 model.
    pub const SORA_2_PRO_2025_10_06: &str = "sora-2-pro-2025-10-06";
    /// Sora 2 2025-12-08 model.
    pub const SORA_2_2025_12_08: &str = "sora-2-2025-12-08";
}

/// Video seconds constants.
pub mod video_seconds {
    /// 4-second clip.
    pub const FOUR: &str = "4";
    /// 8-second clip.
    pub const EIGHT: &str = "8";
    /// 12-second clip.
    pub const TWELVE: &str = "12";
}

/// Video size constants.
pub mod video_size {
    /// 720x1280 resolution.
    pub const SIZE_720X1280: &str = "720x1280";
    /// 1280x720 resolution.
    pub const SIZE_1280X720: &str = "1280x720";
    /// 1024x1792 resolution.
    pub const SIZE_1024X1792: &str = "1024x1792";
    /// 1792x1024 resolution.
    pub const SIZE_1792X1024: &str = "1792x1024";
}

/// Video download content variant constants.
pub mod download_variant {
    /// Video variant.
    pub const VIDEO: &str = "video";
    /// Thumbnail variant.
    pub const THUMBNAIL: &str = "thumbnail";
    /// Spritesheet variant.
    pub const SPRITESHEET: &str = "spritesheet";
}

/// Video status.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoStatus {
    /// Job is queued.
    Queued,
    /// Job is in progress.
    InProgress,
    /// Job is completed.
    Completed,
    /// Job has failed.
    Failed,
}

/// Error payload for a failed video generation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VideoError {
    /// A machine-readable error code.
    #[serde(default)]
    pub code: String,
    /// A human-readable description of the error.
    #[serde(default)]
    pub message: String,
}

/// Video object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Video {
    /// Unique identifier for the video job.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Unix timestamp (seconds) for when the job was created.
    pub created_at: i64,
    /// Unix timestamp (seconds) for when the job completed, if finished.
    #[serde(default)]
    pub completed_at: i64,
    /// Error payload that explains why generation failed, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<VideoError>,
    /// Unix timestamp (seconds) for when the downloadable assets expire.
    #[serde(default)]
    pub expires_at: i64,
    /// The video generation model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Approximate completion percentage.
    #[serde(default)]
    pub progress: i64,
    /// The prompt used to generate the video.
    #[serde(default)]
    pub prompt: String,
    /// Identifier of the source video if this is a remix.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remixed_from_video_id: Option<String>,
    /// Duration of the generated clip in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds: Option<String>,
    /// The resolution of the generated video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Current lifecycle status of the video job.
    pub status: VideoStatus,
}

/// Video delete response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VideoDeleteResponse {
    /// Identifier of the deleted video.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Indicates that the video resource was deleted.
    pub deleted: bool,
}

/// Video character response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VideoCharacter {
    /// Identifier for the character.
    pub id: String,
    /// Unix timestamp (in seconds) when the character was created.
    pub created_at: i64,
    /// Display name for the character.
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::{
        ImageInputReference, Video, VideoCharacter, VideoDeleteResponse,
        VideoDownloadContentParams, VideoError, VideoFileUpload, VideoRemixParams, VideoStatus,
    };

    #[test]
    fn video_deserializes_all_fields() {
        let json = r#"{
            "id":"video_1",
            "object":"video",
            "created_at":1700000000,
            "completed_at":1700000100,
            "error":null,
            "expires_at":1700100000,
            "model":"sora-2",
            "progress":100,
            "prompt":"A cat in space",
            "remixed_from_video_id":null,
            "seconds":"4",
            "size":"720x1280",
            "status":"completed"
        }"#;

        let video: Video = serde_json::from_str(json).expect("deserialize video");
        assert_eq!(video.id, "video_1");
        assert_eq!(video.object, "video");
        assert_eq!(video.created_at, 1_700_000_000);
        assert_eq!(video.completed_at, 1_700_000_100);
        assert_eq!(video.model.as_deref(), Some("sora-2"));
        assert_eq!(video.progress, 100);
        assert_eq!(video.prompt, "A cat in space");
        assert_eq!(video.seconds.as_deref(), Some("4"));
        assert_eq!(video.size.as_deref(), Some("720x1280"));
        assert_eq!(video.status, VideoStatus::Completed);
    }

    #[test]
    fn video_status_deserializes_snake_case() {
        let json = r#""in_progress""#;
        let status: VideoStatus = serde_json::from_str(json).expect("deserialize status");
        assert_eq!(status, VideoStatus::InProgress);
    }

    #[test]
    fn video_delete_response_deserializes() {
        let json = r#"{
            "id":"video_1",
            "object":"video.deleted",
            "deleted":true
        }"#;

        let resp: VideoDeleteResponse =
            serde_json::from_str(json).expect("deserialize delete response");
        assert_eq!(resp.id, "video_1");
        assert!(resp.deleted);
    }

    #[test]
    fn video_character_deserializes() {
        let json = r#"{
            "id":"char_1",
            "created_at":1700000000,
            "name":"Hero"
        }"#;

        let character: VideoCharacter = serde_json::from_str(json).expect("deserialize character");
        assert_eq!(character.id, "char_1");
        assert_eq!(character.name, "Hero");
    }

    #[test]
    fn video_error_deserializes() {
        let json = r#"{
            "code":"content_policy",
            "message":"Prompt violates content policy"
        }"#;

        let err: VideoError = serde_json::from_str(json).expect("deserialize video error");
        assert_eq!(err.code, "content_policy");
        assert_eq!(err.message, "Prompt violates content policy");
    }

    #[test]
    fn video_remix_params_serializes() {
        let params = VideoRemixParams {
            prompt: "Make it blue".to_owned(),
        };

        let value = serde_json::to_value(params).expect("serialize remix params");
        assert_eq!(
            value.get("prompt"),
            Some(&serde_json::Value::String("Make it blue".to_owned()))
        );
    }

    #[test]
    fn video_download_content_params_omit_optional() {
        let params = VideoDownloadContentParams { variant: None };
        let value = serde_json::to_value(params).expect("serialize download params");
        assert!(value.get("variant").is_none());
    }

    #[test]
    fn image_input_reference_omit_optional_fields() {
        let input_ref = ImageInputReference {
            file_id: Some("file_123".to_owned()),
            image_url: None,
        };

        let value = serde_json::to_value(input_ref).expect("serialize image ref");
        assert_eq!(
            value.get("file_id"),
            Some(&serde_json::Value::String("file_123".to_owned()))
        );
        assert!(value.get("image_url").is_none());
    }

    #[test]
    fn video_file_upload_from_bytes() {
        let upload = VideoFileUpload::from_bytes(vec![1_u8, 2, 3], "video.mp4");
        assert_eq!(upload.filename, "video.mp4");
        assert_eq!(upload.bytes, vec![1_u8, 2, 3]);
    }
}
