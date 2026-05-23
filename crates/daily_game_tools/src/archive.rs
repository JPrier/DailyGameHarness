use anyhow::{bail, Result};
use chrono::{Duration, NaiveDate};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", tag = "mode")]
pub enum ArchiveConfig {
    #[serde(rename = "rolling-window")]
    #[serde(rename_all = "camelCase")]
    RollingWindow {
        days: u32,
        include_today: bool,
        allow_future_dates: bool,
        direct_access: Option<String>,
    },
    #[serde(rename = "fixed-list")]
    #[serde(rename_all = "camelCase")]
    FixedList {
        dates: Vec<String>,
        allow_future_dates: Option<bool>,
        direct_access: Option<String>,
    },
    #[serde(rename = "all-published")]
    #[serde(rename_all = "camelCase")]
    AllPublished {
        allow_future_dates: Option<bool>,
        direct_access: Option<String>,
    },
    #[serde(rename = "disabled")]
    #[serde(rename_all = "camelCase")]
    Disabled { direct_access: Option<String> },
}

pub fn validate_archive(config: &ArchiveConfig) -> Result<()> {
    match config {
        ArchiveConfig::RollingWindow {
            days,
            direct_access,
            ..
        } => {
            if *days == 0 {
                bail!("rolling archive days must be positive");
            }
            validate_direct_access(direct_access.as_deref())?;
        }
        ArchiveConfig::FixedList {
            dates,
            direct_access,
            ..
        } => {
            for date in dates {
                parse_date(date)?;
            }
            validate_direct_access(direct_access.as_deref())?;
        }
        ArchiveConfig::AllPublished { direct_access, .. }
        | ArchiveConfig::Disabled { direct_access } => {
            validate_direct_access(direct_access.as_deref())?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub fn rolling_dates(config: &ArchiveConfig, today: &str) -> Result<Vec<String>> {
    let today = parse_date(today)?;
    match config {
        ArchiveConfig::RollingWindow {
            days,
            include_today,
            ..
        } => {
            let end = if *include_today {
                today
            } else {
                today - Duration::days(1)
            };
            Ok((0..*days)
                .map(|offset| (end - Duration::days(i64::from(offset))).to_string())
                .collect())
        }
        ArchiveConfig::FixedList { dates, .. } => Ok(dates.clone()),
        ArchiveConfig::AllPublished { .. } | ArchiveConfig::Disabled { .. } => Ok(Vec::new()),
    }
}

#[allow(dead_code)]
pub fn date_allowed(config: &ArchiveConfig, today: &str, date: &str) -> Result<bool> {
    let today_date = parse_date(today)?;
    let selected = parse_date(date)?;
    let allow_future = match config {
        ArchiveConfig::RollingWindow {
            allow_future_dates, ..
        } => *allow_future_dates,
        ArchiveConfig::FixedList {
            allow_future_dates, ..
        }
        | ArchiveConfig::AllPublished {
            allow_future_dates, ..
        } => allow_future_dates.unwrap_or(false),
        ArchiveConfig::Disabled { .. } => false,
    };
    if !allow_future && selected > today_date {
        return Ok(false);
    }
    let direct = match config {
        ArchiveConfig::RollingWindow { direct_access, .. }
        | ArchiveConfig::FixedList { direct_access, .. }
        | ArchiveConfig::AllPublished { direct_access, .. }
        | ArchiveConfig::Disabled { direct_access } => {
            direct_access.as_deref().unwrap_or("within-archive-window")
        }
    };
    match direct {
        "any-resolvable-date" => Ok(true),
        "within-archive-window" => Ok(rolling_dates(config, today)?.contains(&date.to_string())),
        "disabled" => Ok(false),
        _ => Ok(false),
    }
}

fn validate_direct_access(value: Option<&str>) -> Result<()> {
    if let Some(value) = value {
        if !matches!(
            value,
            "within-archive-window" | "any-resolvable-date" | "disabled"
        ) {
            bail!("unsupported direct access mode");
        }
    }
    Ok(())
}

fn parse_date(date: &str) -> Result<NaiveDate> {
    Ok(NaiveDate::parse_from_str(date, "%Y-%m-%d")?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_archive_modes() {
        assert!(validate_archive(&ArchiveConfig::RollingWindow {
            days: 30,
            include_today: true,
            allow_future_dates: false,
            direct_access: Some("within-archive-window".into()),
        })
        .is_ok());
        assert!(validate_archive(&ArchiveConfig::FixedList {
            dates: vec!["2026-01-01".into()],
            allow_future_dates: None,
            direct_access: None,
        })
        .is_ok());
        assert!(validate_archive(&ArchiveConfig::Disabled {
            direct_access: Some("disabled".into())
        })
        .is_ok());
    }

    #[test]
    fn rejects_bad_archive_configs() {
        assert!(validate_archive(&ArchiveConfig::RollingWindow {
            days: 0,
            include_today: true,
            allow_future_dates: false,
            direct_access: None,
        })
        .is_err());
        assert!(validate_archive(&ArchiveConfig::FixedList {
            dates: vec!["not-a-date".into()],
            allow_future_dates: None,
            direct_access: None,
        })
        .is_err());
        assert!(validate_archive(&ArchiveConfig::Disabled {
            direct_access: Some("bad".into())
        })
        .is_err());
    }

    #[test]
    fn rolling_window_computes_expected_dates() {
        let cfg = ArchiveConfig::RollingWindow {
            days: 3,
            include_today: true,
            allow_future_dates: false,
            direct_access: None,
        };
        assert_eq!(
            rolling_dates(&cfg, "2026-05-22").unwrap(),
            vec!["2026-05-22", "2026-05-21", "2026-05-20"]
        );
    }

    #[test]
    fn include_today_false_ends_yesterday() {
        let cfg = ArchiveConfig::RollingWindow {
            days: 2,
            include_today: false,
            allow_future_dates: false,
            direct_access: None,
        };
        assert_eq!(
            rolling_dates(&cfg, "2026-05-22").unwrap(),
            vec!["2026-05-21", "2026-05-20"]
        );
    }

    #[test]
    fn direct_access_policy_is_enforced() {
        let cfg = ArchiveConfig::RollingWindow {
            days: 2,
            include_today: true,
            allow_future_dates: false,
            direct_access: Some("within-archive-window".into()),
        };
        assert!(date_allowed(&cfg, "2026-05-22", "2026-05-21").unwrap());
        assert!(!date_allowed(&cfg, "2026-05-22", "2026-05-01").unwrap());
        assert!(!date_allowed(&cfg, "2026-05-22", "2026-05-23").unwrap());
    }
}
