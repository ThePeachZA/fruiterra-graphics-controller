use std::collections::HashMap;

use serde_json::{Value, json};

use crate::{AppState, CoreError, SCHEMA_VERSION, validate_and_repair};

type Migration = fn(Value) -> Result<Value, CoreError>;

/// Migrations indexed by the version they upgrade *from*.
///
/// Index 0 upgrades a version-0 document to version 1. Version 0 is the
/// "missing or older schema" case; there is no Electron import. Version 1
/// moves talent roles from the library entry to match assignments.
const MIGRATIONS: &[Migration] = &[migrate_v0_to_v1, migrate_v1_to_v2];

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

/// Moves talent roles from `talents[].type` to the six match assignments.
///
/// Slot defaults preserve the old panel/interviewer meaning when no legacy
/// type is recorded. The obsolete per-person `type` is removed so a later save
/// cannot reintroduce it.
fn migrate_v1_to_v2(mut value: Value) -> Result<Value, CoreError> {
    let legacy_roles = legacy_talent_roles(&value);
    let object = value
        .as_object_mut()
        .ok_or_else(|| CoreError::InvalidField {
            path: "$".to_owned(),
            message: "expected a JSON object".to_owned(),
        })?;
    object.insert("schemaVersion".to_owned(), json!(2));

    if let Some(talents) = object.get_mut("talents").and_then(Value::as_array_mut) {
        for talent in talents {
            if let Some(talent) = talent.as_object_mut() {
                talent.remove("type");
            }
        }
    }

    migrate_match_talents(object.get_mut("currentMatch"), &legacy_roles);
    if let Some(matches) = object.get_mut("matches").and_then(Value::as_array_mut) {
        for match_state in matches {
            migrate_match_talents(Some(match_state), &legacy_roles);
        }
    }

    Ok(value)
}

/// Collects the normalised legacy role for each talent identifier.
fn legacy_talent_roles(value: &Value) -> HashMap<String, &'static str> {
    let Some(talents) = value.get("talents").and_then(Value::as_array) else {
        return HashMap::new();
    };
    talents
        .iter()
        .filter_map(|talent| {
            let id = talent.get("id")?.as_str()?;
            let role = match talent.get("type").and_then(Value::as_str)? {
                "panelist" => "analyst",
                "observer" => "other",
                "caster" => "caster",
                "host" => "host",
                "interviewer" => "interviewer",
                "other" => "other",
                _ => return None,
            };
            Some((id.to_owned(), role))
        })
        .collect()
}

/// Adds a `role` to every occupied talent assignment in one match.
fn migrate_match_talents(
    match_state: Option<&mut Value>,
    legacy_roles: &HashMap<String, &'static str>,
) {
    let Some(talent) = match_state.and_then(|value| value.get_mut("talent")) else {
        return;
    };
    let Some(talent) = talent.as_object_mut() else {
        return;
    };

    for (slot, default_role) in [
        ("caster1", "caster"),
        ("caster2", "caster"),
        ("panel1", "analyst"),
        ("panel2", "analyst"),
        ("panel3", "analyst"),
        ("interviewer", "interviewer"),
    ] {
        let Some(assignment) = talent.get_mut(slot) else {
            continue;
        };
        let Some(assignment) = assignment.as_object_mut() else {
            continue;
        };
        let Some(id) = assignment.get("talentId").and_then(Value::as_str) else {
            continue;
        };
        let role = legacy_roles.get(id).copied().unwrap_or(default_role);
        assignment.insert("role".to_owned(), json!(role));
    }
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
