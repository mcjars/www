use serde::{Deserialize, Deserializer};

pub fn deserialize_string_option<'de, D>(
    deserializer: D,
) -> Result<Option<compact_str::CompactString>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<compact_str::CompactString> =
        Option::deserialize(deserializer).unwrap_or_default();
    Ok(value.filter(|s| !s.is_empty()))
}
