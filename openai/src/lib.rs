#![allow(missing_docs)]
//! Rust SDK for the OpenAI API.

pub mod audio;
pub mod auth;
pub mod batches;
pub mod beta;
pub mod chat;
pub mod client;
pub mod completions;
pub mod config;
pub mod containers;
pub mod conversations;
pub mod embeddings;
pub mod error;
pub mod files;
pub mod fine_tuning;
pub mod graders;
pub mod images;
pub mod models;
pub mod moderations;
pub mod options;
pub mod pagination;
pub mod param;
pub mod raw_json;
pub mod realtime;
pub mod responses;
pub mod shared;
pub mod skills;
pub mod ssestream;
pub mod uploads;
pub mod vector_stores;
pub mod videos;
pub mod webhooks;

pub use client::Client;
pub use config::{RequestConfig, RequestOptions};
pub use error::{Error, Result};
pub use options::ClientConfig;
pub use param::{OneOrMany, Opt};
pub use raw_json::{JsonField, JsonFieldStatus, JsonMetadata, RawJsonExt};
