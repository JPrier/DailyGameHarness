use crate::config::{validate_harness_config, Source};
use anyhow::{anyhow, bail, Result};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GamePackageConfig {
    contract_version: String,
    game: GameMeta,
    runtime: Entry,
    ui: Entry,
    content: Content,
    extension: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct GameMeta {
    id: String,
    slug: String,
}

#[derive(Debug, Deserialize)]
struct Entry {
    entry: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Content {
    manifest: String,
    date_index: String,
}

fn assert_path_within_root(root: &Path, rel: &str, label: &str) -> Result<()> {
    if rel.is_empty() {
        bail!("missing {label}");
    }
    let joined = root.join(rel);
    let canon_parent = joined
        .parent()
        .ok_or_else(|| anyhow!("invalid {label}"))?
        .canonicalize()
        .unwrap_or_else(|_| root.to_path_buf());
    let canon_root = root.canonicalize()?;
    if !canon_parent.starts_with(&canon_root) || rel.contains("..") || rel.starts_with('/') {
        bail!("{label} escapes package root");
    }
    Ok(())
}

pub fn validate_games(harness_path: &str) -> Result<()> {
    let cfg = validate_harness_config(harness_path)?;
    let mut ids = HashSet::new();
    let mut slugs = HashSet::new();
    let slug_re = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")?;
    for game in cfg.games {
        let root = match game.source {
            Source::Local { path } => path,
            Source::Git { .. } => continue,
        };
        let root_path = Path::new(&root);
        let package_path = root_path.join("daily-game.config.json");
        let raw = std::fs::read_to_string(&package_path)?;
        let parsed: GamePackageConfig = serde_json::from_str(&raw)?;
        if parsed.contract_version != "daily-game-runtime.v1" {
            bail!("unsupported contract version")
        }
        if parsed.game.id.is_empty() {
            bail!("missing game id")
        }
        if parsed.game.slug.is_empty() {
            bail!("missing slug")
        }
        if !slug_re.is_match(&parsed.game.slug) {
            bail!("non url safe slug")
        }
        assert_path_within_root(root_path, &parsed.runtime.entry, "runtime entry")?;
        assert_path_within_root(root_path, &parsed.ui.entry, "ui entry")?;
        assert_path_within_root(root_path, &parsed.content.manifest, "content manifest")?;
        assert_path_within_root(root_path, &parsed.content.date_index, "date index")?;

        if !parsed.extension.is_object() {
            bail!("extension must be object")
        }
        if !ids.insert(parsed.game.id) || !slugs.insert(parsed.game.slug) {
            bail!("duplicate game IDs/slugs")
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("root")
    }

    fn write_harness_for(path: &str) -> String {
        let p = format!(
            "/tmp/harness-games-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        );
        let json = format!(
            "{{\"schemaVersion\":\"daily-game-harness.v1\",\"site\":{{\"routePrefix\":\"\"}},\"games\":[{{\"source\":{{\"type\":\"local\",\"path\":\"{}\"}}}}]}}",
            path
        );
        std::fs::write(&p, json).expect("write harness");
        p
    }

    fn write_pkg(dir: &Path, body: &str) {
        std::fs::create_dir_all(dir).expect("mkdir");
        std::fs::write(dir.join("daily-game.config.json"), body).expect("write pkg config");
    }

    #[test]
    fn accepts_valid_fixture_package_config() {
        let fixture = repo_root().join("fixtures/games/minimal-text-game");
        let h = write_harness_for(&fixture.display().to_string());
        assert!(validate_games(&h).is_ok());
    }

    #[test]
    fn rejects_non_url_safe_slug() {
        let tmp = PathBuf::from(format!("/tmp/pkg-slug-{}", std::process::id()));
        write_pkg(
            &tmp,
            r#"{"contractVersion":"daily-game-runtime.v1","game":{"id":"g","slug":"Bad Slug"},"runtime":{"entry":"x"},"ui":{"entry":"x"},"content":{"manifest":"x","dateIndex":"x"},"extension":{}}"#,
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn rejects_unsupported_contract_version() {
        let tmp = PathBuf::from(format!("/tmp/pkg-contract-{}", std::process::id()));
        write_pkg(
            &tmp,
            r#"{"contractVersion":"daily-game-runtime.v2","game":{"id":"g","slug":"good-slug"},"runtime":{"entry":"dist/runtime/index.js"},"ui":{"entry":"src/GameView.svelte"},"content":{"manifest":"content/manifest.json","dateIndex":"content/date-index.json"},"extension":{}}"#,
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn rejects_paths_escaping_package_root() {
        let tmp = PathBuf::from(format!("/tmp/pkg-path-{}", std::process::id()));
        write_pkg(
            &tmp,
            r#"{"contractVersion":"daily-game-runtime.v1","game":{"id":"g2","slug":"good-slug"},"runtime":{"entry":"../evil.js"},"ui":{"entry":"src/GameView.svelte"},"content":{"manifest":"content/manifest.json","dateIndex":"content/date-index.json"},"extension":{}}"#,
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn allows_unknown_fields_inside_extension() {
        let tmp = PathBuf::from(format!("/tmp/pkg-ext-{}", std::process::id()));
        write_pkg(
            &tmp,
            r#"{"contractVersion":"daily-game-runtime.v1","game":{"id":"g3","slug":"ext-slug"},"runtime":{"entry":"dist/runtime/index.js"},"ui":{"entry":"src/GameView.svelte"},"content":{"manifest":"content/manifest.json","dateIndex":"content/date-index.json"},"extension":{"anything":{"goes":true}}}"#,
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_ok());
    }
}
