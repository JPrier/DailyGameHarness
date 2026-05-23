use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn validate_json_file(schema_name: &str, json_path: &Path) -> Result<Value> {
    let raw = std::fs::read_to_string(json_path)
        .with_context(|| format!("read JSON for schema validation: {}", json_path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse JSON for schema validation: {}", json_path.display()))?;
    validate_value(schema_name, &value)
        .with_context(|| format!("schema validation failed for {}", json_path.display()))?;
    Ok(value)
}

pub fn validate_value(schema_name: &str, value: &Value) -> Result<()> {
    let schema = load_schema(schema_name)?;
    let validator = jsonschema::validator_for(&schema)
        .with_context(|| format!("compile JSON schema {schema_name}"))?;
    let errors = validator
        .iter_errors(value)
        .map(|error| format!("{}: {}", error.instance_path(), error))
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        bail!(
            "{} failed JSON schema validation:\n{}",
            schema_name,
            errors.join("\n")
        );
    }
    Ok(())
}

fn load_schema(schema_name: &str) -> Result<Value> {
    let path = schemas_root().join(schema_name);
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read JSON schema {}", path.display()))?;
    let mut schema: Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse JSON schema {}", path.display()))?;
    inline_known_refs(&mut schema)?;
    Ok(schema)
}

fn inline_known_refs(value: &mut Value) -> Result<()> {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(reference)) = map.get("$ref") {
                let replacement = match reference.as_str() {
                    "puzzle-resolver.schema.json" => {
                        Some(load_schema("puzzle-resolver.schema.json")?)
                    }
                    "archive-config.schema.json" => {
                        Some(load_schema("archive-config.schema.json")?)
                    }
                    "./game-state.schema.json" | "game-state.schema.json" => {
                        Some(load_schema("game-state.schema.json")?)
                    }
                    _ => None,
                };
                if let Some(replacement) = replacement {
                    *value = replacement;
                    return Ok(());
                }
            }
            for child in map.values_mut() {
                inline_known_refs(child)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                inline_known_refs(child)?;
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
    Ok(())
}

fn schemas_root() -> PathBuf {
    workspace_root().join("schemas")
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_good_content_manifest_schema_with_refs() {
        let value = json!({
            "schemaVersion": "daily-game-content-manifest.v1",
            "gameId": "g",
            "inputModes": ["text"],
            "puzzleResolver": {
                "mode": "dated-files",
                "timezone": "America/New_York",
                "pathPattern": "content/puzzles/{date}.json"
            },
            "archive": {
                "mode": "rolling-window",
                "days": 7,
                "includeToday": true,
                "allowFutureDates": false
            },
            "extension": {}
        });
        validate_value("content-manifest.schema.json", &value).expect("valid manifest");
    }

    #[test]
    fn rejects_bad_package_schema() {
        let value = json!({
            "schemaVersion": "daily-game-package.v1",
            "contractVersion": "daily-game-runtime.v1",
            "game": {"id": "g", "slug": "Bad Slug", "displayName": "Bad"},
            "runtime": {"entry": "dist/runtime/index.js"},
            "ui": {"entry": "src/GameView.svelte"},
            "content": {"manifest": "content/manifest.json", "puzzlesDir": "content/puzzles"},
            "extension": {}
        });
        assert!(validate_value("game-package-config.schema.json", &value).is_err());
    }
}
