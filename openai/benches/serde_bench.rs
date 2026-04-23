//! Criterion benchmarks for serde (de)serialization of key SDK types.

use criterion::{criterion_group, criterion_main, Criterion};

// ---------------------------------------------------------------------------
// Sample JSON payloads
// ---------------------------------------------------------------------------

const CHAT_COMPLETION_JSON: &str = r#"{
    "id": "chatcmpl-abc123",
    "object": "chat.completion",
    "created": 1700000000,
    "model": "gpt-4o",
    "choices": [
        {
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello! How can I help you today?"
            },
            "finish_reason": "stop"
        }
    ],
    "usage": {
        "prompt_tokens": 12,
        "completion_tokens": 8,
        "total_tokens": 20,
        "completion_tokens_details": {
            "reasoning_tokens": 2,
            "accepted_prediction_tokens": 1,
            "rejected_prediction_tokens": 0
        },
        "prompt_tokens_details": {
            "cached_tokens": 4
        }
    },
    "service_tier": "default",
    "system_fingerprint": "fp_abc123"
}"#;

const RESPONSE_JSON: &str = r#"{
    "id": "resp_abc123",
    "object": "response",
    "created_at": 1700000000.0,
    "status": "completed",
    "model": "gpt-4o",
    "output": [
        {
            "type": "message",
            "id": "msg_001",
            "role": "assistant",
            "content": [
                {
                    "type": "output_text",
                    "text": "Hello! How can I help you today?",
                    "annotations": []
                }
            ],
            "status": "completed"
        }
    ]
}"#;

const CHAT_COMPLETION_CHUNK_JSON: &str = r#"{
    "id": "chatcmpl-stream123",
    "object": "chat.completion.chunk",
    "created": 1700000000,
    "model": "gpt-4o",
    "choices": [
        {
            "index": 0,
            "delta": {
                "role": "assistant",
                "content": "Hello"
            },
            "finish_reason": null
        }
    ]
}"#;

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

fn bench_chat_completion_deser(c: &mut Criterion) {
    c.bench_function("ChatCompletion deserialization", |b| {
        b.iter(|| {
            let _: openai::chat::ChatCompletion =
                serde_json::from_str(criterion::black_box(CHAT_COMPLETION_JSON)).unwrap();
        });
    });
}

fn bench_response_deser(c: &mut Criterion) {
    c.bench_function("Response deserialization", |b| {
        b.iter(|| {
            let _: openai::responses::Response =
                serde_json::from_str(criterion::black_box(RESPONSE_JSON)).unwrap();
        });
    });
}

fn bench_chat_completion_chunk_deser(c: &mut Criterion) {
    c.bench_function("ChatCompletionChunk deserialization", |b| {
        b.iter(|| {
            let _: openai::chat::ChatCompletionChunk =
                serde_json::from_str(criterion::black_box(CHAT_COMPLETION_CHUNK_JSON)).unwrap();
        });
    });
}

fn bench_chat_completion_ser(c: &mut Criterion) {
    let completion: openai::chat::ChatCompletion =
        serde_json::from_str(CHAT_COMPLETION_JSON).unwrap();
    c.bench_function("ChatCompletion serialization", |b| {
        b.iter(|| {
            let _ = serde_json::to_string(criterion::black_box(&completion)).unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_chat_completion_deser,
    bench_response_deser,
    bench_chat_completion_chunk_deser,
    bench_chat_completion_ser,
);
criterion_main!(benches);
