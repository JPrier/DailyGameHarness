use crate::config::{validate_harness_config, Source};
use crate::lockfile::{LockFile, LockGame};
use anyhow::{bail, Result};

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
                let safe = safe_git_dir_name(&repo);
                let local = format!(".harness/external-games/{safe}");
                if !std::path::Path::new(&local).exists() {
                    let status = std::process::Command::new("git")
                        .args(["clone", &repo, &local])
                        .status()?;
                    if !status.success() {
                        bail!("git clone failed for {repo}");
                    }
                }
                let status = std::process::Command::new("git")
                    .current_dir(&local)
                    .args(["fetch", "--all", "--tags"])
                    .status()?;
                if !status.success() {
                    bail!("git fetch failed for {repo}");
                }
                let status = std::process::Command::new("git")
                    .current_dir(&local)
                    .args(["checkout", &r#ref])
                    .status()?;
                if !status.success() {
                    bail!("git checkout failed for {repo} at {ref}");
                }
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

fn safe_git_dir_name(repo: &str) -> String {
    repo.replace('\\', "/")
        .split('/')
        .next_back()
        .unwrap_or("game")
        .trim_end_matches(".git")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn write_harness(source: &str) -> String {
        std::fs::create_dir_all("/tmp").expect("tmp");
        let p = format!(
            "/tmp/sync-harness-{}-{}.json",
            std::process::id(),
            rand_suffix()
        );
        std::fs::write(
            &p,
            format!(
                "{{\"schemaVersion\":\"daily-game-harness.v1\",\"site\":{{\"name\":\"Daily Games\",\"baseUrl\":\"https://example.com\",\"routePrefix\":\"\"}},\"games\":[{{\"source\":{source}}}],\"staticGeneration\":{{\"routeMode\":\"single-shell\"}},\"deployment\":{{\"target\":\"github-pages\"}}}}"
            ),
        )
        .expect("harness");
        p
    }

    fn rand_suffix() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    }

    fn run_git(dir: &Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .current_dir(dir)
            .args(args)
            .status()
            .expect("git");
        assert!(status.success(), "git command failed: {args:?}");
    }

    #[test]
    fn local_source_records_resolved_path_and_lockfile_schema() {
        let cwd = std::env::current_dir().expect("cwd");
        let source = format!(
            "{{\"type\":\"local\",\"path\":\"{}\"}}",
            cwd.display().to_string().replace('\\', "\\\\")
        );
        let harness = write_harness(&source);
        sync_games(&harness).expect("sync");
        let raw = std::fs::read_to_string("harness.lock.json").expect("lock");
        let lock: LockFile = serde_json::from_str(&raw).expect("parse lock");
        assert_eq!(lock.schema_version, "daily-game-harness-lock.v1");
        assert_eq!(lock.games[0].source_type, "local");
        assert_eq!(
            lock.games[0].local_path,
            cwd.canonicalize().unwrap().display().to_string()
        );
    }

    #[test]
    fn git_source_records_requested_ref_and_changed_ref_updates_lockfile() {
        let tmp = PathBuf::from(format!(
            "/tmp/sync-git-{}-{}",
            std::process::id(),
            rand_suffix()
        ));
        let repo = tmp.join("repo");
        std::fs::create_dir_all(&repo).expect("mkdir");
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "test@example.com"]);
        run_git(&repo, &["config", "user.name", "Test"]);
        std::fs::write(repo.join("file.txt"), "one").expect("file");
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "one"]);
        run_git(&repo, &["tag", "v1"]);
        std::fs::write(repo.join("file.txt"), "two").expect("file");
        run_git(&repo, &["commit", "-am", "two"]);
        run_git(&repo, &["tag", "v2"]);

        let repo_path = repo
            .canonicalize()
            .unwrap()
            .display()
            .to_string()
            .replace("\\\\?\\", "")
            .replace('\\', "/");
        let repo_url = format!("file:///{repo_path}");
        let h1 = write_harness(&format!(
            "{{\"type\":\"git\",\"repo\":\"{repo_url}\",\"ref\":\"v1\"}}"
        ));
        sync_games(&h1).expect("sync v1");
        let lock1: LockFile =
            serde_json::from_str(&std::fs::read_to_string("harness.lock.json").unwrap()).unwrap();
        let sha1 = lock1.games[0].resolved_sha.clone().unwrap();
        assert_eq!(lock1.games[0].requested_ref.as_deref(), Some("v1"));

        let h2 = write_harness(&format!(
            "{{\"type\":\"git\",\"repo\":\"{repo_url}\",\"ref\":\"v2\"}}"
        ));
        sync_games(&h2).expect("sync v2");
        let lock2: LockFile =
            serde_json::from_str(&std::fs::read_to_string("harness.lock.json").unwrap()).unwrap();
        let sha2 = lock2.games[0].resolved_sha.clone().unwrap();
        assert_eq!(lock2.games[0].requested_ref.as_deref(), Some("v2"));
        assert_ne!(sha1, sha2);
    }
}
