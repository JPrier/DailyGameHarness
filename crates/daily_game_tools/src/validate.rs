use crate::config::{validate_harness_config, Source};
use anyhow::{anyhow, bail, Context, Result};
use chrono::NaiveDate;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GamePackageConfig {
    pub schema_version: String,
    pub contract_version: String,
    pub game: GameMeta,
    pub runtime: Entry,
    pub ui: Entry,
    pub content: Content,
    pub static_generation: StaticGeneration,
    pub build: Option<Build>,
    pub extension: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[allow(dead_code)]
pub struct GameMeta {
    pub id: String,
    pub slug: String,
    pub display_name: String,
    pub short_description: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub entry: String,
    pub wasm: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub manifest: String,
    pub puzzles_dir: String,
    pub assets_dir: Option<String>,
    pub date_index: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticGeneration {
    pub date_discovery: DateDiscovery,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateDiscovery {
    pub mode: String,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Build {
    pub mode: Option<String>,
    pub commands: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[allow(dead_code)]
struct ContentManifest {
    schema_version: String,
    game_id: String,
    default_max_guesses: Option<u32>,
    input_modes: Vec<String>,
    share: Option<Value>,
    puzzle_schema: Option<Value>,
    extension: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DateIndex {
    schema_version: String,
    game_id: String,
    dates: Vec<DateEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DateEntry {
    date: String,
    puzzle_path: String,
    assets_prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PuzzleCommon {
    schema_version: String,
    game_id: String,
    puzzle_id: String,
    date: String,
    seed: String,
    display: PuzzleDisplay,
    extension: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PuzzleDisplay {
    title: String,
    initial_prompt: String,
}

pub fn package_roots(harness_path: &str) -> Result<Vec<PathBuf>> {
    let cfg = validate_harness_config(harness_path)?;
    let mut roots = Vec::new();
    for game in cfg.games {
        match game.source {
            Source::Local { path } => roots.push(PathBuf::from(path)),
            Source::Git { repo, .. } => {
                let safe = safe_git_dir_name(&repo);
                roots.push(PathBuf::from(".harness/external-games").join(safe));
            }
        }
    }
    Ok(roots)
}

pub fn read_package_config(root: &Path) -> Result<GamePackageConfig> {
    let raw = std::fs::read_to_string(root.join("daily-game.config.json"))
        .with_context(|| format!("read package config at {}", root.display()))?;
    let parsed: GamePackageConfig = serde_json::from_str(&raw)
        .with_context(|| format!("parse package config at {}", root.display()))?;
    validate_package_config(root, &parsed)?;
    Ok(parsed)
}

pub fn validate_games(harness_path: &str) -> Result<()> {
    let mut ids = HashSet::new();
    let mut slugs = HashSet::new();
    for root in package_roots(harness_path)? {
        let root = root.canonicalize().with_context(|| {
            format!(
                "game package root does not exist or is inaccessible: {}",
                root.display()
            )
        })?;
        let parsed = read_package_config(&root)?;
        if !ids.insert(parsed.game.id.clone()) || !slugs.insert(parsed.game.slug.clone()) {
            bail!("duplicate game IDs/slugs")
        }
    }
    Ok(())
}

pub fn build_or_verify_games(harness_path: &str) -> Result<()> {
    for root in package_roots(harness_path)? {
        let root = root.canonicalize()?;
        let parsed = read_package_config(&root)?;
        let mode = parsed
            .build
            .as_ref()
            .and_then(|build| build.mode.as_deref())
            .unwrap_or("prebuilt");
        if mode == "build-from-source" {
            for command in parsed
                .build
                .as_ref()
                .and_then(|build| build.commands.as_ref())
                .into_iter()
                .flatten()
            {
                run_package_command(&root, command)?;
            }
        } else if mode != "prebuilt" {
            bail!("unsupported build mode: {mode}");
        }
        assert_existing_file(&root, &parsed.runtime.entry, "runtime entry")?;
        assert_existing_file(&root, &parsed.ui.entry, "ui entry")?;
        if let Some(wasm) = &parsed.runtime.wasm {
            assert_existing_file(&root, wasm, "runtime wasm")?;
        }
    }
    Ok(())
}

pub fn validate_content(harness_path: &str) -> Result<()> {
    for root in package_roots(harness_path)? {
        let root = root.canonicalize()?;
        let package = read_package_config(&root)?;
        let manifest_path =
            assert_existing_file(&root, &package.content.manifest, "content manifest")?;
        let date_index_path =
            assert_existing_file(&root, &package.content.date_index, "date index")?;
        let manifest_raw = std::fs::read_to_string(&manifest_path)?;
        let date_index_raw = std::fs::read_to_string(&date_index_path)?;
        let manifest: ContentManifest = serde_json::from_str(&manifest_raw)?;
        let date_index: DateIndex = serde_json::from_str(&date_index_raw)?;
        validate_manifest_common(&package, &manifest)?;
        validate_date_index_common(&package, &date_index)?;
        run_runtime_validation(
            &root,
            &package.runtime.entry,
            &root.join("daily-game.config.json"),
            &manifest_path,
            &date_index_path,
            None,
        )?;

        for entry in &date_index.dates {
            let puzzle_path = resolve_safe_file(&root, &entry.puzzle_path, "puzzle path")?;
            let puzzle_raw = std::fs::read_to_string(&puzzle_path)?;
            let puzzle: PuzzleCommon = serde_json::from_str(&puzzle_raw)?;
            validate_puzzle_common(&package, entry, &puzzle)?;
            run_runtime_validation(
                &root,
                &package.runtime.entry,
                &root.join("daily-game.config.json"),
                &manifest_path,
                &date_index_path,
                Some(&puzzle_path),
            )?;
            if let Some(prefix) = &entry.assets_prefix {
                assert_path_within_root(&root, prefix, "assetsPrefix")?;
                let assets = root.join(prefix);
                if !assets.exists() {
                    bail!(
                        "referenced assetsPrefix does not exist: {}",
                        assets.display()
                    );
                }
            }
        }
    }
    Ok(())
}

fn validate_package_config(root: &Path, parsed: &GamePackageConfig) -> Result<()> {
    let slug_re = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")?;
    if parsed.schema_version != "daily-game-package.v1" {
        bail!("unsupported package schemaVersion")
    }
    if parsed.contract_version != "daily-game-runtime.v1" {
        bail!("unsupported contract version")
    }
    if parsed.game.id.is_empty() {
        bail!("missing game id")
    }
    if parsed.game.slug.is_empty() {
        bail!("missing slug")
    }
    if parsed.game.display_name.is_empty() {
        bail!("missing display name")
    }
    if !slug_re.is_match(&parsed.game.slug) {
        bail!("non url safe slug")
    }
    assert_path_within_root(root, &parsed.runtime.entry, "runtime entry")?;
    assert_path_within_root(root, &parsed.ui.entry, "ui entry")?;
    assert_path_within_root(root, &parsed.content.manifest, "content manifest")?;
    assert_path_within_root(root, &parsed.content.date_index, "date index")?;
    assert_path_within_root(root, &parsed.content.puzzles_dir, "puzzles dir")?;
    if let Some(assets_dir) = &parsed.content.assets_dir {
        assert_path_within_root(root, assets_dir, "assets dir")?;
    }
    if parsed.static_generation.date_discovery.mode != "date-index" {
        bail!("unsupported date discovery mode")
    }
    let discovery_path = parsed
        .static_generation
        .date_discovery
        .path
        .as_deref()
        .ok_or_else(|| anyhow!("missing date discovery path"))?;
    assert_path_within_root(root, discovery_path, "date discovery path")?;
    if discovery_path != parsed.content.date_index {
        bail!("date discovery path must match content.dateIndex")
    }
    if !parsed.extension.is_object() {
        bail!("extension must be object")
    }
    Ok(())
}

fn validate_manifest_common(package: &GamePackageConfig, manifest: &ContentManifest) -> Result<()> {
    if manifest.schema_version != "daily-game-content-manifest.v1" {
        bail!("unsupported content manifest schemaVersion")
    }
    if manifest.game_id != package.game.id {
        bail!("content manifest game id mismatch")
    }
    if manifest.input_modes.is_empty() {
        bail!("content manifest must declare input modes")
    }
    if manifest.default_max_guesses == Some(0) {
        bail!("defaultMaxGuesses must be positive")
    }
    if !manifest.extension.is_object() {
        bail!("content manifest extension must be object")
    }
    Ok(())
}

fn validate_date_index_common(package: &GamePackageConfig, idx: &DateIndex) -> Result<()> {
    if idx.schema_version != "daily-game-date-index.v1" {
        bail!("unsupported date index schemaVersion")
    }
    if idx.game_id != package.game.id {
        bail!("date-index game id mismatch")
    }
    let mut seen = HashSet::new();
    for entry in &idx.dates {
        NaiveDate::parse_from_str(&entry.date, "%Y-%m-%d")?;
        if !seen.insert(entry.date.clone()) {
            bail!("duplicate date");
        }
        if entry.puzzle_path.is_empty() {
            bail!("missing puzzle path");
        }
        if entry.puzzle_path.contains("..") {
            bail!("path traversal");
        }
    }
    Ok(())
}

fn validate_puzzle_common(
    package: &GamePackageConfig,
    entry: &DateEntry,
    puzzle: &PuzzleCommon,
) -> Result<()> {
    if puzzle.schema_version != "daily-game-puzzle.v1" {
        bail!("unsupported puzzle schemaVersion")
    }
    if puzzle.game_id != package.game.id {
        bail!("puzzle game id mismatch")
    }
    if puzzle.date != entry.date {
        bail!("puzzle date mismatch")
    }
    NaiveDate::parse_from_str(&puzzle.date, "%Y-%m-%d")?;
    if puzzle.puzzle_id.is_empty()
        || puzzle.seed.is_empty()
        || puzzle.display.title.is_empty()
        || puzzle.display.initial_prompt.is_empty()
    {
        bail!("puzzle missing required common fields")
    }
    if !puzzle.extension.is_object() {
        bail!("puzzle extension must be object")
    }
    Ok(())
}

pub fn assert_path_within_root(root: &Path, rel: &str, label: &str) -> Result<()> {
    if rel.is_empty() {
        bail!("missing {label}");
    }
    let rel_path = Path::new(rel);
    if rel_path.is_absolute() || rel.contains("..") {
        bail!("{label} escapes package root");
    }
    let canon_root = root.canonicalize()?;
    let joined = root.join(rel_path);
    let parent = joined.parent().ok_or_else(|| anyhow!("invalid {label}"))?;
    let canon_parent = parent.canonicalize().unwrap_or_else(|_| canon_root.clone());
    if !canon_parent.starts_with(&canon_root) {
        bail!("{label} escapes package root");
    }
    Ok(())
}

pub fn resolve_safe_file(root: &Path, rel: &str, label: &str) -> Result<PathBuf> {
    assert_path_within_root(root, rel, label)?;
    let path = root.join(rel);
    let meta = std::fs::symlink_metadata(&path)?;
    if meta.file_type().is_symlink() {
        bail!("{label} is a symlink");
    }
    if !meta.is_file() {
        bail!("{label} is not a file");
    }
    let canon = path.canonicalize()?;
    if !canon.starts_with(root.canonicalize()?) {
        bail!("{label} escapes package root");
    }
    Ok(path)
}

fn assert_existing_file(root: &Path, rel: &str, label: &str) -> Result<PathBuf> {
    resolve_safe_file(root, rel, label)
        .with_context(|| format!("{label} missing or invalid: {}", root.join(rel).display()))
}

fn run_package_command(root: &Path, command: &str) -> Result<()> {
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .current_dir(root)
            .args(["/C", command])
            .status()?
    } else {
        Command::new("sh")
            .current_dir(root)
            .args(["-c", command])
            .status()?
    };
    if !status.success() {
        bail!("package build command failed: {command}")
    }
    Ok(())
}

fn run_runtime_validation(
    root: &Path,
    runtime_entry: &str,
    package_config: &Path,
    manifest: &Path,
    date_index: &Path,
    puzzle: Option<&Path>,
) -> Result<()> {
    let runtime = root.join(runtime_entry).canonicalize()?;
    let script = r#"
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
const [runtimePath, packagePath, manifestPath, dateIndexPath, puzzlePath] = process.argv.slice(1);
const mod = await import(pathToFileURL(runtimePath).href);
const runtime = await mod.createRuntime();
const packageConfig = JSON.parse(readFileSync(packagePath, 'utf8'));
const contentManifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
const dateIndex = JSON.parse(readFileSync(dateIndexPath, 'utf8'));
let result;
if (puzzlePath) {
  const puzzle = JSON.parse(readFileSync(puzzlePath, 'utf8'));
  result = await runtime.validatePuzzle({ contentManifest, puzzle });
} else {
  result = await runtime.validateContent({ packageConfig, contentManifest, dateIndex });
}
if (!result || result.ok !== true) {
  console.error(JSON.stringify(result ?? { ok: false, errors: [{ code: 'invalid_result', message: 'runtime returned no validation result' }]}));
  process.exit(1);
}
"#;
    let mut command = Command::new("node");
    command
        .arg("--input-type=module")
        .arg("-e")
        .arg(script)
        .arg(runtime)
        .arg(package_config)
        .arg(manifest)
        .arg(date_index);
    if let Some(puzzle) = puzzle {
        command.arg(puzzle);
    }
    let output = command
        .output()
        .context("run runtime validation through node")?;
    if !output.status.success() {
        bail!(
            "runtime validation failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn safe_git_dir_name(repo: &str) -> String {
    repo.split('/')
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
    use std::path::PathBuf;

    fn repo_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("root")
    }

    fn write_harness_for(path: &str) -> String {
        std::fs::create_dir_all("/tmp").expect("tmp");
        let p = format!(
            "/tmp/harness-games-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        );
        let json = format!(
            "{{\"schemaVersion\":\"daily-game-harness.v1\",\"site\":{{\"routePrefix\":\"\"}},\"games\":[{{\"source\":{{\"type\":\"local\",\"path\":\"{}\"}}}}]}}",
            path.replace('\\', "\\\\")
        );
        std::fs::write(&p, json).expect("write harness");
        p
    }

    fn valid_pkg_body(id: &str, slug: &str) -> String {
        format!(
            r#"{{"schemaVersion":"daily-game-package.v1","contractVersion":"daily-game-runtime.v1","game":{{"id":"{id}","slug":"{slug}","displayName":"Game"}},"runtime":{{"entry":"dist/runtime/index.js"}},"ui":{{"entry":"src/GameView.svelte"}},"content":{{"manifest":"content/manifest.json","puzzlesDir":"content/puzzles","assetsDir":"content/assets","dateIndex":"content/date-index.json"}},"staticGeneration":{{"dateDiscovery":{{"mode":"date-index","path":"content/date-index.json"}}}},"extension":{{}}}}"#
        )
    }

    fn write_pkg(dir: &Path, body: &str) {
        std::fs::create_dir_all("/tmp").expect("tmp");
        std::fs::create_dir_all(dir.join("dist/runtime")).expect("mkdir runtime");
        std::fs::create_dir_all(dir.join("src")).expect("mkdir src");
        std::fs::create_dir_all(dir.join("content/puzzles")).expect("mkdir content");
        std::fs::create_dir_all(dir.join("content/assets")).expect("mkdir assets");
        std::fs::write(dir.join("daily-game.config.json"), body).expect("write pkg config");
        std::fs::write(
            dir.join("dist/runtime/index.js"),
            "export async function createRuntime(){}",
        )
        .expect("runtime");
        std::fs::write(dir.join("src/GameView.svelte"), "").expect("ui");
        std::fs::write(dir.join("content/manifest.json"), r#"{"schemaVersion":"daily-game-content-manifest.v1","gameId":"g","inputModes":["text"],"extension":{}}"#).expect("manifest");
        std::fs::write(
            dir.join("content/date-index.json"),
            r#"{"schemaVersion":"daily-game-date-index.v1","gameId":"g","dates":[]}"#,
        )
        .expect("index");
    }

    #[test]
    fn accepts_valid_fixture_package_config() {
        let fixture = repo_root().join("fixtures/games/minimal-text-game");
        let h = write_harness_for(&fixture.display().to_string());
        assert!(validate_games(&h).is_ok());
    }

    #[test]
    fn rejects_missing_game_id() {
        let tmp = PathBuf::from(format!("/tmp/pkg-missing-id-{}", std::process::id()));
        write_pkg(&tmp, &valid_pkg_body("", "good-slug"));
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn rejects_missing_slug() {
        let tmp = PathBuf::from(format!("/tmp/pkg-missing-slug-{}", std::process::id()));
        write_pkg(&tmp, &valid_pkg_body("g", ""));
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn rejects_non_url_safe_slug() {
        let tmp = PathBuf::from(format!("/tmp/pkg-slug-{}", std::process::id()));
        write_pkg(&tmp, &valid_pkg_body("g", "Bad Slug"));
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn rejects_missing_runtime_entry() {
        let tmp = PathBuf::from(format!("/tmp/pkg-runtime-{}", std::process::id()));
        let body = valid_pkg_body("g", "good-slug")
            .replace(r#""entry":"dist/runtime/index.js""#, r#""entry":"""#);
        write_pkg(&tmp, &body);
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn rejects_missing_ui_entry() {
        let tmp = PathBuf::from(format!("/tmp/pkg-ui-{}", std::process::id()));
        let body = valid_pkg_body("g", "good-slug")
            .replace(r#""entry":"src/GameView.svelte""#, r#""entry":"""#);
        write_pkg(&tmp, &body);
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn rejects_missing_content_manifest() {
        let tmp = PathBuf::from(format!("/tmp/pkg-manifest-{}", std::process::id()));
        let body = valid_pkg_body("g", "good-slug")
            .replace(r#""manifest":"content/manifest.json""#, r#""manifest":"""#);
        write_pkg(&tmp, &body);
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn rejects_missing_date_index() {
        let tmp = PathBuf::from(format!("/tmp/pkg-index-{}", std::process::id()));
        let body = valid_pkg_body("g", "good-slug").replace(
            r#""dateIndex":"content/date-index.json""#,
            r#""dateIndex":"""#,
        );
        write_pkg(&tmp, &body);
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn rejects_unsupported_contract_version() {
        let tmp = PathBuf::from(format!("/tmp/pkg-contract-{}", std::process::id()));
        let body = valid_pkg_body("g", "good-slug")
            .replace("daily-game-runtime.v1", "daily-game-runtime.v2");
        write_pkg(&tmp, &body);
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn rejects_paths_escaping_package_root() {
        let tmp = PathBuf::from(format!("/tmp/pkg-path-{}", std::process::id()));
        let body = valid_pkg_body("g2", "good-slug").replace("dist/runtime/index.js", "../evil.js");
        write_pkg(&tmp, &body);
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn allows_unknown_fields_inside_extension() {
        let tmp = PathBuf::from(format!("/tmp/pkg-ext-{}", std::process::id()));
        let body = valid_pkg_body("g3", "ext-slug").replace(
            r#""extension":{}"#,
            r#""extension":{"anything":{"goes":true}}"#,
        );
        write_pkg(&tmp, &body);
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_ok());
    }

    #[test]
    fn rejects_duplicate_game_entries() {
        std::fs::create_dir_all("/tmp").expect("tmp");
        let fixture = repo_root().join("fixtures/games/minimal-text-game");
        let p = format!("/tmp/harness-dupe-{}.json", std::process::id());
        let path = fixture.display().to_string().replace('\\', "\\\\");
        std::fs::write(
            &p,
            format!(
                "{{\"schemaVersion\":\"daily-game-harness.v1\",\"site\":{{\"routePrefix\":\"\"}},\"games\":[{{\"source\":{{\"type\":\"local\",\"path\":\"{path}\"}}}},{{\"source\":{{\"type\":\"local\",\"path\":\"{path}\"}}}}]}}"
            ),
        )
        .expect("write harness");
        assert!(validate_games(&p).is_err());
    }
}
