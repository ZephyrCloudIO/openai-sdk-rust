//! Raw JSON metadata captured while decoding API responses.

use std::{
    collections::{BTreeMap, HashMap},
    sync::{Mutex, OnceLock},
};

use serde::Serialize;
use serde_json::Value;

static REGISTRY: OnceLock<Mutex<HashMap<String, JsonMetadata>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<String, JsonMetadata>> {
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Presence status for a JSON field captured from the API response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonFieldStatus {
    /// The field was omitted.
    Omitted,
    /// The field was present with a JSON `null` value.
    Null,
    /// The field was present but could not be decoded into the generated type.
    Invalid,
    /// The field was present with a non-null JSON value.
    Valid,
}

/// Metadata for a single JSON field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonField {
    raw: String,
    status: JsonFieldStatus,
}

impl JsonField {
    /// Creates field metadata for an omitted field.
    #[must_use]
    pub fn omitted() -> Self {
        Self {
            raw: String::new(),
            status: JsonFieldStatus::Omitted,
        }
    }

    /// Creates field metadata from a raw JSON value.
    #[must_use]
    pub fn from_value(value: &Value) -> Self {
        let raw = serde_json::to_string(value).unwrap_or_default();
        let status = if value.is_null() {
            JsonFieldStatus::Null
        } else {
            JsonFieldStatus::Valid
        };
        Self { raw, status }
    }

    /// Creates field metadata for a value that failed typed decoding.
    #[must_use]
    pub fn invalid(raw: impl Into<String>) -> Self {
        Self {
            raw: raw.into(),
            status: JsonFieldStatus::Invalid,
        }
    }

    /// Returns true when the field was present and decoded successfully.
    #[must_use]
    pub fn valid(&self) -> bool {
        self.status == JsonFieldStatus::Valid
    }

    /// Returns the raw JSON value for this field.
    ///
    /// Omitted fields return an empty string, matching Go's `respjson.Field`.
    #[must_use]
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Returns the captured presence status.
    #[must_use]
    pub fn status(&self) -> JsonFieldStatus {
        self.status
    }
}

/// Raw JSON metadata for a decoded response value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JsonMetadata {
    raw: String,
    extra_fields: BTreeMap<String, JsonField>,
    fields: BTreeMap<String, JsonField>,
}

impl JsonMetadata {
    /// Returns the raw JSON document used to decode the value.
    #[must_use]
    pub fn raw_json(&self) -> &str {
        &self.raw
    }

    /// Returns unknown fields that were present in the response object.
    #[must_use]
    pub fn extra_fields(&self) -> &BTreeMap<String, JsonField> {
        &self.extra_fields
    }

    /// Returns field presence metadata for a named JSON field.
    #[must_use]
    pub fn field(&self, name: &str) -> JsonField {
        self.fields
            .get(name)
            .cloned()
            .unwrap_or_else(JsonField::omitted)
    }

    fn from_values(typed: &Value, raw: &Value, raw_json: String) -> Self {
        let mut metadata = Self {
            raw: raw_json,
            extra_fields: BTreeMap::new(),
            fields: BTreeMap::new(),
        };

        let (Value::Object(typed_object), Value::Object(raw_object)) = (typed, raw) else {
            return metadata;
        };

        for (name, value) in raw_object {
            let field = JsonField::from_value(value);
            metadata.fields.insert(name.clone(), field.clone());
            if !typed_object.contains_key(name) {
                metadata.extra_fields.insert(name.clone(), field);
            }
        }

        metadata
    }
}

/// Extension trait for values decoded by the SDK client.
pub trait RawJsonExt: Serialize {
    /// Returns the raw JSON document used to decode this value, when captured.
    #[must_use]
    fn raw_json(&self) -> String {
        metadata_for(self)
            .map(|metadata| metadata.raw_json().to_owned())
            .unwrap_or_default()
    }

    /// Returns unknown fields that were present in the decoded JSON object.
    #[must_use]
    fn extra_fields(&self) -> BTreeMap<String, JsonField> {
        metadata_for(self)
            .map(|metadata| metadata.extra_fields().clone())
            .unwrap_or_default()
    }

    /// Returns presence metadata for a named JSON field.
    #[must_use]
    fn json_field(&self, name: &str) -> JsonField {
        metadata_for(self)
            .map(|metadata| metadata.field(name))
            .unwrap_or_else(JsonField::omitted)
    }
}

impl<T> RawJsonExt for T where T: Serialize {}

/// Registers raw JSON metadata for a decoded value.
pub(crate) fn register_raw_json<T>(value: &T, raw_json: &str)
where
    T: Serialize + ?Sized,
{
    let Ok(typed_value) = serde_json::to_value(value) else {
        return;
    };
    let Ok(raw_value) = serde_json::from_str::<Value>(raw_json) else {
        return;
    };

    register_value_tree(&typed_value, &raw_value, raw_json.to_owned());
}

fn metadata_for<T>(value: &T) -> Option<JsonMetadata>
where
    T: Serialize + ?Sized,
{
    let typed_value = serde_json::to_value(value).ok()?;
    let key = metadata_key(&typed_value)?;
    registry().lock().ok()?.get(&key).cloned()
}

fn register_value_tree(typed: &Value, raw: &Value, raw_json: String) {
    if let Some(key) = metadata_key(typed) {
        let metadata = JsonMetadata::from_values(typed, raw, raw_json);
        if let Ok(mut registry) = registry().lock() {
            registry.insert(key, metadata);
        }
    }

    match (typed, raw) {
        (Value::Object(typed_object), Value::Object(raw_object)) => {
            for (name, typed_child) in typed_object {
                let Some(raw_child) = raw_object.get(name) else {
                    continue;
                };
                let raw_child_json = serde_json::to_string(raw_child).unwrap_or_default();
                register_value_tree(typed_child, raw_child, raw_child_json);
            }
        }
        (Value::Array(typed_items), Value::Array(raw_items)) => {
            for (typed_child, raw_child) in typed_items.iter().zip(raw_items.iter()) {
                let raw_child_json = serde_json::to_string(raw_child).unwrap_or_default();
                register_value_tree(typed_child, raw_child, raw_child_json);
            }
        }
        _ => {}
    }
}

fn metadata_key(value: &Value) -> Option<String> {
    serde_json::to_string(value).ok()
}

#[cfg(test)]
mod tests {
    use super::{JsonFieldStatus, RawJsonExt};

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Example {
        id: String,
        optional: Option<String>,
    }

    #[test]
    fn captures_raw_json_extra_fields_and_presence() {
        let raw = r#"{"id":"x","optional":null,"unknown":{"a":1}}"#;
        let value: Example = serde_json::from_str(raw).expect("decode example");
        super::register_raw_json(&value, raw);

        assert_eq!(value.raw_json(), raw);
        assert_eq!(value.json_field("id").raw(), r#""x""#);
        assert_eq!(value.json_field("optional").status(), JsonFieldStatus::Null);
        assert_eq!(
            value.json_field("missing").status(),
            JsonFieldStatus::Omitted
        );
        assert!(value.extra_fields().contains_key("unknown"));
    }
}
