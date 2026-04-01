//! Parameter helper types.

/// Optional parameter alias used in generated types.
pub type Opt<T> = Option<T>;

/// A helper for APIs that accept either one item or many.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    /// Single value.
    One(T),
    /// Multiple values.
    Many(Vec<T>),
}

impl<T> From<T> for OneOrMany<T> {
    fn from(value: T) -> Self {
        Self::One(value)
    }
}

impl<T> From<Vec<T>> for OneOrMany<T> {
    fn from(value: Vec<T>) -> Self {
        Self::Many(value)
    }
}

impl<T, const N: usize> From<[T; N]> for OneOrMany<T> {
    fn from(value: [T; N]) -> Self {
        Self::Many(value.into_iter().collect())
    }
}

impl<T> OneOrMany<T> {
    /// Converts this value into a vector, preserving order.
    #[must_use]
    pub fn into_vec(self) -> Vec<T> {
        match self {
            Self::One(value) => vec![value],
            Self::Many(values) => values,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OneOrMany;

    #[test]
    fn into_vec_wraps_single_value() {
        let value: OneOrMany<&str> = OneOrMany::One("hello");
        assert_eq!(value.into_vec(), vec!["hello"]);
    }

    #[test]
    fn from_array_creates_many_variant() {
        let value: OneOrMany<u8> = [1, 2, 3].into();
        assert!(matches!(value, OneOrMany::Many(_)));
    }
}
