use crate::validate::{package_roots, read_package_config, resolve_safe_file};
use anyhow::{anyhow, bail, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DateIndex {
    dates: Vec<DateEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DateEntry {
    date: String,
    puzzle_path: String,
}

pub fn generate_static_registry(harness: &str) -> Result<()> {
    let workspace = workspace_root()?;
    std::fs::create_dir_all(workspace.join("web/src/generated"))?;
    let mut imports = String::new();
    let mut body = String::from("export const generatedGameRegistry = {\n");

    for (idx, root) in package_roots(harness)?.into_iter().enumerate() {
        let root = root.canonicalize()?;
        let pkg = read_package_config(&root)?;
        let date_index_path = resolve_safe_file(&root, &pkg.content.date_index, "date index")?;
        let di: DateIndex = serde_json::from_str(&std::fs::read_to_string(date_index_path)?)?;
        let import_root = relative_import_from_web_generated(&root)?;
        imports.push_str(&format!(
            "import GameView_{idx} from '{}';\n",
            js_join(&import_root, &pkg.ui.entry)
        ));
        imports.push_str(&format!(
            "import {{ createRuntime as createRuntime_{idx} }} from '{}';\n",
            js_join(&import_root, &pkg.runtime.entry)
        ));
        let dates = di
            .dates
            .into_iter()
            .map(|d| format!("\"{}\"", escape_ts(&d.date)))
            .collect::<Vec<_>>()
            .join(", ");
        let category = pkg.game.category.as_deref().unwrap_or("fixture");
        body.push_str(&format!(
            "  \"{}\": {{ id: \"{}\", slug: \"{}\", displayName: \"{}\", category: \"{}\", contentManifestUrl: \"/_games/{}/content/manifest.json\", dateIndexUrl: \"/_games/{}/content/date-index.json\", puzzleBaseUrl: \"/_games/{}/content/puzzles\", assetBaseUrl: \"/_games/{}/content/assets\", runtimeAssetBaseUrl: \"/_games/{}/runtime\", GameView: GameView_{idx}, createRuntime: createRuntime_{idx}, dates: [{}] }},\n",
            escape_ts(&pkg.game.slug),
            escape_ts(&pkg.game.id),
            escape_ts(&pkg.game.slug),
            escape_ts(&pkg.game.display_name),
            escape_ts(category),
            escape_ts(&pkg.game.slug),
            escape_ts(&pkg.game.slug),
            escape_ts(&pkg.game.slug),
            escape_ts(&pkg.game.slug),
            escape_ts(&pkg.game.slug),
            dates
        ));
    }
    body.push_str("} as const;\n");
    std::fs::write(
        workspace.join("web/src/generated/game-registry.ts"),
        format!("{imports}\n{body}"),
    )?;
    Ok(())
}

pub fn prepare_public_assets(harness: &str) -> Result<()> {
    let workspace = workspace_root()?;
    let games_public_root = workspace.join("web/public/_games");
    if games_public_root.exists() {
        std::fs::remove_dir_all(&games_public_root)?;
    }
    std::fs::create_dir_all(&games_public_root)?;
    for root in package_roots(harness)? {
        let root = root.canonicalize()?;
        let pkg = read_package_config(&root)?;
        let slug = &pkg.game.slug;
        let base = games_public_root.join(slug);
        std::fs::create_dir_all(base.join("content/puzzles"))?;
        std::fs::create_dir_all(base.join("content/assets"))?;
        std::fs::create_dir_all(base.join("runtime"))?;

        copy_file_checked(
            &root,
            &pkg.content.manifest,
            &base.join("content/manifest.json"),
            "content manifest",
        )?;
        copy_file_checked(
            &root,
            &pkg.content.date_index,
            &base.join("content/date-index.json"),
            "date index",
        )?;

        let idx: DateIndex = serde_json::from_str(&std::fs::read_to_string(
            root.join(&pkg.content.date_index),
        )?)?;
        for entry in idx.dates {
            let source = resolve_safe_file(&root, &entry.puzzle_path, "puzzle path")?;
            let target = base
                .join("content/puzzles")
                .join(format!("{}.json", entry.date));
            std::fs::copy(source, target)?;
        }

        if let Some(assets_dir) = &pkg.content.assets_dir {
            let source = root.join(assets_dir);
            if source.exists() {
                copy_tree_checked(&root, &source, &base.join("content/assets"))?;
            }
        }

        let runtime_source = root
            .join(&pkg.runtime.entry)
            .parent()
            .ok_or_else(|| anyhow!("runtime entry has no parent"))?
            .to_path_buf();
        copy_tree_checked(&root, &runtime_source, &base.join("runtime"))?;
    }
    Ok(())
}

fn relative_import_from_web_generated(root: &Path) -> Result<String> {
    let web_generated = workspace_root()?.join("web/src/generated").canonicalize()?;
    let root = root.canonicalize()?;
    let rel = pathdiff(&root, &web_generated)?;
    Ok(normalize_js_path(&rel))
}

fn pathdiff(path: &Path, base: &Path) -> Result<PathBuf> {
    let path_components = path.components().collect::<Vec<_>>();
    let base_components = base.components().collect::<Vec<_>>();
    let mut common = 0usize;
    while common < path_components.len()
        && common < base_components.len()
        && path_components[common] == base_components[common]
    {
        common += 1;
    }
    let mut out = PathBuf::new();
    for _ in common..base_components.len() {
        out.push("..");
    }
    for comp in &path_components[common..] {
        out.push(comp.as_os_str());
    }
    if out.as_os_str().is_empty() {
        bail!("cannot import generated registry from package root")
    }
    Ok(out)
}

fn js_join(root: &str, rel: &str) -> String {
    format!("{}/{}", root.trim_end_matches('/'), rel.replace('\\', "/"))
}

fn normalize_js_path(path: &Path) -> String {
    let mut out = path.to_string_lossy().replace('\\', "/");
    if !out.starts_with('.') {
        out = format!("./{out}");
    }
    out
}

fn copy_file_checked(root: &Path, rel: &str, target: &Path, label: &str) -> Result<()> {
    let source = resolve_safe_file(root, rel, label)?;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(source, target)?;
    Ok(())
}

fn copy_tree_checked(package_root: &Path, source: &Path, target: &Path) -> Result<()> {
    let root = package_root.canonicalize()?;
    let source = source.canonicalize()?;
    if !source.starts_with(&root) {
        bail!("copy source escapes package root");
    }
    let mut entries = WalkDir::new(&source)
        .sort_by_file_name()
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path().to_path_buf());
    for entry in entries {
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with('.'))
        {
            continue;
        }
        let meta = std::fs::symlink_metadata(entry.path())?;
        if meta.file_type().is_symlink() {
            bail!("refusing to copy symlink: {}", entry.path().display());
        }
        let rel = entry.path().strip_prefix(&source)?;
        let dest = target.join(rel);
        if meta.is_dir() {
            std::fs::create_dir_all(&dest)?;
        } else if meta.is_file() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}

fn escape_ts(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn workspace_root() -> Result<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};

    fn output_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn repo_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("root")
    }

    fn harness_for_paths(paths: &[PathBuf]) -> String {
        std::fs::create_dir_all("/tmp").expect("tmp");
        let p = format!(
            "/tmp/generate-harness-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        );
        let games = paths
            .iter()
            .map(|path| {
                format!(
                    "{{\"source\":{{\"type\":\"local\",\"path\":\"{}\"}}}}",
                    path.display().to_string().replace('\\', "\\\\")
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        std::fs::write(
            &p,
            format!("{{\"schemaVersion\":\"daily-game-harness.v1\",\"site\":{{\"routePrefix\":\"\"}},\"games\":[{games}]}}"),
        )
        .expect("harness");
        p
    }

    #[test]
    fn generated_registry_contains_fixture_game_and_static_imports() {
        let _lock = output_lock();
        let fixture = repo_root().join("fixtures/games/minimal-text-game");
        let harness = harness_for_paths(&[fixture]);
        generate_static_registry(&harness).expect("generate");
        let reg = std::fs::read_to_string(repo_root().join("web/src/generated/game-registry.ts"))
            .expect("read");
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
        let _lock = output_lock();
        let fixture = repo_root().join("fixtures/games/minimal-text-game");
        let harness = harness_for_paths(&[fixture]);
        prepare_public_assets(&harness).expect("prepare");
        let root = repo_root();
        assert!(root
            .join("web/public/_games/minimal-text-game/content/manifest.json")
            .exists());
        assert!(root
            .join("web/public/_games/minimal-text-game/content/date-index.json")
            .exists());
        assert!(root
            .join("web/public/_games/minimal-text-game/content/puzzles/2026-01-01.json")
            .exists());
        assert!(root
            .join("web/public/_games/minimal-text-game/runtime/index.js")
            .exists());
    }

    #[test]
    fn public_assets_preparation_is_deterministic() {
        let _lock = output_lock();
        let fixture = repo_root().join("fixtures/games/minimal-text-game");
        let harness = harness_for_paths(&[fixture]);
        prepare_public_assets(&harness).expect("prepare 1");
        let root = repo_root();
        let first = std::fs::read_to_string(
            root.join("web/public/_games/minimal-text-game/content/puzzles/2026-01-01.json"),
        )
        .expect("read first");
        prepare_public_assets(&harness).expect("prepare 2");
        let second = std::fs::read_to_string(
            root.join("web/public/_games/minimal-text-game/content/puzzles/2026-01-01.json"),
        )
        .expect("read second");
        assert_eq!(first, second);
    }

    #[cfg(unix)]
    #[test]
    fn public_assets_preparation_rejects_symlink_escape() {
        let _lock = output_lock();
        use std::os::unix::fs::symlink;
        let base = PathBuf::from(format!("/tmp/generate-symlink-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("content/puzzles")).expect("mkdir");
        std::fs::create_dir_all(base.join("content/assets")).expect("mkdir");
        std::fs::create_dir_all(base.join("dist/runtime")).expect("mkdir");
        std::fs::create_dir_all(base.join("src")).expect("mkdir");
        std::fs::write(base.join("daily-game.config.json"), r#"{"schemaVersion":"daily-game-package.v1","contractVersion":"daily-game-runtime.v1","game":{"id":"g","slug":"symlink-game","displayName":"Symlink Game"},"runtime":{"entry":"dist/runtime/index.js"},"ui":{"entry":"src/GameView.svelte"},"content":{"manifest":"content/manifest.json","puzzlesDir":"content/puzzles","assetsDir":"content/assets","dateIndex":"content/date-index.json"},"staticGeneration":{"dateDiscovery":{"mode":"date-index","path":"content/date-index.json"}},"extension":{}}"#).expect("pkg");
        std::fs::write(base.join("content/manifest.json"), r#"{"schemaVersion":"daily-game-content-manifest.v1","gameId":"g","inputModes":["text"],"extension":{}}"#).expect("manifest");
        std::fs::write(base.join("content/date-index.json"), r#"{"schemaVersion":"daily-game-date-index.v1","gameId":"g","dates":[{"date":"2026-01-01","puzzlePath":"content/puzzles/2026-01-01.json"}]}"#).expect("index");
        std::fs::write(base.join("content/puzzles/2026-01-01.json"), "{}").expect("puzzle");
        std::fs::write(base.join("dist/runtime/index.js"), "").expect("runtime");
        std::fs::write(base.join("src/GameView.svelte"), "").expect("ui");
        symlink("/etc/passwd", base.join("content/assets/escape")).expect("symlink");
        let harness = harness_for_paths(&[base]);
        assert!(prepare_public_assets(&harness).is_err());
    }

    #[test]
    fn add_game_by_config_only_generates_both_games() {
        let _lock = output_lock();
        let root = repo_root();
        let first = root.join("fixtures/games/minimal-text-game");
        let second = root.join("fixtures/games/second-minimal-game");
        let harness = harness_for_paths(&[first, second]);
        generate_static_registry(&harness).expect("generate");
        prepare_public_assets(&harness).expect("prepare assets");
        let status = std::process::Command::new("node")
            .current_dir(root.join("web"))
            .args(["scripts/build.mjs"])
            .status()
            .expect("node build");
        assert!(status.success());
        let reg =
            std::fs::read_to_string(root.join("web/src/generated/game-registry.ts")).expect("read");
        assert!(reg.contains("\"minimal-text-game\""));
        assert!(reg.contains("\"second-minimal-game\""));
        assert!(reg.contains("GameView_1"));
        assert!(reg.contains("createRuntime_1"));
        let home = std::fs::read_to_string(root.join("web/dist/index.html")).expect("home");
        assert!(home.contains("Minimal Text Game"));
        assert!(home.contains("Second Minimal Game"));
        assert!(root
            .join("web/dist/games/minimal-text-game/index.html")
            .exists());
        assert!(root
            .join("web/dist/games/second-minimal-game/index.html")
            .exists());
    }
}
