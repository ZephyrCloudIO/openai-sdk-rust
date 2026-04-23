//! Integration tests for skills service.

use openai::{
    skills::{SkillFileUpload, SkillUpdateParams},
    Client, ClientConfig,
};
use serde_json::json;
use wiremock::{
    matchers::{body_partial_json, method, path},
    Mock, MockServer, ResponseTemplate,
};

fn test_client(server: &MockServer) -> Client {
    Client::new(
        ClientConfig::default()
            .with_api_key("test-key")
            .with_base_url(server.uri())
            .with_max_retries(0),
    )
    .expect("client init")
}

#[tokio::test]
async fn skills_crud_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/skills"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"skill_1",
            "object":"skill",
            "created_at":1700000000,
            "default_version":"v1",
            "description":"A test skill",
            "latest_version":"v1",
            "name":"my-skill"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/skills/skill_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"skill_1",
            "object":"skill",
            "created_at":1700000000,
            "default_version":"v1",
            "description":"A test skill",
            "latest_version":"v1",
            "name":"my-skill"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/skills/skill_1"))
        .and(body_partial_json(json!({"default_version":"v2"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"skill_1",
            "object":"skill",
            "created_at":1700000000,
            "default_version":"v2",
            "description":"A test skill",
            "latest_version":"v2",
            "name":"my-skill"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/skills"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id":"skill_1",
                "object":"skill",
                "created_at":1700000000,
                "default_version":"v2",
                "description":"A test skill",
                "latest_version":"v2",
                "name":"my-skill"
            }],
            "has_more":false,
            "first_id":"skill_1",
            "last_id":"skill_1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/skills/skill_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"skill_1",
            "object":"skill.deleted",
            "deleted":true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let skills = client.skills();

    // Create
    let created = skills
        .create(openai::skills::SkillCreateParams {
            files: vec![SkillFileUpload::from_bytes(b"data".to_vec(), "skill.zip")],
        })
        .await
        .expect("create skill");
    assert_eq!(created.id, "skill_1");
    assert_eq!(created.name, "my-skill");

    // Get
    let got = skills.get("skill_1").await.expect("get skill");
    assert_eq!(got.id, "skill_1");
    assert_eq!(got.default_version, "v1");

    // Update
    let updated = skills
        .update(
            "skill_1",
            SkillUpdateParams {
                default_version: "v2".to_owned(),
            },
        )
        .await
        .expect("update skill");
    assert_eq!(updated.default_version, "v2");

    // List
    let listed = skills.list().await.expect("list skills");
    assert_eq!(listed.data.len(), 1);

    // Delete
    let deleted = skills.delete("skill_1").await.expect("delete skill");
    assert!(deleted.deleted);
}

#[tokio::test]
async fn skill_content_download() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/skills/skill_1/content"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"zipdata".to_vec()))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let content = client.skills().content();

    let bytes = content
        .get("skill_1")
        .await
        .expect("download skill content");
    assert_eq!(bytes, b"zipdata");
}

#[tokio::test]
async fn skill_versions_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/skills/skill_1/versions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"skillver_1",
            "object":"skill.version",
            "created_at":1700000000,
            "description":"First version",
            "name":"my-skill",
            "skill_id":"skill_1",
            "version":"v1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/skills/skill_1/versions/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"skillver_1",
            "object":"skill.version",
            "created_at":1700000000,
            "description":"First version",
            "name":"my-skill",
            "skill_id":"skill_1",
            "version":"v1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/skills/skill_1/versions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id":"skillver_1",
                "object":"skill.version",
                "created_at":1700000000,
                "description":"First version",
                "name":"my-skill",
                "skill_id":"skill_1",
                "version":"v1"
            }],
            "has_more":false,
            "first_id":"skillver_1",
            "last_id":"skillver_1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/skills/skill_1/versions/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"skill_1",
            "object":"skill.version.deleted",
            "deleted":true,
            "version":"v1"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let versions = client.skills().versions();

    // Create version
    let created = versions
        .create(
            "skill_1",
            openai::skills::SkillVersionCreateParams {
                default: Some(true),
                files: vec![SkillFileUpload::from_bytes(b"data".to_vec(), "v1.zip")],
            },
        )
        .await
        .expect("create version");
    assert_eq!(created.version, "v1");

    // Get version
    let got = versions.get("skill_1", "v1").await.expect("get version");
    assert_eq!(got.skill_id, "skill_1");

    // List versions
    let listed = versions.list("skill_1").await.expect("list versions");
    assert_eq!(listed.data.len(), 1);

    // Delete version
    let deleted = versions
        .delete("skill_1", "v1")
        .await
        .expect("delete version");
    assert!(deleted.deleted);
    assert_eq!(deleted.version, "v1");
}

#[tokio::test]
async fn skill_version_content_download() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/skills/skill_1/versions/v1/content"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"versionzip".to_vec()))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let content = client.skills().versions().content();

    let bytes = content
        .get("skill_1", "v1")
        .await
        .expect("download version content");
    assert_eq!(bytes, b"versionzip");
}
