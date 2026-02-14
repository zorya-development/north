/// Serde module for `Option<Option<T>>` three-state pattern:
/// - `None` → field omitted (don't change)
/// - `Some(None)` → JSON `null` (clear to null)
/// - `Some(Some(v))` → JSON value (set)
pub mod double_option {
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S, T>(value: &Option<Option<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        match value {
            None => serializer.serialize_none(),
            Some(inner) => inner.serialize(serializer),
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: DeserializeOwned,
    {
        // If the field is present, deserialize its value
        let value: Option<T> = Option::deserialize(deserializer)?;
        Ok(Some(value))
    }
}

pub fn is_none_outer<T>(opt: &Option<Option<T>>) -> bool {
    opt.is_none()
}
