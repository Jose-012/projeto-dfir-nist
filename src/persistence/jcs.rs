//! Canonicalização JSON (RFC 8785/JCS) compartilhada pela persistência.
//!
//! Inteiros fora de ±2^53 são rejeitados antes da canonicalização para não
//! produzir hashes incompatíveis entre implementações JSON. O formato e os
//! erros permanecem os mesmos das antigas cópias em `audit.rs` e
//! `analytical_results.rs`.

use serde_json::Value;

use super::validation_error;

const UNSAFE_INTEGER: &str =
    "RFC 8785 canonicalization rejects integers outside +/-2^53; encode them as strings";

pub(super) fn ensure_jcs_safe(value: &Value) -> Result<(), sqlx::Error> {
    const MAX_EXACT_INTEGER: i128 = 9_007_199_254_740_992; // 2^53

    match value {
        Value::Number(number) => {
            if let Some(integer) = number.as_i64()
                && i128::from(integer).abs() > MAX_EXACT_INTEGER
            {
                return Err(validation_error(UNSAFE_INTEGER));
            }
            if let Some(integer) = number.as_u64()
                && u128::from(integer) > MAX_EXACT_INTEGER as u128
            {
                return Err(validation_error(UNSAFE_INTEGER));
            }
        }
        Value::Array(values) => {
            for value in values {
                ensure_jcs_safe(value)?;
            }
        }
        Value::Object(map) => {
            for value in map.values() {
                ensure_jcs_safe(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn canonicalize_json(raw_json: &str) -> Result<String, sqlx::Error> {
    let value: Value =
        serde_json::from_str(raw_json).map_err(|error| validation_error(error.to_string()))?;
    ensure_jcs_safe(&value)?;
    serde_json_canonicalizer::to_string(&value).map_err(|error| validation_error(error.to_string()))
}

pub(super) fn canonicalize_value(value: &Value) -> Result<Vec<u8>, sqlx::Error> {
    ensure_jcs_safe(value)?;
    serde_json_canonicalizer::to_vec(value).map_err(|error| validation_error(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{canonicalize_json, canonicalize_value};

    #[test]
    fn canonical_text_and_value_have_the_same_bytes() {
        let raw = r#"{"z":1,"a":[true,"x"]}"#;
        let canonical = canonicalize_json(raw).expect("valid JSON");
        assert_eq!(canonical, r#"{"a":[true,"x"],"z":1}"#);
        let value = serde_json::from_str(raw).expect("test value");
        assert_eq!(
            canonicalize_value(&value).expect("canonical bytes"),
            canonical.as_bytes()
        );
    }

    #[test]
    fn integer_boundaries_and_nested_values_are_enforced() {
        for accepted in ["9007199254740992", "-9007199254740992"] {
            assert!(canonicalize_json(accepted).is_ok(), "accepted: {accepted}");
        }
        for rejected in [
            "9007199254740993",
            "-9007199254740993",
            r#"{"nested":[9007199254740993]}"#,
        ] {
            assert!(canonicalize_json(rejected).is_err(), "rejected: {rejected}");
        }
    }
}
