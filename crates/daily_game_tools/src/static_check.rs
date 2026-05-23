use anyhow::{bail, Result};
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

pub fn check_static_output(dist: &str) -> Result<()> {
    let root = Path::new(dist);
    if !root.join("index.html").exists() {
        bail!("missing dist/index.html")
    }
    assert_no_private_files(root)?;
    assert_has_generated_assets(root)?;

    let registry = std::fs::read_to_string("web/src/generated/game-registry.ts")?;
    if registry.contains("/fixtures/") || registry.contains("\\fixtures\\") {
        // Source import paths are allowed in the generated registry for bundling, but public URLs are not.
        for url_key in [
            "contentManifestUrl:",
            "dateIndexUrl:",
            "puzzleBaseUrl:",
            "assetBaseUrl:",
        ] {
            for url in extract_all_urls_after(&registry, url_key)? {
                if url.contains("fixtures") || url.contains(".harness") {
                    bail!("generated public URL points to source package path: {url}");
                }
            }
        }
    }

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

    Ok(())
}

fn assert_no_private_files(dist: &Path) -> Result<()> {
    for entry in WalkDir::new(dist)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
    {
        let path = entry.path().to_string_lossy();
        if path.contains(".git") || path.contains(".harness") {
            bail!("build output contains private file/cache path: {path}");
        }
    }
    Ok(())
}

fn assert_has_generated_assets(dist: &Path) -> Result<()> {
    let mut js = false;
    let mut css = false;
    for entry in WalkDir::new(dist)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
    {
        if entry.path().extension().and_then(|ext| ext.to_str()) == Some("js") {
            js = true;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) == Some("css") {
            css = true;
        }
    }
    if !js || !css {
        bail!("missing generated JS/CSS assets");
    }
    Ok(())
}

fn extract_all_urls_after(content: &str, key: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for line in content.lines() {
        if let Some(idx) = line.find(key) {
            let fragment = &line[idx..];
            if !fragment
                .trim_start_matches(key)
                .trim_start()
                .starts_with("null")
            {
                out.push(extract_url(fragment)?);
            }
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
        std::fs::create_dir_all("/tmp/dgh-dist/games/minimal-text-game").expect("mkdir");
        std::fs::create_dir_all("/tmp/dgh-dist/_games/minimal-text-game/content/puzzles/v1")
            .expect("mkdir");
        std::fs::create_dir_all("/tmp/dgh-dist/_games/minimal-text-game/runtime").expect("mkdir");
        std::fs::create_dir_all("/tmp/dgh-dist/assets").expect("assets");
        std::fs::write("/tmp/dgh-dist/index.html", "ok").expect("write");
        std::fs::write("/tmp/dgh-dist/games/minimal-text-game/index.html", "ok").expect("write");
        std::fs::write(
            "/tmp/dgh-dist/_games/minimal-text-game/content/manifest.json",
            "{}",
        )
        .expect("write");
        std::fs::write(
            "/tmp/dgh-dist/_games/minimal-text-game/runtime/index.js",
            "",
        )
        .expect("write");
        std::fs::write("/tmp/dgh-dist/assets/app.js", "").expect("js");
        std::fs::write("/tmp/dgh-dist/assets/app.css", "").expect("css");

        std::fs::create_dir_all("web/src/generated").expect("mkdir registry");
        std::fs::write(
            "web/src/generated/game-registry.ts",
            "export const generatedGameRegistry = { \"minimal-text-game\": { slug: \"minimal-text-game\", contentManifestUrl: \"/_games/minimal-text-game/content/manifest.json\", dateIndexUrl: null, puzzleBaseUrl: \"/_games/minimal-text-game/content/puzzles\", runtimeAssetBaseUrl: \"/_games/minimal-text-game/runtime\", dates: [] } } as const;",
        )
        .expect("write registry");
        assert!(check_static_output("/tmp/dgh-dist").is_ok());
    }

    #[test]
    fn rejects_private_cache_in_output() {
        std::fs::create_dir_all("/tmp/dgh-private/.harness").expect("mkdir");
        std::fs::write("/tmp/dgh-private/index.html", "ok").expect("index");
        assert!(check_static_output("/tmp/dgh-private").is_err());
    }
}
