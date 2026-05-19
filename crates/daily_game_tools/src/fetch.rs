use crate::config::{validate_harness_config, Source};
use crate::lockfile::{LockFile, LockGame};
use anyhow::Result;

pub fn sync_games(harness: &str) -> Result<()> {
    let cfg = validate_harness_config(harness)?;
    std::fs::create_dir_all(".harness/external-games")?;
    let mut games = Vec::new();
    for g in cfg.games {
        match g.source {
            Source::Local { path } => {
                let abs = std::path::Path::new(&path).canonicalize()?;
                games.push(LockGame {
                    source_type: "local".into(),
                    repo: None,
                    requested_ref: None,
                    resolved_sha: None,
                    local_path: abs.display().to_string(),
                });
            }
            Source::Git { repo, r#ref } => {
                let safe = repo
                    .split('/')
                    .next_back()
                    .unwrap_or("game")
                    .replace(".git", "");
                let local = format!(".harness/external-games/{safe}");
                if !std::path::Path::new(&local).exists() {
                    std::process::Command::new("git")
                        .args(["clone", &repo, &local])
                        .status()?;
                }
                std::process::Command::new("git")
                    .current_dir(&local)
                    .args(["fetch", "--all", "--tags"])
                    .status()?;
                std::process::Command::new("git")
                    .current_dir(&local)
                    .args(["checkout", &r#ref])
                    .status()?;
                let out = std::process::Command::new("git")
                    .current_dir(&local)
                    .args(["rev-parse", "HEAD"])
                    .output()?;
                let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
                games.push(LockGame {
                    source_type: "git".into(),
                    repo: Some(repo),
                    requested_ref: Some(r#ref),
                    resolved_sha: Some(sha),
                    local_path: local,
                });
            }
        }
    }
    let lock = LockFile {
        schema_version: "daily-game-harness-lock.v1".into(),
        games,
    };
    std::fs::write("harness.lock.json", serde_json::to_string_pretty(&lock)?)?;
    Ok(())
}
