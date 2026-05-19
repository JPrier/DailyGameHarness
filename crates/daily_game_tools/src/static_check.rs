use anyhow::{bail, Result};
use regex::Regex;
use std::path::Path;

pub fn check_static_output(dist: &str) -> Result<()> {
    let root = Path::new(dist);
    if !root.join("index.html").exists() {
        bail!("missing dist/index.html")
    }

    let registry = std::fs::read_to_string("web/src/generated/game-registry.ts")?;

    for key in [
        "contentManifestUrl:",
        "dateIndexUrl:",
        "puzzleBaseUrl:",
        "runtimeAssetBaseUrl:",
    ] {
        for url in extract_all_urls_after(&registry, key)? {
            assert_exists(root, &url)?;
        }
    }

    for slug in extract_slugs(&registry)? {
        let route = format!("games/{slug}/index.html");
        assert_rel_exists(root, &route)?;
    }

    for (slug, date) in extract_slug_dates(&registry)? {
        let route = format!("games/{slug}/{date}/index.html");
        assert_rel_exists(root, &route)?;

        let puzzle = format!("_games/{slug}/content/puzzles/{date}.json");
        assert_rel_exists(root, &puzzle)?;
    }

    Ok(())
}

fn extract_all_urls_after(content: &str, key: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for line in content.lines() {
        if let Some(idx) = line.find(key) {
            out.push(extract_url(&line[idx..])?);
        }
    }
    Ok(out)
}

fn extract_slugs(content: &str) -> Result<Vec<String>> {
    let re = Regex::new(r#"slug:\s*\"([^\"]+)\""#)?;
    Ok(re
        .captures_iter(content)
        .map(|c| c[1].to_string())
        .collect::<Vec<_>>())
}

fn extract_slug_dates(content: &str) -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    let re = Regex::new(r#"\"([^\"]+)\":\s*\{[^\n]*dates:\s*\[([^\]]*)\]"#)?;
    let re_date = Regex::new(r#"\"([^\"]+)\""#)?;
    for cap in re.captures_iter(content) {
        let slug = cap[1].to_string();
        let dates_blob = cap[2].to_string();
        for d in re_date.captures_iter(&dates_blob) {
            out.push((slug.clone(), d[1].to_string()));
        }
    }
    Ok(out)
}

fn extract_url(fragment: &str) -> Result<String> {
    let first = fragment
        .find('"')
        .ok_or_else(|| anyhow::anyhow!("missing quote"))?;
    let rest = &fragment[first + 1..];
    let last = rest
        .find('"')
        .ok_or_else(|| anyhow::anyhow!("missing quote"))?;
    Ok(rest[..last].to_string())
}

fn assert_exists(dist: &Path, url: &str) -> Result<()> {
    let rel = url.trim_start_matches('/');
    assert_rel_exists(dist, rel)
}

fn assert_rel_exists(dist: &Path, rel: &str) -> Result<()> {
    let p = dist.join(rel);
    if !p.exists() {
        bail!("missing referenced static file: {}", p.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_dist_index_exists() {
        std::fs::create_dir_all("/tmp/dgh-dist/games/minimal-text-game/2026-01-01").expect("mkdir");
        std::fs::create_dir_all("/tmp/dgh-dist/games/minimal-text-game").expect("mkdir");
        std::fs::create_dir_all("/tmp/dgh-dist/_games/minimal-text-game/content/puzzles")
            .expect("mkdir");
        std::fs::create_dir_all("/tmp/dgh-dist/_games/minimal-text-game/runtime").expect("mkdir");
        std::fs::write("/tmp/dgh-dist/index.html", "ok").expect("write");
        std::fs::write("/tmp/dgh-dist/games/minimal-text-game/index.html", "ok").expect("write");
        std::fs::write(
            "/tmp/dgh-dist/games/minimal-text-game/2026-01-01/index.html",
            "ok",
        )
        .expect("write");
        std::fs::write(
            "/tmp/dgh-dist/_games/minimal-text-game/content/manifest.json",
            "{}",
        )
        .expect("write");
        std::fs::write(
            "/tmp/dgh-dist/_games/minimal-text-game/content/date-index.json",
            "{}",
        )
        .expect("write");
        std::fs::write(
            "/tmp/dgh-dist/_games/minimal-text-game/content/puzzles/2026-01-01.json",
            "{}",
        )
        .expect("write");
        std::fs::write(
            "/tmp/dgh-dist/_games/minimal-text-game/runtime/index.js",
            "",
        )
        .expect("write");

        std::fs::create_dir_all("web/src/generated").expect("mkdir registry");
        std::fs::write(
            "web/src/generated/game-registry.ts",
            "export const generatedGameRegistry = { \"minimal-text-game\": { slug: \"minimal-text-game\", contentManifestUrl: \"/_games/minimal-text-game/content/manifest.json\", dateIndexUrl: \"/_games/minimal-text-game/content/date-index.json\", puzzleBaseUrl: \"/_games/minimal-text-game/content/puzzles\", runtimeAssetBaseUrl: \"/_games/minimal-text-game/runtime\", dates: [\"2026-01-01\"] } } as const;",
        )
        .expect("write registry");
        assert!(check_static_output("/tmp/dgh-dist").is_ok());
    }
}
