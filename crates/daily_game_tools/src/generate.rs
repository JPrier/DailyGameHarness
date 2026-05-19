use crate::config::{validate_harness_config, Source};
use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct PkgCfg {
    game: Game,
    runtime: Runtime,
    ui: Ui,
}
#[derive(Deserialize)]
struct Game {
    id: String,
    slug: String,
    #[serde(rename = "displayName")]
    display_name: String,
}
#[derive(Deserialize)]
struct Runtime {
    entry: String,
}
#[derive(Deserialize)]
struct Ui {
    entry: String,
}
#[derive(Deserialize)]
struct DateIndex {
    dates: Vec<DateEntry>,
}
#[derive(Deserialize)]
struct DateEntry {
    date: String,
}

pub fn generate_static_registry(harness: &str) -> Result<()> {
    let cfg = validate_harness_config(harness)?;
    std::fs::create_dir_all("web/src/generated")?;
    let mut imports = String::new();
    let mut body = String::from("export const generatedGameRegistry = {\n");
    let mut idx = 0usize;
    for g in cfg.games {
        let path = match g.source {
            Source::Local { path } => path,
            Source::Git { .. } => continue,
        };
        let pkg: PkgCfg = serde_json::from_str(&std::fs::read_to_string(format!(
            "{path}/daily-game.config.json"
        ))?)?;
        let di: DateIndex = serde_json::from_str(&std::fs::read_to_string(format!(
            "{path}/content/date-index.json"
        ))?)?;
        imports.push_str(&format!(
            "import GameView_{idx} from '../../../{path}/{}';\n",
            pkg.ui.entry
        ));
        imports.push_str(&format!(
            "import {{ createRuntime as createRuntime_{idx} }} from '../../../{path}/{}';\n",
            pkg.runtime.entry
        ));
        let dates = di
            .dates
            .into_iter()
            .map(|d| format!("\"{}\"", d.date))
            .collect::<Vec<_>>()
            .join(", ");
        body.push_str(&format!("  \"{}\": {{ id: \"{}\", slug: \"{}\", displayName: \"{}\", category: \"fixture\", contentManifestUrl: \"/_games/{}/content/manifest.json\", dateIndexUrl: \"/_games/{}/content/date-index.json\", puzzleBaseUrl: \"/_games/{}/content/puzzles\", assetBaseUrl: \"/_games/{}/content/assets\", runtimeAssetBaseUrl: \"/_games/{}/runtime\", GameView: GameView_{idx}, createRuntime: createRuntime_{idx}, dates: [{}] }},\n", pkg.game.slug,pkg.game.id,pkg.game.slug,pkg.game.display_name,pkg.game.slug,pkg.game.slug,pkg.game.slug,pkg.game.slug,pkg.game.slug,dates));
        idx += 1;
    }
    body.push_str("} as const;\n");
    std::fs::write(
        "web/src/generated/game-registry.ts",
        format!("{imports}\n{body}"),
    )?;
    Ok(())
}

pub fn prepare_public_assets(harness: &str) -> Result<()> {
    let cfg = validate_harness_config(harness)?;
    for g in cfg.games {
        let path = match g.source {
            Source::Local { path } => path,
            Source::Git { .. } => continue,
        };
        let pkg: PkgCfg = serde_json::from_str(&std::fs::read_to_string(format!(
            "{path}/daily-game.config.json"
        ))?)?;
        let slug = pkg.game.slug;
        let base = format!("web/public/_games/{slug}");
        std::fs::create_dir_all(format!("{base}/content/puzzles"))?;
        std::fs::create_dir_all(format!("{base}/content/assets"))?;
        std::fs::create_dir_all(format!("{base}/runtime"))?;
        for f in ["manifest.json", "date-index.json"] {
            std::fs::copy(format!("{path}/content/{f}"), format!("{base}/content/{f}"))?;
        }
        let idx: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(format!(
            "{path}/content/date-index.json"
        ))?)?;
        for d in idx["dates"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("bad index"))?
        {
            let pp = d["puzzlePath"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("missing puzzlePath"))?;
            if pp.contains("..") {
                bail!("path traversal");
            }
            std::fs::copy(format!("{path}/{pp}"), format!("{base}/{pp}"))?;
        }
        std::fs::copy(
            format!("{path}/dist/runtime/index.js"),
            format!("{base}/runtime/index.js"),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn repo_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("root")
    }

    fn harness_for_path(path: &str) -> String {
        let p = format!(
            "/tmp/generate-harness-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        );
        let json = format!("{{\"schemaVersion\":\"daily-game-harness.v1\",\"site\":{{\"routePrefix\":\"\"}},\"games\":[{{\"source\":{{\"type\":\"local\",\"path\":\"{}\"}}}}]}}", path);
        std::fs::write(&p, json).expect("harness");
        p
    }

    #[test]
    fn generated_registry_contains_fixture_game_and_static_imports() {
        let fixture = repo_root().join("fixtures/games/minimal-text-game");
        let harness = harness_for_path(&fixture.display().to_string());
        generate_static_registry(&harness).expect("generate");
        let reg = std::fs::read_to_string("web/src/generated/game-registry.ts").expect("read");
        assert!(reg.contains("import GameView_0"));
        assert!(reg.contains("createRuntime_0"));
        assert!(reg.contains("\"minimal-text-game\""));
        assert!(
            reg.contains("contentManifestUrl: \"/_games/minimal-text-game/content/manifest.json\"")
        );
        assert!(reg.contains("dates: [\"2026-01-01\"]"));
    }

    #[test]
    fn public_assets_preparation_copies_manifest_date_index_puzzles_and_runtime() {
        let fixture = repo_root().join("fixtures/games/minimal-text-game");
        let harness = harness_for_path(&fixture.display().to_string());
        prepare_public_assets(&harness).expect("prepare");
        assert!(Path::new("web/public/_games/minimal-text-game/content/manifest.json").exists());
        assert!(Path::new("web/public/_games/minimal-text-game/content/date-index.json").exists());
        assert!(
            Path::new("web/public/_games/minimal-text-game/content/puzzles/2026-01-01.json")
                .exists()
        );
        assert!(Path::new("web/public/_games/minimal-text-game/runtime/index.js").exists());
    }

    #[test]
    fn public_assets_preparation_rejects_path_traversal() {
        let base = format!("/tmp/generate-traversal-{}", std::process::id());
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(format!("{base}/content/puzzles")).expect("mkdir");
        std::fs::create_dir_all(format!("{base}/dist/runtime")).expect("mkdir");
        std::fs::write(
            format!("{base}/daily-game.config.json"),
            r#"{"game":{"id":"g","slug":"traversal-game","displayName":"Traversal Game"},"runtime":{"entry":"dist/runtime/index.js"},"ui":{"entry":"src/GameView.svelte"}}"#,
        )
        .expect("pkg");
        std::fs::write(format!("{base}/content/manifest.json"), "{}").expect("manifest");
        std::fs::write(
            format!("{base}/content/date-index.json"),
            r#"{"dates":[{"date":"2026-01-01","puzzlePath":"../evil.json"}]}"#,
        )
        .expect("index");
        std::fs::write(format!("{base}/dist/runtime/index.js"), "").expect("runtime");

        let harness = format!(
            "/tmp/generate-traversal-harness-{}.json",
            std::process::id()
        );
        std::fs::write(
            &harness,
            format!(
                "{{\"schemaVersion\":\"daily-game-harness.v1\",\"site\":{{\"routePrefix\":\"\"}},\"games\":[{{\"source\":{{\"type\":\"local\",\"path\":\"{}\"}}}}]}}",
                base
            ),
        )
        .expect("harness");

        assert!(prepare_public_assets(&harness).is_err());
    }
}
