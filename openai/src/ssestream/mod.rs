//! SSE stream decoding for streaming APIs.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use eventsource_stream::Eventsource;
use futures::{Stream, StreamExt};
use serde::{de::DeserializeOwned, Serialize};

use crate::{Error, Result};

enum ParseEventAction<T> {
    Yield(Result<T>),
    Skip,
    Stop,
}

fn normalize_payload(data: &str) -> String {
    let uses_data_prefix = data
        .lines()
        .any(|line| line.trim_start().starts_with("data:"));

    if !uses_data_prefix {
        return data.trim().to_owned();
    }

    let mut lines = Vec::new();
    for line in data.lines() {
        let trimmed = line.trim_end_matches('\r');
        if let Some(after_prefix) = trimmed.trim_start().strip_prefix("data:") {
            lines.push(after_prefix.trim_start().to_owned());
        } else {
            lines.push(trimmed.to_owned());
        }
    }

    lines.join("\n").trim().to_owned()
}

fn extract_error_message(data: &str) -> String {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
        if let Some(message) = parsed
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(serde_json::Value::as_str)
        {
            return message.to_owned();
        }

        if let Some(message) = parsed.get("message").and_then(serde_json::Value::as_str) {
            return message.to_owned();
        }
    }

    data.to_owned()
}

fn parse_event_payload<T>(event_type: &str, data: &str) -> ParseEventAction<T>
where
    T: DeserializeOwned + Serialize,
{
    let payload = normalize_payload(data);

    if payload == "[DONE]" {
        return ParseEventAction::Stop;
    }

    if payload.is_empty() {
        return ParseEventAction::Skip;
    }

    if event_type.trim() == "error" {
        return ParseEventAction::Yield(Err(Error::Stream(extract_error_message(&payload))));
    }

    match serde_json::from_str::<T>(&payload) {
        Ok(value) => {
            crate::raw_json::register_raw_json(&value, &payload);
            ParseEventAction::Yield(Ok(value))
        }
        Err(err) => ParseEventAction::Yield(Err(Error::Json(err))),
    }
}

/// Generic SSE stream for API events.
pub struct SseStream<T> {
    inner: Pin<Box<dyn Stream<Item = Result<T>> + Send>>,
}

impl<T> SseStream<T>
where
    T: DeserializeOwned + Serialize + Send + 'static,
{
    /// Builds an SSE stream from an HTTP response body stream.
    #[must_use]
    pub fn new(response: reqwest::Response) -> Self {
        let stream = async_stream::stream! {
            let mut events = response.bytes_stream().eventsource();
            while let Some(event) = events.next().await {
                let event = match event {
                    Ok(ev) => ev,
                    Err(err) => {
                        yield Err(Error::Stream(err.to_string()));
                        break;
                    }
                };

                match parse_event_payload::<T>(&event.event, &event.data) {
                    ParseEventAction::Stop => break,
                    ParseEventAction::Skip => continue,
                    ParseEventAction::Yield(Ok(value)) => yield Ok(value),
                    ParseEventAction::Yield(Err(err)) => {
                        yield Err(err);
                        break;
                    }
                }
            }
        };

        Self {
            inner: Box::pin(stream),
        }
    }
}

impl<T> Stream for SseStream<T> {
    type Item = Result<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().inner.as_mut().poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    use crate::Error;

    #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    struct TestChunk {
        token: String,
    }

    #[test]
    fn parse_event_payload_stops_on_done() {
        let payload = super::parse_event_payload::<TestChunk>("message", " [DONE] \n");
        assert!(matches!(payload, super::ParseEventAction::Stop));
    }

    #[test]
    fn parse_event_payload_skips_empty_payload() {
        let payload = super::parse_event_payload::<TestChunk>("message", "\n\n");
        assert!(matches!(payload, super::ParseEventAction::Skip));
    }

    #[test]
    fn parse_event_payload_handles_error_events() {
        let payload = super::parse_event_payload::<TestChunk>(
            "error",
            r#"{"error":{"message":"upstream failed"}}"#,
        );
        match payload {
            super::ParseEventAction::Yield(Err(Error::Stream(message))) => {
                assert_eq!(message, "upstream failed")
            }
            _ => panic!("expected stream error payload"),
        }
    }

    #[test]
    fn parse_event_payload_deserializes_multiline_data_prefix() {
        let payload = super::parse_event_payload::<TestChunk>(
            "message",
            "data: {\ndata:   \"token\": \"hi\"\ndata: }",
        );
        match payload {
            super::ParseEventAction::Yield(Ok(chunk)) => assert_eq!(
                chunk,
                TestChunk {
                    token: "hi".to_owned()
                }
            ),
            _ => panic!("expected successful payload"),
        }
    }

    #[test]
    fn parse_event_payload_deserializes_data_without_prefix() {
        let payload = super::parse_event_payload::<TestChunk>("message", "{\"token\":\"hi\"}");
        match payload {
            super::ParseEventAction::Yield(Ok(chunk)) => assert_eq!(
                chunk,
                TestChunk {
                    token: "hi".to_owned()
                }
            ),
            _ => panic!("expected successful payload"),
        }
    }

    // --- Proptest fuzz tests ---

    mod fuzz {
        use proptest::prelude::*;

        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        #[allow(dead_code)]
        struct AnyChunk {
            #[serde(default)]
            value: Option<String>,
        }

        proptest! {
            #[test]
            fn parse_event_payload_never_panics(
                event_type in ".*",
                data in ".*",
            ) {
                // We only care that it does not panic; the result is irrelevant.
                let _ = super::super::parse_event_payload::<AnyChunk>(&event_type, &data);
            }

            #[test]
            fn normalize_payload_never_panics(data in ".*") {
                let _ = super::super::normalize_payload(&data);
            }

            #[test]
            fn extract_error_message_never_panics(data in ".*") {
                let _ = super::super::extract_error_message(&data);
            }
        }
    }
}
