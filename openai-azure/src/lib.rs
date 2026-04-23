//! Azure OpenAI configuration helpers.
//!
//! Provides builder functions that produce an `openai::ClientConfig` preconfigured
//! for Azure OpenAI Service. Azure uses deployment names instead of model names and
//! requires an `api-version` query parameter on every request.
//!
//! # Usage with API key
//!
//! ```rust,ignore
//! use openai_azure;
//!
//! let config = openai_azure::with_api_key(
//!     "https://my-resource.openai.azure.com",
//!     "2024-10-21",
//!     "my-api-key",
//! );
//! ```
//!
//! # Usage with token credential (requires `azure` feature)
//!
//! ```rust,ignore
//! use openai_azure;
//! use std::sync::Arc;
//!
//! let credential = /* azure_identity credential */;
//! let config = openai_azure::with_token_credential(
//!     "https://my-resource.openai.azure.com",
//!     "2024-10-21",
//!     credential,
//! ).await.unwrap();
//! ```

#[cfg(feature = "config")]
use openai::ClientConfig;

#[cfg(feature = "azure")]
use std::sync::Arc;

#[cfg(feature = "azure")]
use azure_core::credentials::TokenCredential;

/// Header name for Azure API key authentication.
#[cfg(feature = "config")]
const API_KEY_HEADER: &str = "api-key";

/// Query parameter name for the Azure API version.
#[cfg(feature = "config")]
const API_VERSION_QUERY_PARAM: &str = "api-version";

/// Default scope used when requesting Azure AD tokens for cognitive services.
#[cfg(feature = "azure")]
const COGNITIVE_SERVICES_SCOPE: &str = "https://cognitiveservices.azure.com/.default";

/// Routes that carry a JSON body containing a `model` field that needs to be
/// mapped to an Azure deployment name.
#[cfg(feature = "config")]
const JSON_DEPLOYMENT_ROUTES: &[&str] = &[
    "/completions",
    "/chat/completions",
    "/embeddings",
    "/audio/speech",
    "/images/generations",
];

/// Routes that carry a multipart body with a `model` form field.
#[cfg(feature = "config")]
const MULTIPART_DEPLOYMENT_ROUTES: &[&str] = &[
    "/audio/transcriptions",
    "/audio/translations",
    "/images/edits",
];

/// Builds an `openai::ClientConfig` with Azure endpoint and API version defaults.
///
/// The base URL is set to `{endpoint}/openai` and the `api-version` query
/// parameter is injected on every request.
#[cfg(feature = "config")]
pub fn with_endpoint(endpoint: &str, api_version: &str) -> ClientConfig {
    let base_url = format!("{}/openai", endpoint.trim_end_matches('/'));

    ClientConfig::default()
        .with_base_url(base_url)
        .with_query_param(API_VERSION_QUERY_PARAM, api_version)
}

/// Builds an `openai::ClientConfig` with endpoint, API version, and API key.
///
/// Azure OpenAI uses the `api-key` header instead of Bearer auth.
#[cfg(feature = "config")]
pub fn with_api_key(endpoint: &str, api_version: &str, api_key: impl Into<String>) -> ClientConfig {
    with_endpoint(endpoint, api_version).with_header(API_KEY_HEADER, api_key)
}

/// Builds an `openai::ClientConfig` with endpoint, API version, and bearer token auth.
///
/// Acquires an access token from the provided credential and sets it as a Bearer
/// token in the Authorization header.
#[cfg(feature = "azure")]
pub async fn with_token_credential(
    endpoint: &str,
    api_version: &str,
    credential: Arc<dyn TokenCredential>,
) -> azure_core::Result<ClientConfig> {
    let token = credential.get_token(&[COGNITIVE_SERVICES_SCOPE]).await?;
    Ok(with_endpoint(endpoint, api_version)
        .with_header("Authorization", format!("Bearer {}", token.token.secret())))
}

/// Given a request path (e.g. `/chat/completions`) and a deployment name,
/// returns the Azure-style path with the deployment segment injected.
///
/// Azure routes take the form:
/// `/openai/deployments/{deployment}/chat/completions`
///
/// Routes that do not need a deployment (e.g. `/models`, `/files`) get
/// prefixed with `/openai/` only.
#[cfg(feature = "config")]
#[must_use]
pub fn deployment_path(route: &str, deployment: &str) -> String {
    if is_json_deployment_route(route) || is_multipart_deployment_route(route) {
        let escaped = urlencoding::encode(deployment);
        format!("/openai/deployments/{escaped}{route}")
    } else {
        format!("/openai{route}")
    }
}

/// Returns true if the route carries a JSON body with a model field that needs
/// to be mapped to a deployment name.
#[cfg(feature = "config")]
fn is_json_deployment_route(route: &str) -> bool {
    JSON_DEPLOYMENT_ROUTES.contains(&route)
}

/// Returns true if the route carries a multipart body with a model form field.
#[cfg(feature = "config")]
fn is_multipart_deployment_route(route: &str) -> bool {
    MULTIPART_DEPLOYMENT_ROUTES.contains(&route)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
#[cfg(feature = "config")]
mod tests {
    use super::*;

    #[test]
    fn with_endpoint_builds_correct_base_url() {
        let config = with_endpoint("https://my-resource.openai.azure.com", "2024-10-21");
        assert_eq!(
            config.base_url.as_deref(),
            Some("https://my-resource.openai.azure.com/openai")
        );
    }

    #[test]
    fn with_endpoint_strips_trailing_slash() {
        let config = with_endpoint("https://my-resource.openai.azure.com/", "2024-10-21");
        assert_eq!(
            config.base_url.as_deref(),
            Some("https://my-resource.openai.azure.com/openai")
        );
    }

    #[test]
    fn with_endpoint_injects_api_version_query_param() {
        let config = with_endpoint("https://my-resource.openai.azure.com", "2024-10-21");
        assert_eq!(
            config.query_params,
            vec![("api-version".to_owned(), "2024-10-21".to_owned())]
        );
    }

    #[test]
    fn with_api_key_sets_api_key_header() {
        let config = with_api_key(
            "https://my-resource.openai.azure.com",
            "2024-10-21",
            "my-secret-key",
        );
        let has_api_key_header = config
            .headers
            .iter()
            .any(|(k, v)| k == "api-key" && v == "my-secret-key");
        assert!(has_api_key_header, "api-key header should be present");
    }

    #[test]
    fn with_api_key_does_not_set_bearer_auth() {
        let config = with_api_key(
            "https://my-resource.openai.azure.com",
            "2024-10-21",
            "my-secret-key",
        );
        // api_key field is used for Bearer auth in the Client, and should NOT be set
        assert!(
            config.api_key.is_none(),
            "Bearer API key should not be set for Azure; use api-key header instead"
        );
    }

    #[test]
    fn deployment_path_json_routes() {
        assert_eq!(
            deployment_path("/chat/completions", "gpt-4"),
            "/openai/deployments/gpt-4/chat/completions"
        );
        assert_eq!(
            deployment_path("/completions", "gpt-4"),
            "/openai/deployments/gpt-4/completions"
        );
        assert_eq!(
            deployment_path("/embeddings", "text-embedding-ada-002"),
            "/openai/deployments/text-embedding-ada-002/embeddings"
        );
        assert_eq!(
            deployment_path("/audio/speech", "tts-1"),
            "/openai/deployments/tts-1/audio/speech"
        );
        assert_eq!(
            deployment_path("/images/generations", "dall-e-3"),
            "/openai/deployments/dall-e-3/images/generations"
        );
    }

    #[test]
    fn deployment_path_multipart_routes() {
        assert_eq!(
            deployment_path("/audio/transcriptions", "whisper-1"),
            "/openai/deployments/whisper-1/audio/transcriptions"
        );
        assert_eq!(
            deployment_path("/audio/translations", "whisper-1"),
            "/openai/deployments/whisper-1/audio/translations"
        );
        assert_eq!(
            deployment_path("/images/edits", "dall-e-2"),
            "/openai/deployments/dall-e-2/images/edits"
        );
    }

    #[test]
    fn deployment_path_non_deployment_routes() {
        assert_eq!(deployment_path("/models", "irrelevant"), "/openai/models");
        assert_eq!(deployment_path("/files", "irrelevant"), "/openai/files");
        assert_eq!(
            deployment_path("/fine-tuning/jobs", "irrelevant"),
            "/openai/fine-tuning/jobs"
        );
    }

    #[test]
    fn deployment_path_escapes_special_chars() {
        assert_eq!(
            deployment_path("/chat/completions", "my-model/v1"),
            "/openai/deployments/my-model%2Fv1/chat/completions"
        );
        assert_eq!(
            deployment_path("/chat/completions", "model with spaces"),
            "/openai/deployments/model%20with%20spaces/chat/completions"
        );
    }

    #[test]
    fn with_endpoint_preserves_custom_path() {
        let config = with_endpoint(
            "https://my-resource.openai.azure.com/custom/path",
            "2023-05-15",
        );
        assert_eq!(
            config.base_url.as_deref(),
            Some("https://my-resource.openai.azure.com/custom/path/openai")
        );
        assert_eq!(
            config.query_params,
            vec![("api-version".to_owned(), "2023-05-15".to_owned())]
        );
    }

    #[test]
    fn with_api_key_includes_endpoint_and_version() {
        let config = with_api_key(
            "https://my-resource.openai.azure.com",
            "2024-10-21",
            "key123",
        );
        assert_eq!(
            config.base_url.as_deref(),
            Some("https://my-resource.openai.azure.com/openai")
        );
        assert_eq!(
            config.query_params,
            vec![("api-version".to_owned(), "2024-10-21".to_owned())]
        );
    }
}
