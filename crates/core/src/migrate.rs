use serde_json::{Value, json};

use crate::{AppState, CoreError, SCHEMA_VERSION, validate_and_repair};

type Migration = fn(Value) -> Result<Value, CoreError>;

/// Migrations indexed by the version they upgrade *from*.
///
/// Index 0 upgrades a version-0 document to version 1. Version 0 is the
/// "missing or older schema" case; there is no Electron import.
const MIGRATIONS: &[Migration] = &[migrate_v0_to_v1];

/// Converts a raw JSON document into the current domain model.
///
/// A missing `schemaVersion` is treated as version 0. A version newer than
/// this build is refused rather than silently downgraded. The resulting state
/// is validated and repaired before it is returned.
pub fn migrate(mut raw: Value) -> Result<AppState, CoreError> {
    let found = schema_version(&raw)?;
    if found > SCHEMA_VERSION {
        return Err(CoreError::UnsupportedSchema {
            found,
            supported: SCHEMA_VERSION,
        });
    }

    for from in found..SCHEMA_VERSION {
        let migration = MIGRATIONS.get(from as usize).ok_or(CoreError::Migration {
            from,
            message: "no migration is registered for this version".to_owned(),
        })?;
        raw = migration(raw)?;
    }

    let object = raw.as_object_mut().ok_or_else(|| CoreError::InvalidField {
        path: "$".to_owned(),
        message: "expected a JSON object".to_owned(),
    })?;
    object.insert("schemaVersion".to_owned(), json!(SCHEMA_VERSION));

    let mut state = deserialize_state(raw)?;
    validate_and_repair(&mut state);

    Ok(state)
}

fn schema_version(value: &Value) -> Result<u32, CoreError> {
    match value.get("schemaVersion") {
        None => Ok(0),
        Some(Value::Number(number)) => number
            .as_u64()
            .and_then(|version| u32::try_from(version).ok())
            .ok_or(CoreError::InvalidSchemaVersion),
        Some(_) => Err(CoreError::InvalidSchemaVersion),
    }
}

fn migrate_v0_to_v1(mut value: Value) -> Result<Value, CoreError> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| CoreError::InvalidField {
            path: "$".to_owned(),
            message: "expected a JSON object".to_owned(),
        })?;
    object.insert("schemaVersion".to_owned(), json!(1));
    Ok(value)
}

fn deserialize_state(value: Value) -> Result<AppState, CoreError> {
    let json =
        serde_json::to_string(&value).map_err(|error| CoreError::Serialize(error.to_string()))?;
    let mut deserializer = serde_json::Deserializer::from_str(&json);
    serde_path_to_error::deserialize(&mut deserializer).map_err(|error| CoreError::InvalidField {
        path: error.path().to_string(),
        message: error.into_inner().to_string(),
    })
}
