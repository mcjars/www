use axum::http::HeaderMap;
use compact_str::ToCompactString;
use garde::Validate;
use serde::Serialize;
use std::net::IpAddr;

#[inline]
pub fn extract_ip(headers: &HeaderMap) -> Option<IpAddr> {
    let ip = headers
        .get("x-real-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .map(|ip| ip.to_str().unwrap_or_default())
        .unwrap_or_default();

    if ip.is_empty() {
        return None;
    }

    let ip = if ip.contains(',') {
        ip.split(',').next().unwrap_or_default().trim().to_string()
    } else {
        ip.to_string()
    };

    ip.parse().ok()
}

#[inline]
pub fn slice_up_to(s: &str, max_len: usize) -> &str {
    if max_len >= s.len() {
        return s;
    }

    let mut idx = max_len;
    while !s.is_char_boundary(idx) {
        idx -= 1;
    }

    &s[..idx]
}

#[inline]
pub fn extract_fields<T: Serialize>(data: T, fields: &[impl AsRef<str>]) -> serde_json::Value {
    let data = serde_json::to_value(data).unwrap();
    if fields.is_empty() {
        return data;
    }

    let mut result = serde_json::Map::new();

    if let serde_json::Value::Object(map) = data {
        for (key, value) in map {
            if fields.iter().any(|f| f.as_ref() == key) {
                result.insert(key, value);
            }
        }
    }

    serde_json::Value::Object(result)
}

#[inline]
pub fn validate_data<T: Validate>(data: &T) -> Result<(), Vec<String>>
where
    T::Context: Default,
{
    if let Err(err) = data.validate() {
        let error_messages = flatten_validation_errors(&err);

        return Err(error_messages);
    }

    Ok(())
}

pub fn flatten_validation_errors(errors: &garde::Report) -> Vec<String> {
    let mut messages = Vec::new();

    for (path, error) in errors.iter() {
        let full_name = path.to_compact_string();

        messages.push(format!("{full_name}: {}", error.message()));
    }

    messages
}
