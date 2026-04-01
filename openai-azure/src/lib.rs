//! Azure OpenAI configuration helpers.

#[cfg(feature = "config")]
use openai::ClientConfig;

#[cfg(feature = "azure")]
use std::sync::Arc;

#[cfg(feature = "azure")]
use azure_core::credentials::TokenCredential;

#[cfg(feature = "config")]
const API_KEY_HEADER: &str = "api-key";

#[cfg(feature = "config")]
const API_VERSION_QUERY_PARAM: &str = "api-version";

#[cfg(feature = "azure")]
const COGNITIVE_SERVICES_SCOPE: &str = "https://cognitiveservices.azure.com/.default";

/// Builds an `openai::ClientConfig` with Azure endpoint and API version defaults.
#[cfg(feature = "config")]
pub fn with_endpoint(endpoint: &str, api_version: &str) -> ClientConfig {
    let base_url = format!("{}/openai", endpoint.trim_end_matches('/'));

    ClientConfig::default()
        .with_base_url(base_url)
        .with_query_param(API_VERSION_QUERY_PARAM, api_version)
}

/// Builds an `openai::ClientConfig` with endpoint, API version, and API key.
#[cfg(feature = "config")]
pub fn with_api_key(endpoint: &str, api_version: &str, api_key: impl Into<String>) -> ClientConfig {
    with_endpoint(endpoint, api_version).with_header(API_KEY_HEADER, api_key)
}

/// Builds an `openai::ClientConfig` with endpoint, API version, and bearer token auth.
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
