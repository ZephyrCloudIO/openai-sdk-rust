//! Integration tests for fine-tuning jobs and batches services.

use openai::{
    batches::{BatchCompletionWindow, BatchCreateParams, BatchEndpoint},
    fine_tuning::FineTuningJobCreateParams,
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
async fn fine_tuning_jobs_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/fine_tuning/jobs"))
        .and(body_partial_json(json!({"training_file":"file_train"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"ftjob_1",
            "object":"fine_tuning.job",
            "model":"gpt-4o-mini",
            "status":"queued",
            "training_file":"file_train"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/fine_tuning/jobs/ftjob_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"ftjob_1",
            "object":"fine_tuning.job",
            "model":"gpt-4o-mini",
            "status":"running",
            "training_file":"file_train"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/fine_tuning/jobs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id":"ftjob_1",
                "object":"fine_tuning.job",
                "model":"gpt-4o-mini",
                "status":"running",
                "training_file":"file_train"
            }],
            "has_more":false,
            "first_id":"ftjob_1",
            "last_id":"ftjob_1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/fine_tuning/jobs/ftjob_1/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"ftjob_1",
            "object":"fine_tuning.job",
            "model":"gpt-4o-mini",
            "status":"cancelled",
            "training_file":"file_train"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/fine_tuning/jobs/ftjob_1/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id":"ftevent_1",
                "object":"fine_tuning.job.event",
                "created_at":123,
                "message":"queued",
                "level":"info"
            }],
            "has_more":false,
            "first_id":"ftevent_1",
            "last_id":"ftevent_1"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let jobs = client.fine_tuning().jobs();

    let created = jobs
        .create(FineTuningJobCreateParams {
            model: "gpt-4o-mini".to_owned(),
            training_file: "file_train".to_owned(),
            validation_file: None,
            suffix: None,
            metadata: None,
            seed: None,
            integrations: None,
            hyperparameters: None,
            method: None,
        })
        .await
        .expect("create fine-tuning job");
    assert_eq!(created.id, "ftjob_1");

    let got = jobs.get("ftjob_1").await.expect("get fine-tuning job");
    assert_eq!(got.id, "ftjob_1");

    let listed = jobs.list().await.expect("list fine-tuning jobs");
    assert_eq!(listed.data.len(), 1);

    let cancelled = jobs
        .cancel("ftjob_1")
        .await
        .expect("cancel fine-tuning job");
    assert_eq!(cancelled.status, "cancelled");

    let events = jobs
        .list_events("ftjob_1")
        .await
        .expect("list fine-tuning job events");
    assert_eq!(events.data.len(), 1);
}

#[tokio::test]
async fn batches_round_trip() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/batches"))
        .and(body_partial_json(json!({"input_file_id":"file_input"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"batch_1",
            "object":"batch",
            "status":"in_progress",
            "endpoint":"/v1/responses",
            "input_file_id":"file_input",
            "completion_window":"24h"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/batches/batch_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"batch_1",
            "object":"batch",
            "status":"completed",
            "endpoint":"/v1/responses",
            "input_file_id":"file_input",
            "completion_window":"24h"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/batches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object":"list",
            "data":[{
                "id":"batch_1",
                "object":"batch",
                "status":"completed",
                "endpoint":"/v1/responses",
                "input_file_id":"file_input",
                "completion_window":"24h"
            }],
            "has_more":false,
            "first_id":"batch_1",
            "last_id":"batch_1"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/batches/batch_1/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"batch_1",
            "object":"batch",
            "status":"cancelled",
            "endpoint":"/v1/responses",
            "input_file_id":"file_input",
            "completion_window":"24h"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let batches = client.batches();

    let created = batches
        .create(BatchCreateParams {
            input_file_id: "file_input".to_owned(),
            endpoint: BatchEndpoint::V1Responses,
            completion_window: BatchCompletionWindow::TwentyFourHours,
            metadata: None,
            output_expires_after: None,
        })
        .await
        .expect("create batch");
    assert_eq!(created.id, "batch_1");

    let got = batches.get("batch_1").await.expect("get batch");
    assert_eq!(got.status, "completed");

    let listed = batches.list(None).await.expect("list batches");
    assert_eq!(listed.data.len(), 1);

    let cancelled = batches.cancel("batch_1").await.expect("cancel batch");
    assert_eq!(cancelled.status, "cancelled");
}
