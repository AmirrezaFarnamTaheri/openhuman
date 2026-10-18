//! Enforce JSON object and string shapes before Serde's typed validation.
//! Derived structs and unit enums also accept positional and object encodings,
//! which are outside this packet's documented JSON contract.

use super::types::CodingHandoff;
use serde_json::Value;

pub(super) fn parse(json: &str) -> Result<CodingHandoff, String> {
    let value: Value = serde_json::from_str(json).map_err(contract_error)?;
    if !value.is_object() {
        return Err("invalid JSON contract: packet must be an object".into());
    }
    string_field(&value, "type")?;
    records(&value, "source_refs")?;
    records(&value, "changed_files")?;
    for check in records(&value, "checks")? {
        string_field(check, "status")?;
        records(check, "source_revisions")?;
    }

    // Parse the original bytes again: converting Value into the typed packet
    // would erase duplicate fields before the typed deserializer can reject them.
    serde_json::from_str(json).map_err(contract_error)
}

fn records<'a>(value: &'a Value, field: &str) -> Result<&'a [Value], String> {
    match value.get(field) {
        None => Ok(&[]),
        Some(Value::Array(records)) if records.iter().all(Value::is_object) => Ok(records),
        Some(_) => Err(format!(
            "invalid JSON contract: {field} must be an array of objects"
        )),
    }
}

fn string_field(value: &Value, field: &str) -> Result<(), String> {
    if value.get(field).is_some_and(|value| !value.is_string()) {
        return Err(format!("invalid JSON contract: {field} must be a string"));
    }
    Ok(())
}

fn contract_error(error: serde_json::Error) -> String {
    // Do not echo an unknown field/value that may contain the caller's source.
    format!(
        "invalid JSON contract at line {}, column {}; see docs/specs/coding-handoff.md",
        error.line(),
        error.column()
    )
}
