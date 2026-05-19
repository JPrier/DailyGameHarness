use crate::config::{validate_harness_config, Source};
use anyhow::{bail, Result};
use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackageConfig {
    game: PackageGame,
}

#[derive(Deserialize)]
struct PackageGame {
    id: String,
}

#[derive(Deserialize)]
struct DateIndex {
    #[serde(rename = "gameId")]
    game_id: String,
    dates: Vec<DateEntry>,
}
#[derive(Deserialize)]
struct DateEntry {
    date: String,
    #[serde(rename = "puzzlePath")]
    puzzle_path: String,
}

pub fn discover_dates(harness: &str) -> Result<()> {
    let cfg = validate_harness_config(harness)?;
    for g in cfg.games {
        if let Source::Local { path } = g.source {
            let root = Path::new(&path);
            let pkg: PackageConfig = serde_json::from_str(&std::fs::read_to_string(
                root.join("daily-game.config.json"),
            )?)?;
            let raw = std::fs::read_to_string(root.join("content/date-index.json"))?;
            let idx: DateIndex = serde_json::from_str(&raw)?;
            if idx.game_id.is_empty() {
                bail!("date-index game id missing");
            }
            if idx.game_id != pkg.game.id {
                bail!("date-index game id mismatch");
            }
            let mut seen = HashSet::new();
            for d in idx.dates {
                NaiveDate::parse_from_str(&d.date, "%Y-%m-%d")?;
                if !seen.insert(d.date.clone()) {
                    bail!("duplicate date");
                }
                if d.puzzle_path.is_empty() {
                    bail!("missing puzzle path");
                }
                if d.puzzle_path.contains("..") {
                    bail!("path traversal");
                }
            }
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
            "/tmp/harness-discover-{}-{}.json",
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

    fn mk_pkg(tmp: &Path, game_id: &str, date_index_json: &str) {
        std::fs::create_dir_all(tmp.join("content")).expect("mkdir content");
        std::fs::write(
            tmp.join("daily-game.config.json"),
            format!("{{\"game\":{{\"id\":\"{}\"}}}}", game_id),
        )
        .expect("write package");
        std::fs::write(tmp.join("content/date-index.json"), date_index_json).expect("write index");
    }

    #[test]
    fn reads_date_index_mode() {
        let fixture = repo_root().join("fixtures/games/minimal-text-game");
        let h = write_harness_for(&fixture.display().to_string());
        assert!(discover_dates(&h).is_ok());
    }

    #[test]
    fn rejects_invalid_date() {
        let tmp = PathBuf::from(format!("/tmp/date-invalid-{}", std::process::id()));
        mk_pkg(
            &tmp,
            "g1",
            r#"{"gameId":"g1","dates":[{"date":"2026-13-99","puzzlePath":"content/puzzles/2026-01-01.json"}]}"#,
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(discover_dates(&h).is_err());
    }

    #[test]
    fn rejects_duplicate_date() {
        let tmp = PathBuf::from(format!("/tmp/date-dup-{}", std::process::id()));
        mk_pkg(
            &tmp,
            "g2",
            r#"{"gameId":"g2","dates":[{"date":"2026-01-01","puzzlePath":"content/puzzles/a.json"},{"date":"2026-01-01","puzzlePath":"content/puzzles/b.json"}]}"#,
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(discover_dates(&h).is_err());
    }

    #[test]
    fn rejects_missing_puzzle_path() {
        let tmp = PathBuf::from(format!("/tmp/date-missing-path-{}", std::process::id()));
        mk_pkg(
            &tmp,
            "g3",
            r#"{"gameId":"g3","dates":[{"date":"2026-01-01","puzzlePath":""}]}"#,
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(discover_dates(&h).is_err());
    }

    #[test]
    fn rejects_path_traversal() {
        let tmp = PathBuf::from(format!("/tmp/date-traversal-{}", std::process::id()));
        mk_pkg(
            &tmp,
            "g4",
            r#"{"gameId":"g4","dates":[{"date":"2026-01-01","puzzlePath":"../evil.json"}]}"#,
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(discover_dates(&h).is_err());
    }

    #[test]
    fn rejects_date_index_game_id_mismatch() {
        let tmp = PathBuf::from(format!("/tmp/date-mismatch-{}", std::process::id()));
        mk_pkg(
            &tmp,
            "package-game",
            r#"{"gameId":"other-game","dates":[{"date":"2026-01-01","puzzlePath":"content/puzzles/2026-01-01.json"}]}"#,
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(discover_dates(&h).is_err());
    }
}
