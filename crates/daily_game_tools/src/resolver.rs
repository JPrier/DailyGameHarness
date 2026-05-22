use anyhow::{bail, Result};
use chrono::NaiveDate;
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", tag = "mode")]
pub enum PuzzleResolverConfig {
    #[serde(rename = "static-pool")]
    #[serde(rename_all = "camelCase")]
    StaticPool {
        timezone: String,
        start_date: String,
        pool_versions: Vec<PoolVersion>,
    },
    #[serde(rename = "dated-files")]
    #[serde(rename_all = "camelCase")]
    DatedFiles {
        timezone: String,
        path_pattern: String,
        asset_path_pattern: Option<String>,
    },
    #[serde(rename = "date-index")]
    #[serde(rename_all = "camelCase")]
    DateIndex {
        timezone: String,
        index_path: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolVersion {
    pub version: String,
    pub start_date: String,
    pub pool_size: u32,
    pub path_pattern: String,
    pub asset_path_pattern: Option<String>,
    pub selector: Selector,
    pub cycle_policy: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Selector {
    #[serde(rename = "affine-permutation")]
    AffinePermutation { a: u32, b: u32 },
    #[serde(rename = "seeded-shuffle")]
    SeededShuffle { seed: String },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedPuzzle {
    Path(String),
    Unavailable,
}

pub fn validate_resolver(config: &PuzzleResolverConfig) -> Result<()> {
    match config {
        PuzzleResolverConfig::StaticPool {
            timezone,
            start_date,
            pool_versions,
        } => {
            validate_timezone(timezone)?;
            parse_date(start_date)?;
            if pool_versions.is_empty() {
                bail!("static-pool requires at least one pool version");
            }
            let mut starts = std::collections::HashSet::new();
            for version in pool_versions {
                parse_date(&version.start_date)?;
                if !starts.insert(version.start_date.clone()) {
                    bail!("overlapping pool versions with same start date");
                }
                if version.version.is_empty() {
                    bail!("missing pool version");
                }
                if version.pool_size == 0 {
                    bail!("pool size must be positive");
                }
                if version.path_pattern.is_empty() {
                    bail!("missing path pattern");
                }
                if version
                    .asset_path_pattern
                    .as_deref()
                    .is_some_and(str::is_empty)
                {
                    bail!("asset path pattern cannot be empty");
                }
                if !matches!(
                    version.cycle_policy.as_str(),
                    "repeat" | "error-after-exhaustion" | "next-version-required"
                ) {
                    bail!("unsupported cycle policy");
                }
                match &version.selector {
                    Selector::AffinePermutation { a, .. } => {
                        if *a == 0 || gcd(*a, version.pool_size) != 1 {
                            bail!("affine selector a must be coprime with pool size");
                        }
                    }
                    Selector::SeededShuffle { seed } if seed.is_empty() => {
                        bail!("seeded shuffle requires seed");
                    }
                    Selector::SeededShuffle { .. } => {}
                }
            }
        }
        PuzzleResolverConfig::DatedFiles {
            timezone,
            path_pattern,
            asset_path_pattern,
        } => {
            validate_timezone(timezone)?;
            if path_pattern.is_empty() {
                bail!("missing path pattern");
            }
            if asset_path_pattern.as_deref().is_some_and(str::is_empty) {
                bail!("asset path pattern cannot be empty");
            }
        }
        PuzzleResolverConfig::DateIndex {
            timezone,
            index_path,
        } => {
            validate_timezone(timezone)?;
            if index_path.is_empty() {
                bail!("missing index path");
            }
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub fn resolve_puzzle(config: &PuzzleResolverConfig, date: &str) -> Result<ResolvedPuzzle> {
    parse_date(date)?;
    match config {
        PuzzleResolverConfig::StaticPool { pool_versions, .. } => {
            let requested = parse_date(date)?;
            let mut versions = pool_versions.clone();
            versions.sort_by_key(|version| parse_date(&version.start_date).expect("validated"));
            let Some(version) = versions
                .into_iter()
                .filter(|version| parse_date(&version.start_date).expect("validated") <= requested)
                .next_back()
            else {
                return Ok(ResolvedPuzzle::Unavailable);
            };
            let start = parse_date(&version.start_date)?;
            let day_offset = (requested - start).num_days();
            if day_offset < 0 {
                return Ok(ResolvedPuzzle::Unavailable);
            }
            let offset = day_offset as u32;
            let cycle_offset = match version.cycle_policy.as_str() {
                "repeat" => offset % version.pool_size,
                "error-after-exhaustion" | "next-version-required"
                    if offset >= version.pool_size =>
                {
                    return Ok(ResolvedPuzzle::Unavailable);
                }
                _ => offset,
            };
            let index = match version.selector {
                Selector::AffinePermutation { a, b } => (a * cycle_offset + b) % version.pool_size,
                Selector::SeededShuffle { .. } => cycle_offset % version.pool_size,
            };
            Ok(ResolvedPuzzle::Path(format_pattern(
                &version.path_pattern,
                &version.version,
                index,
                date,
            )))
        }
        PuzzleResolverConfig::DatedFiles { path_pattern, .. } => Ok(ResolvedPuzzle::Path(
            format_pattern(path_pattern, "", 0, date),
        )),
        PuzzleResolverConfig::DateIndex { .. } => Ok(ResolvedPuzzle::Unavailable),
    }
}

pub fn format_pattern(pattern: &str, version: &str, index: u32, date: &str) -> String {
    pattern
        .replace("{version}", version)
        .replace("{date}", date)
        .replace("{index:04}", &format!("{index:04}"))
        .replace("{index}", &index.to_string())
}

fn validate_timezone(timezone: &str) -> Result<()> {
    if timezone.is_empty() {
        bail!("timezone is required");
    }
    Ok(())
}

fn parse_date(date: &str) -> Result<NaiveDate> {
    Ok(NaiveDate::parse_from_str(date, "%Y-%m-%d")?)
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn static_pool(pool_size: u32, a: u32, cycle_policy: &str) -> PuzzleResolverConfig {
        PuzzleResolverConfig::StaticPool {
            timezone: "America/New_York".into(),
            start_date: "2026-01-01".into(),
            pool_versions: vec![PoolVersion {
                version: "v1".into(),
                start_date: "2026-01-01".into(),
                pool_size,
                path_pattern: "content/puzzles/{version}/puzzle-{index:04}.json".into(),
                asset_path_pattern: Some("content/assets/{version}/puzzle-{index:04}/".into()),
                selector: Selector::AffinePermutation { a, b: 1 },
                cycle_policy: cycle_policy.into(),
            }],
        }
    }

    #[test]
    fn accepts_supported_resolver_modes() {
        assert!(validate_resolver(&static_pool(3, 2, "repeat")).is_ok());
        assert!(validate_resolver(&PuzzleResolverConfig::DatedFiles {
            timezone: "America/New_York".into(),
            path_pattern: "content/puzzles/{date}.json".into(),
            asset_path_pattern: None,
        })
        .is_ok());
        assert!(validate_resolver(&PuzzleResolverConfig::DateIndex {
            timezone: "America/New_York".into(),
            index_path: "content/date-index.json".into(),
        })
        .is_ok());
    }

    #[test]
    fn rejects_invalid_static_pool_config() {
        assert!(validate_resolver(&static_pool(0, 1, "repeat")).is_err());
        assert!(validate_resolver(&static_pool(4, 2, "repeat")).is_err());
        assert!(validate_resolver(&static_pool(3, 2, "bad-policy")).is_err());
    }

    #[test]
    fn affine_resolver_is_stable_unique_and_repeats() {
        let cfg = static_pool(3, 2, "repeat");
        assert_eq!(
            resolve_puzzle(&cfg, "2026-01-01").unwrap(),
            ResolvedPuzzle::Path("content/puzzles/v1/puzzle-0001.json".into())
        );
        assert_eq!(
            resolve_puzzle(&cfg, "2026-01-02").unwrap(),
            ResolvedPuzzle::Path("content/puzzles/v1/puzzle-0000.json".into())
        );
        assert_eq!(
            resolve_puzzle(&cfg, "2026-01-03").unwrap(),
            ResolvedPuzzle::Path("content/puzzles/v1/puzzle-0002.json".into())
        );
        assert_eq!(
            resolve_puzzle(&cfg, "2026-01-04").unwrap(),
            ResolvedPuzzle::Path("content/puzzles/v1/puzzle-0001.json".into())
        );
    }

    #[test]
    fn exhaustion_can_return_unavailable() {
        let cfg = static_pool(3, 2, "error-after-exhaustion");
        assert_eq!(
            resolve_puzzle(&cfg, "2026-01-04").unwrap(),
            ResolvedPuzzle::Unavailable
        );
    }

    #[test]
    fn versioned_pools_preserve_old_mappings() {
        let mut cfg = static_pool(3, 2, "repeat");
        if let PuzzleResolverConfig::StaticPool { pool_versions, .. } = &mut cfg {
            pool_versions.push(PoolVersion {
                version: "v2".into(),
                start_date: "2026-01-03".into(),
                pool_size: 5,
                path_pattern: "content/puzzles/{version}/puzzle-{index:04}.json".into(),
                asset_path_pattern: None,
                selector: Selector::AffinePermutation { a: 2, b: 0 },
                cycle_policy: "repeat".into(),
            });
        }
        assert_eq!(
            resolve_puzzle(&cfg, "2026-01-02").unwrap(),
            ResolvedPuzzle::Path("content/puzzles/v1/puzzle-0000.json".into())
        );
        assert_eq!(
            resolve_puzzle(&cfg, "2026-01-03").unwrap(),
            ResolvedPuzzle::Path("content/puzzles/v2/puzzle-0000.json".into())
        );
    }
}
