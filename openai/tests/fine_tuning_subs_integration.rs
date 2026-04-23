//! Integration tests for fine-tuning sub-services (checkpoints, permissions, graders).

use openai::{
    fine_tuning::{
        AlphaGraderRunParams, AlphaGraderValidateParams, CheckpointPermissionCreateParams,
    },
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

// ---------------------------------------------------------------------------
// Job checkpoint service
// ---------------------------------------------------------------------------

#[tokio::test]
async fn job_checkpoints_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/fine_tuning/jobs/ftjob_1/checkpoints"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id":"ftckpt_1",
                "object":"fine_tuning.job.checkpoint",
                "created_at":1700000000,
                "fine_tuned_model_checkpoint":"ft:gpt-4o-mini:org:ckpt-step-10",
                "fine_tuning_job_id":"ftjob_1",
                "metrics":{
                    "step":10.0,
                    "train_loss":0.5
                },
                "step_number":10
            }],
            "has_more":false,
            "first_id":"ftckpt_1",
            "last_id":"ftckpt_1"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let checkpoints = client
        .fine_tuning()
        .jobs()
        .checkpoints()
        .list("ftjob_1", None)
        .await
        .expect("list job checkpoints");

    assert_eq!(checkpoints.data.len(), 1);
    assert_eq!(checkpoints.data[0].id, "ftckpt_1");
    assert_eq!(checkpoints.data[0].step_number, 10);
}

// ---------------------------------------------------------------------------
// Checkpoint permission service
// ---------------------------------------------------------------------------

#[tokio::test]
async fn checkpoint_permissions_create() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/fine_tuning/checkpoints/ckpt_abc/permissions"))
        .and(body_partial_json(json!({"project_ids":["proj_1"]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id":"cperm_1",
                "object":"checkpoint.permission",
                "created_at":1700000000,
                "project_id":"proj_1"
            }],
            "has_more":false
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let resp = client
        .fine_tuning()
        .checkpoint_permissions()
        .create(
            "ckpt_abc",
            CheckpointPermissionCreateParams {
                project_ids: vec!["proj_1".to_owned()],
            },
        )
        .await
        .expect("create checkpoint permissions");

    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].project_id, "proj_1");
}

#[tokio::test]
async fn checkpoint_permissions_get() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/fine_tuning/checkpoints/ckpt_abc/permissions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id":"cperm_1",
                "object":"checkpoint.permission",
                "created_at":1700000000,
                "project_id":"proj_1"
            }],
            "has_more":false,
            "first_id":"cperm_1",
            "last_id":"cperm_1"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let resp = client
        .fine_tuning()
        .checkpoint_permissions()
        .get("ckpt_abc", None)
        .await
        .expect("get checkpoint permissions");

    assert_eq!(resp.data.len(), 1);
    assert!(!resp.has_more);
}

#[tokio::test]
async fn checkpoint_permissions_delete() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(
            "/fine_tuning/checkpoints/ckpt_abc/permissions/cperm_1",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"cperm_1",
            "deleted":true,
            "object":"checkpoint.permission"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let resp = client
        .fine_tuning()
        .checkpoint_permissions()
        .delete("ckpt_abc", "cperm_1")
        .await
        .expect("delete checkpoint permission");

    assert!(resp.deleted);
    assert_eq!(resp.id, "cperm_1");
}

// ---------------------------------------------------------------------------
// Alpha grader service
// ---------------------------------------------------------------------------

#[tokio::test]
async fn alpha_grader_run() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/fine_tuning/alpha/graders/run"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata":{
                "errors":{
                    "formula_parse_error":false,
                    "invalid_variable_error":false,
                    "model_grader_parse_error":false,
                    "model_grader_refusal_error":false,
                    "model_grader_server_error":false,
                    "model_grader_server_error_details":"",
                    "other_error":false,
                    "python_grader_runtime_error":false,
                    "python_grader_runtime_error_details":"",
                    "python_grader_server_error":false,
                    "python_grader_server_error_type":"",
                    "sample_parse_error":false,
                    "truncated_observation_error":false,
                    "unresponsive_reward_error":false
                },
                "execution_time":0.01,
                "name":"test_grader",
                "sampled_model_name":"gpt-4o-mini",
                "scores":{},
                "token_usage":10,
                "type":"string_check"
            },
            "model_grader_token_usage_per_model":{},
            "reward":1.0,
            "sub_rewards":{}
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let resp = client
        .fine_tuning()
        .alpha_graders()
        .run(AlphaGraderRunParams {
            grader: json!({
                "type": "string_check",
                "name": "test_grader",
                "input": "sample",
                "reference": "expected",
                "operation": "eq"
            }),
            model_sample: "Hello world".to_owned(),
            item: None,
        })
        .await
        .expect("run alpha grader");

    assert!((resp.reward - 1.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn alpha_grader_validate() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/fine_tuning/alpha/graders/validate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "grader":{
                "type":"python",
                "name":"my_grader",
                "source":"def grade(sample, item): return 1.0"
            }
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let resp = client
        .fine_tuning()
        .alpha_graders()
        .validate(AlphaGraderValidateParams {
            grader: json!({
                "type": "python",
                "name": "my_grader",
                "source": "def grade(sample, item): return 1.0"
            }),
        })
        .await
        .expect("validate alpha grader");

    assert!(resp.grader.is_some());
}

// ---------------------------------------------------------------------------
// Checkpoint service (structural navigation)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn checkpoint_service_permissions_accessor() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/fine_tuning/checkpoints/ckpt_abc/permissions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id":"cperm_1",
                "object":"checkpoint.permission",
                "created_at":1700000000,
                "project_id":"proj_1"
            }],
            "has_more":false,
            "first_id":"cperm_1",
            "last_id":"cperm_1"
        })))
        .mount(&server)
        .await;

    // Access permissions through checkpoint_permissions() path
    let client = test_client(&server);
    let resp = client
        .fine_tuning()
        .checkpoint_permissions()
        .get("ckpt_abc", None)
        .await
        .expect("get permissions via checkpoints accessor");

    assert_eq!(resp.data.len(), 1);
}
