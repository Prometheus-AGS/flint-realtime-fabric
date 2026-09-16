use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, SecondsFormat, Utc};
use pg_walstream::ColumnValue;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::model::{CanonicalValue, EntityKey, KeyPart};

const PROTOBUF_MIN_SECONDS: i64 = -62_135_596_800;
const PROTOBUF_MAX_SECONDS: i64 = 253_402_300_799;
const MAX_DECIMAL_TEXT_LEN: usize = 200_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceType {
    Bool,
    SignedInteger,
    UnsignedInteger,
    Float,
    Text,
    Bytes,
    Decimal,
    Uuid,
    Timestamp,
    Date,
    Time,
    TimestampWithoutZone,
    Json,
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CanonicalError {
    #[error("column '{column}' has NULL in a required value")]
    NullRequired { column: String },
    #[error("column '{column}' has unsupported binary/text representation")]
    Representation { column: String },
    #[error("column '{column}' contains an invalid {kind}")]
    Invalid { column: String, kind: &'static str },
    #[error("JSON canonicalization failed for column '{column}': {source}")]
    Json {
        column: String,
        source: serde_json::Error,
    },
    #[error("canonical key cannot be empty")]
    EmptyKey,
    #[error("canonical key serialization failed: {0}")]
    KeySerialization(serde_json::Error),
}

pub(crate) fn canonical_value(
    column: &str,
    source_type: SourceType,
    value: &ColumnValue,
) -> Result<CanonicalValue, CanonicalError> {
    if value.is_null() {
        return Ok(CanonicalValue::Null);
    }
    if source_type == SourceType::Bytes {
        return canonical_bytes(column, value).map(CanonicalValue::Bytes);
    }
    let text = value
        .as_str()
        .ok_or_else(|| CanonicalError::Representation {
            column: column.to_owned(),
        })?;
    let invalid = |kind, _value: &str| CanonicalError::Invalid {
        column: column.to_owned(),
        kind,
    };
    match source_type {
        SourceType::Bool => match text {
            "t" | "true" => Ok(CanonicalValue::Bool(true)),
            "f" | "false" => Ok(CanonicalValue::Bool(false)),
            other => Err(invalid("boolean", other)),
        },
        SourceType::SignedInteger => text
            .parse::<i64>()
            .map(CanonicalValue::SignedInteger)
            .map_err(|_| invalid("signed integer", text)),
        SourceType::UnsignedInteger => text
            .parse::<u64>()
            .map(CanonicalValue::UnsignedInteger)
            .map_err(|_| invalid("unsigned integer", text)),
        SourceType::Float => {
            let number = text
                .parse::<f64>()
                .map_err(|_| invalid("floating-point value", text))?;
            if number.is_finite() {
                Ok(CanonicalValue::Float(number))
            } else {
                Err(invalid("finite floating-point value", text))
            }
        }
        SourceType::Text => Ok(CanonicalValue::Text(text.to_owned())),
        SourceType::Decimal => normalize_decimal(text)
            .map(CanonicalValue::Decimal)
            .ok_or_else(|| invalid("decimal", text)),
        SourceType::Uuid => uuid::Uuid::parse_str(text)
            .map(|value| CanonicalValue::Uuid(value.hyphenated().to_string()))
            .map_err(|_| invalid("UUID", text)),
        SourceType::Timestamp => {
            let value = parse_timestamp(text)
                .filter(|value| {
                    (PROTOBUF_MIN_SECONDS..=PROTOBUF_MAX_SECONDS).contains(&value.timestamp())
                })
                .ok_or_else(|| invalid("protobuf-range timestamp with time zone", text))?;
            Ok(CanonicalValue::Timestamp {
                seconds: value.timestamp(),
                nanos: i32::try_from(value.timestamp_subsec_nanos()).unwrap_or_default(),
            })
        }
        SourceType::Date => NaiveDate::parse_from_str(text, "%Y-%m-%d")
            .map(|value| CanonicalValue::Date(value.format("%Y-%m-%d").to_string()))
            .map_err(|_| invalid("date", text)),
        SourceType::Time => parse_time(text)
            .map(CanonicalValue::Time)
            .ok_or_else(|| invalid("time", text)),
        SourceType::TimestampWithoutZone => {
            NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S%.f")
                .map(|value| {
                    CanonicalValue::TimestampWithoutZone(
                        value.format("%Y-%m-%dT%H:%M:%S%.f").to_string(),
                    )
                })
                .map_err(|_| invalid("timestamp without time zone", text))
        }
        SourceType::Json => canonical_json(column, text).map(CanonicalValue::Json),
        SourceType::Bytes => unreachable!("byte values are handled above"),
    }
}

pub(crate) fn canonical_key(parts: Vec<KeyPart>) -> Result<EntityKey, CanonicalError> {
    if parts.is_empty() {
        return Err(CanonicalError::EmptyKey);
    }
    let mut values = Vec::with_capacity(parts.len());
    for part in &parts {
        if matches!(part.value, CanonicalValue::Null) {
            return Err(CanonicalError::NullRequired {
                column: part.column.clone(),
            });
        }
        values.push(json!({
            "column": part.column,
            "kind": kind_name(&part.value),
            "value": canonical_text(&part.value),
        }));
    }
    let bytes =
        serde_json_canonicalizer::to_vec(&values).map_err(CanonicalError::KeySerialization)?;
    Ok(EntityKey {
        parts,
        canonical_id: format!("frfkey:v1:{}", URL_SAFE_NO_PAD.encode(bytes)),
    })
}

pub(crate) fn stable_event_id(
    epoch: &str,
    entity_type: &str,
    key: &str,
    commit_lsn: u64,
    transaction_index: u32,
) -> Result<(String, uuid::Uuid), CanonicalError> {
    let identity = json!({
        "commit_lsn": commit_lsn.to_string(),
        "entity_type": entity_type,
        "epoch": epoch,
        "key": key,
        "transaction_index": transaction_index,
    });
    let bytes =
        serde_json_canonicalizer::to_vec(&identity).map_err(CanonicalError::KeySerialization)?;
    let digest = Sha256::digest(bytes);
    let event_id = format!("frfevent:v1:{digest:x}");
    let mut uuid_bytes = [0_u8; 16];
    uuid_bytes.copy_from_slice(&digest[..16]);
    // RFC 4122 variant/version bits make the internal envelope ID portable.
    uuid_bytes[6] = (uuid_bytes[6] & 0x0f) | 0x80;
    uuid_bytes[8] = (uuid_bytes[8] & 0x3f) | 0x80;
    Ok((event_id, uuid::Uuid::from_bytes(uuid_bytes)))
}

fn canonical_bytes(column: &str, value: &ColumnValue) -> Result<String, CanonicalError> {
    match value {
        ColumnValue::Binary(bytes) => Ok(URL_SAFE_NO_PAD.encode(bytes)),
        ColumnValue::Text(bytes) => {
            let text = std::str::from_utf8(bytes).map_err(|_| CanonicalError::Representation {
                column: column.to_owned(),
            })?;
            let hex = text
                .strip_prefix("\\x")
                .ok_or_else(|| CanonicalError::Invalid {
                    column: column.to_owned(),
                    kind: "bytea",
                })?;
            let decoded = decode_hex(hex).ok_or_else(|| CanonicalError::Invalid {
                column: column.to_owned(),
                kind: "bytea",
            })?;
            Ok(URL_SAFE_NO_PAD.encode(decoded))
        }
        ColumnValue::Null => unreachable!("NULL handled before bytes"),
    }
}

fn canonical_json(column: &str, text: &str) -> Result<String, CanonicalError> {
    let value: Value = serde_json::from_str(text).map_err(|source| CanonicalError::Json {
        column: column.to_owned(),
        source,
    })?;
    let bytes =
        serde_json_canonicalizer::to_vec(&value).map_err(|source| CanonicalError::Json {
            column: column.to_owned(),
            source,
        })?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .or_else(|_| DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f%#z"))
        .map(|value| value.with_timezone(&Utc))
        .ok()
}

fn parse_time(value: &str) -> Option<String> {
    if let Ok(time) = NaiveTime::parse_from_str(value, "%H:%M:%S%.f") {
        return Some(time.format("%H:%M:%S%.f").to_string());
    }
    // `timetz` has no date, so attach a fixed date solely to validate its time
    // and offset while retaining PostgreSQL's explicit offset text.
    let compact = value.replace(' ', "");
    let candidate = format!("1970-01-01T{compact}");
    DateTime::parse_from_str(&candidate, "%Y-%m-%dT%H:%M:%S%.f%#z")
        .ok()
        .map(|_| compact)
}

fn normalize_decimal(value: &str) -> Option<String> {
    if value.len() > MAX_DECIMAL_TEXT_LEN {
        return None;
    }
    let (negative, unsigned) = value
        .strip_prefix('-')
        .map_or((false, value), |rest| (true, rest));
    let (mantissa, exponent) =
        unsigned
            .split_once(['e', 'E'])
            .map_or((unsigned, 0_i64), |(m, e)| {
                e.parse::<i64>()
                    .map_or(("", i64::MIN), |parsed| (m, parsed))
            });
    if mantissa.is_empty() || exponent == i64::MIN {
        return None;
    }
    let mut digits = String::new();
    let mut fractional = 0_i64;
    let mut seen_dot = false;
    for character in mantissa.chars() {
        match character {
            '0'..='9' => {
                digits.push(character);
                if seen_dot {
                    fractional += 1;
                }
            }
            '.' if !seen_dot => seen_dot = true,
            _ => return None,
        }
    }
    if digits.is_empty() {
        return None;
    }
    let scale = fractional.checked_sub(exponent)?;
    let mut result = if scale <= 0 {
        let zeros = usize::try_from(-scale).ok()?;
        if digits.len().checked_add(zeros)? > MAX_DECIMAL_TEXT_LEN {
            return None;
        }
        digits.push_str(&"0".repeat(zeros));
        digits
    } else {
        let scale = usize::try_from(scale).ok()?;
        if scale >= digits.len() {
            if scale.checked_add(2)? > MAX_DECIMAL_TEXT_LEN {
                return None;
            }
            format!("0.{}{}", "0".repeat(scale - digits.len()), digits)
        } else {
            let split = digits.len() - scale;
            format!("{}.{}", &digits[..split], &digits[split..])
        }
    };
    if let Some((whole, fraction)) = result.split_once('.') {
        let whole = whole.trim_start_matches('0');
        let fraction = fraction.trim_end_matches('0');
        let whole = if whole.is_empty() { "0" } else { whole };
        result = if fraction.is_empty() {
            whole.to_owned()
        } else {
            format!("{whole}.{fraction}")
        };
    } else {
        let trimmed = result.trim_start_matches('0');
        result = if trimmed.is_empty() { "0" } else { trimmed }.to_owned();
    }
    if negative && result != "0" {
        result.insert(0, '-');
    }
    Some(result)
}

fn kind_name(value: &CanonicalValue) -> &'static str {
    match value {
        CanonicalValue::Null => "null",
        CanonicalValue::Bool(_) => "bool",
        CanonicalValue::SignedInteger(_) => "signed_integer",
        CanonicalValue::UnsignedInteger(_) => "unsigned_integer",
        CanonicalValue::Float(_) => "float",
        CanonicalValue::Text(_) => "text",
        CanonicalValue::Bytes(_) => "bytes",
        CanonicalValue::Decimal(_) => "decimal",
        CanonicalValue::Uuid(_) => "uuid",
        CanonicalValue::Timestamp { .. } => "timestamp",
        CanonicalValue::Date(_) => "date",
        CanonicalValue::Time(_) => "time",
        CanonicalValue::TimestampWithoutZone(_) => "timestamp_without_zone",
        CanonicalValue::Json(_) => "json",
        _ => unreachable!("future canonical values require an explicit source mapping"),
    }
}

fn canonical_text(value: &CanonicalValue) -> String {
    match value {
        CanonicalValue::Null => "null".to_owned(),
        CanonicalValue::Bool(value) => value.to_string(),
        CanonicalValue::SignedInteger(value) => value.to_string(),
        CanonicalValue::UnsignedInteger(value) => value.to_string(),
        CanonicalValue::Float(value) => value.to_string(),
        CanonicalValue::Text(value)
        | CanonicalValue::Bytes(value)
        | CanonicalValue::Decimal(value)
        | CanonicalValue::Uuid(value)
        | CanonicalValue::Date(value)
        | CanonicalValue::Time(value)
        | CanonicalValue::TimestampWithoutZone(value)
        | CanonicalValue::Json(value) => value.clone(),
        CanonicalValue::Timestamp { seconds, nanos } => {
            DateTime::from_timestamp(*seconds, u32::try_from(*nanos).unwrap_or_default())
                .map_or_else(
                    || format!("{seconds}.{nanos}"),
                    |value| value.to_rfc3339_opts(SecondsFormat::AutoSi, true),
                )
        }
        _ => unreachable!("future canonical values require an explicit text mapping"),
    }
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(text, 16).ok()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_normalization_removes_exponent_and_redundant_zeroes() {
        assert_eq!(normalize_decimal("001.2300"), Some("1.23".to_owned()));
        assert_eq!(normalize_decimal("-0.000"), Some("0".to_owned()));
        assert_eq!(normalize_decimal("1.2e3"), Some("1200".to_owned()));
        assert_eq!(normalize_decimal("12e-3"), Some("0.012".to_owned()));
        assert_eq!(normalize_decimal("1e999999999"), None);
    }

    #[test]
    fn time_with_zone_requires_a_valid_explicit_offset() {
        assert_eq!(
            parse_time("12:34:56.25+05:30"),
            Some("12:34:56.25+05:30".to_owned())
        );
        assert_eq!(parse_time("12:34:56garbage"), None);
    }

    #[test]
    fn canonical_composite_key_preserves_type_and_boundaries() {
        let key = canonical_key(vec![
            KeyPart {
                column: "region".to_owned(),
                value: CanonicalValue::Text("us".to_owned()),
            },
            KeyPart {
                column: "number".to_owned(),
                value: CanonicalValue::SignedInteger(42),
            },
        ])
        .expect("canonical key");
        assert!(key.canonical_id.starts_with("frfkey:v1:"));
        assert_eq!(key.parts.len(), 2);
    }

    #[test]
    fn stable_event_identity_uses_source_position() {
        let first =
            stable_event_id("epoch", "public.item@default", "key", 50, 0).expect("event identity");
        let retry =
            stable_event_id("epoch", "public.item@default", "key", 50, 0).expect("event identity");
        let next =
            stable_event_id("epoch", "public.item@default", "key", 50, 1).expect("event identity");
        assert_eq!(first, retry);
        assert_ne!(first, next);
    }
}
