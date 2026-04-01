//! Integration test for graders namespace wiring.

use openai::{Client, ClientConfig};

#[tokio::test]
async fn graders_namespace_is_accessible_from_client() {
    let client = Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url("http://localhost"),
    )
    .expect("client init");

    let graders = client.graders();
    let models = graders.grader_models();

    // Namespace smoke check: service wiring composes without request execution.
    let _ = models.client_ref();
}
