use anyhow::{bail, Result};
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessConfig {
    pub schema_version: String,
    pub site: Site,
    pub games: Vec<GameEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub route_prefix: String,
}

#[derive(Debug, Deserialize)]
pub struct GameEntry {
    pub source: Source,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Source {
    #[serde(rename = "local")]
    Local { path: String },
    #[serde(rename = "git")]
    Git { repo: String, r#ref: String },
}

pub fn read(path: &str) -> Result<HarnessConfig> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

pub fn validate_harness_config(path: &str) -> Result<HarnessConfig> {
    let c = read(path)?;
    if c.schema_version != "daily-game-harness.v1" {
        bail!("unsupported schemaVersion")
    }
    if c.games.is_empty() {
        bail!("no games")
    }
    let re = Regex::new(r"^$|^/[a-zA-Z0-9/_-]*$")?;
    if !re.is_match(&c.site.route_prefix) {
        bail!("invalid route prefix")
    }
    for game in &c.games {
        match &game.source {
            Source::Local { path } if path.is_empty() => bail!("missing source path"),
            Source::Git { repo, r#ref } if repo.is_empty() || r#ref.is_empty() => {
                bail!("invalid git source")
            }
            _ => {}
        }
    }
    Ok(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_temp(content: &str) -> String {
        fs::create_dir_all("/tmp").expect("tmp");
        let p = format!(
            "/tmp/harness-config-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        );
        fs::write(&p, content).expect("write config");
        p
    }

    #[test]
    fn accepts_valid_local_game_source() {
        let p = write_temp(
            r#"{"schemaVersion":"daily-game-harness.v1","site":{"routePrefix":""},"games":[{"source":{"type":"local","path":"fixtures/games/minimal-text-game"}}]}"#,
        );
        assert!(validate_harness_config(&p).is_ok());
    }

    #[test]
    fn accepts_valid_git_game_source() {
        let p = write_temp(
            r#"{"schemaVersion":"daily-game-harness.v1","site":{"routePrefix":"/games"},"games":[{"source":{"type":"git","repo":"https://example.com/repo.git","ref":"main"}}]}"#,
        );
        assert!(validate_harness_config(&p).is_ok());
    }

    #[test]
    fn rejects_unknown_source_type() {
        let p = write_temp(
            r#"{"schemaVersion":"daily-game-harness.v1","site":{"routePrefix":""},"games":[{"source":{"type":"zip","path":"x"}}]}"#,
        );
        assert!(validate_harness_config(&p).is_err());
    }

    #[test]
    fn rejects_missing_source_path() {
        let p = write_temp(
            r#"{"schemaVersion":"daily-game-harness.v1","site":{"routePrefix":""},"games":[{"source":{"type":"local","path":""}}]}"#,
        );
        assert!(validate_harness_config(&p).is_err());
    }

    #[test]
    fn rejects_invalid_route_prefix() {
        let p = write_temp(
            r#"{"schemaVersion":"daily-game-harness.v1","site":{"routePrefix":"games"},"games":[{"source":{"type":"local","path":"fixtures/games/minimal-text-game"}}]}"#,
        );
        assert!(validate_harness_config(&p).is_err());
    }
}
