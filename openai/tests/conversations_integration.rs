//! Integration tests for conversations service.

use std::collections::HashMap;

use openai::{
    conversations::{
        ConversationCreateParams, ConversationInputContent, ConversationInputItem,
        ConversationItemCreateParams, ConversationItemListParams, ConversationUpdateParams,
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

#[tokio::test]
async fn conversations_crud_round_trip() {
    let server = MockServer::start().await;

    // Create
    Mock::given(method("POST"))
        .and(path("/conversations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "conv_1",
            "object": "conversation",
            "created_at": 1234567890,
            "metadata": {"key": "value"}
        })))
        .mount(&server)
        .await;

    // Get
    Mock::given(method("GET"))
        .and(path("/conversations/conv_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "conv_1",
            "object": "conversation",
            "created_at": 1234567890,
            "metadata": {"key": "value"}
        })))
        .mount(&server)
        .await;

    // Update
    Mock::given(method("POST"))
        .and(path("/conversations/conv_1"))
        .and(body_partial_json(json!({"metadata": {"key": "updated"}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "conv_1",
            "object": "conversation",
            "created_at": 1234567890,
            "metadata": {"key": "updated"}
        })))
        .mount(&server)
        .await;

    // List
    Mock::given(method("GET"))
        .and(path("/conversations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [{
                "id": "conv_1",
                "object": "conversation",
                "created_at": 1234567890,
                "metadata": {"key": "updated"}
            }],
            "has_more": false,
            "first_id": "conv_1",
            "last_id": "conv_1"
        })))
        .mount(&server)
        .await;

    // Delete
    Mock::given(method("DELETE"))
        .and(path("/conversations/conv_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "conv_1",
            "object": "conversation.deleted",
            "deleted": true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let conversations = client.conversations();

    // Create conversation
    let mut meta = HashMap::new();
    meta.insert("key".to_owned(), "value".to_owned());
    let created = conversations
        .create(ConversationCreateParams {
            items: None,
            metadata: Some(meta),
        })
        .await
        .expect("create conversation");
    assert_eq!(created.id, "conv_1");
    assert_eq!(created.object, "conversation");

    // Get conversation
    let got = conversations.get("conv_1").await.expect("get conversation");
    assert_eq!(got.id, "conv_1");

    // Update conversation
    let mut updated_meta = HashMap::new();
    updated_meta.insert("key".to_owned(), "updated".to_owned());
    let updated = conversations
        .update(
            "conv_1",
            ConversationUpdateParams {
                metadata: updated_meta,
            },
        )
        .await
        .expect("update conversation");
    assert_eq!(updated.id, "conv_1");

    // List conversations
    let listed = conversations.list().await.expect("list conversations");
    assert_eq!(listed.data.len(), 1);
    assert!(!listed.has_more);

    // Delete conversation
    let deleted = conversations
        .delete("conv_1")
        .await
        .expect("delete conversation");
    assert!(deleted.deleted);
}

#[tokio::test]
async fn conversation_items_crud_round_trip() {
    let server = MockServer::start().await;

    // Create items
    Mock::given(method("POST"))
        .and(path("/conversations/conv_1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [{
                "id": "msg_1",
                "type": "message",
                "status": "completed",
                "role": "user",
                "content": [{"type": "input_text", "text": "Hello"}]
            }],
            "has_more": false,
            "first_id": "msg_1",
            "last_id": "msg_1"
        })))
        .mount(&server)
        .await;

    // Get item
    Mock::given(method("GET"))
        .and(path("/conversations/conv_1/items/msg_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_1",
            "type": "message",
            "status": "completed",
            "role": "user",
            "content": [{"type": "input_text", "text": "Hello"}]
        })))
        .mount(&server)
        .await;

    // List items
    Mock::given(method("GET"))
        .and(path("/conversations/conv_1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [{
                "id": "msg_1",
                "type": "message",
                "status": "completed",
                "role": "user",
                "content": [{"type": "input_text", "text": "Hello"}]
            }],
            "has_more": false,
            "first_id": "msg_1",
            "last_id": "msg_1"
        })))
        .mount(&server)
        .await;

    // Delete item
    Mock::given(method("DELETE"))
        .and(path("/conversations/conv_1/items/msg_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_1",
            "object": "conversation.item.deleted",
            "deleted": true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let items = client.conversations().items();

    // Create items
    let created = items
        .create(
            "conv_1",
            ConversationItemCreateParams {
                items: vec![ConversationInputItem {
                    item_type: "message".to_owned(),
                    role: Some("user".to_owned()),
                    content: Some(vec![ConversationInputContent {
                        content_type: "input_text".to_owned(),
                        text: Some("Hello".to_owned()),
                    }]),
                }],
            },
        )
        .await
        .expect("create items");
    assert_eq!(created.data.len(), 1);
    assert_eq!(created.data[0].id, "msg_1");

    // Get item
    let got = items.get("conv_1", "msg_1", None).await.expect("get item");
    match &got {
        openai::conversations::ConversationItemUnion::Message { id, .. } => {
            assert_eq!(id, "msg_1");
        }
        _ => panic!("expected Message variant"),
    }

    // List items
    let listed = items
        .list(
            "conv_1",
            ConversationItemListParams {
                after: None,
                limit: Some(10),
                order: Some("asc".to_owned()),
                include: None,
            },
        )
        .await
        .expect("list items");
    assert_eq!(listed.data.len(), 1);

    // Delete item
    let deleted = items.delete("conv_1", "msg_1").await.expect("delete item");
    assert!(deleted.deleted);
}
