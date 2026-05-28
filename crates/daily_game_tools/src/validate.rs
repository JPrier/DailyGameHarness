use crate::archive::{validate_archive, ArchiveConfig};
use crate::config::{validate_harness_config, Source};
use crate::resolver::{format_pattern, validate_resolver, PuzzleResolverConfig};
use crate::schema;
use anyhow::{anyhow, bail, Context, Result};
use chrono::NaiveDate;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GamePackageConfig {
    pub schema_version: String,
    pub contract_version: String,
    pub game: GameMeta,
    pub runtime: Entry,
    pub ui: Entry,
    pub content: Content,
    pub static_generation: Option<StaticGeneration>,
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
    pub date_index: Option<String>,
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Build {
    pub mode: String,
    pub command: Option<String>,
    pub commands: Option<Vec<BuildCommand>>,
    pub outputs: Option<Vec<String>>,
    pub environment: Option<BuildEnvironment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildCommand {
    pub name: String,
    pub run: String,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildEnvironment {
    pub node: Option<String>,
    pub system_tools: Option<Vec<String>>,
    pub network: Option<String>,
    pub cache_dirs: Option<Vec<String>>,
    pub max_build_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[allow(dead_code)]
struct ContentManifest {
    schema_version: String,
    game_id: String,
    display_name: Option<String>,
    default_max_guesses: Option<u32>,
    input_modes: Vec<String>,
    share: Option<Value>,
    puzzle_resolver: PuzzleResolverConfig,
    archive: ArchiveConfig,
    asset_loading: Option<Value>,
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
#[serde(rename_all = "camelCase")]
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
    let config_path = root.join("daily-game.config.json");
    let value = schema::validate_json_file("game-package-config.schema.json", &config_path)?;
    let raw = serde_json::to_string(&value)?;
    let parsed: GamePackageConfig = serde_json::from_str(&raw)
        .with_context(|| format!("read package config at {}", root.display()))?;
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
            .map(|build| build.mode.as_str())
            .unwrap_or("prebuilt");
        match mode {
            "prebuilt" => {}
            "command" => run_command_build(&root, &parsed)?,
            other => bail!("unsupported build mode: {other}"),
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
        let date_index_path = package
            .content
            .date_index
            .as_deref()
            .map(|path| assert_existing_file(&root, path, "date index"))
            .transpose()?;
        let manifest_value =
            schema::validate_json_file("content-manifest.schema.json", &manifest_path)?;
        let manifest_raw = serde_json::to_string(&manifest_value)?;
        let manifest: ContentManifest = serde_json::from_str(&manifest_raw)?;
        validate_manifest_common(&package, &manifest)?;
        validate_resolver(&manifest.puzzle_resolver)?;
        validate_archive(&manifest.archive)?;

        let date_index = if let Some(date_index_path) = &date_index_path {
            let date_index_value =
                schema::validate_json_file("date-index.schema.json", date_index_path)?;
            let date_index_raw = serde_json::to_string(&date_index_value)?;
            let date_index: DateIndex = serde_json::from_str(&date_index_raw)?;
            validate_date_index_common(&package, &date_index)?;
            Some(date_index)
        } else {
            None
        };
        run_runtime_validation(
            &root,
            &package.runtime.entry,
            &root.join("daily-game.config.json"),
            &manifest_path,
            date_index_path.as_deref(),
            None,
        )?;

        if let Some(date_index) = &date_index {
            for entry in &date_index.dates {
                let puzzle_path = resolve_safe_file(&root, &entry.puzzle_path, "puzzle path")?;
                let puzzle_value =
                    schema::validate_json_file("puzzle-common.schema.json", &puzzle_path)?;
                let puzzle_raw = serde_json::to_string(&puzzle_value)?;
                let puzzle: PuzzleCommon = serde_json::from_str(&puzzle_raw)?;
                validate_puzzle_common(&package, entry, &puzzle)?;
                run_runtime_validation(
                    &root,
                    &package.runtime.entry,
                    &root.join("daily-game.config.json"),
                    &manifest_path,
                    date_index_path.as_deref(),
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
        } else {
            validate_static_pool_puzzles(&root, &package, &manifest)?;
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
    if let Some(date_index) = &parsed.content.date_index {
        assert_path_within_root(root, date_index, "date index")?;
    }
    assert_path_within_root(root, &parsed.content.puzzles_dir, "puzzles dir")?;
    if let Some(assets_dir) = &parsed.content.assets_dir {
        assert_path_within_root(root, assets_dir, "assets dir")?;
    }
    if let Some(static_generation) = &parsed.static_generation {
        if static_generation.date_discovery.mode != "date-index" {
            bail!("unsupported date discovery mode")
        }
        let discovery_path = static_generation
            .date_discovery
            .path
            .as_deref()
            .ok_or_else(|| anyhow!("missing date discovery path"))?;
        assert_path_within_root(root, discovery_path, "date discovery path")?;
        if Some(discovery_path) != parsed.content.date_index.as_deref() {
            bail!("date discovery path must match content.dateIndex")
        }
    }
    validate_build_config(root, parsed)?;
    if !parsed.extension.is_object() {
        bail!("extension must be object")
    }
    Ok(())
}

fn validate_build_config(root: &Path, parsed: &GamePackageConfig) -> Result<()> {
    let Some(build) = &parsed.build else {
        return Ok(());
    };
    match build.mode.as_str() {
        "prebuilt" => {
            if build.command.is_some() || build.commands.is_some() {
                bail!("prebuilt mode must not declare build commands");
            }
        }
        "command" => {
            if build.command.is_some() == build.commands.is_some() {
                bail!("command mode must declare exactly one of command or commands");
            }
            let outputs = build
                .outputs
                .as_ref()
                .filter(|outputs| !outputs.is_empty())
                .ok_or_else(|| anyhow!("command mode must declare non-empty outputs"))?;
            for output in outputs {
                assert_path_within_root(root, output, "build output")?;
            }
            for command in build_commands(build)? {
                if command.name.trim().is_empty() {
                    bail!("build command name must be non-empty");
                }
                if command.run.trim().is_empty() {
                    bail!("build command run string must be non-empty");
                }
                if command.timeout_seconds == Some(0) {
                    bail!("build command timeoutSeconds must be positive");
                }
            }
            if let Some(environment) = &build.environment {
                if let Some(node) = &environment.node {
                    if node.trim().is_empty() {
                        bail!("environment.node must be non-empty");
                    }
                }
                if let Some(system_tools) = &environment.system_tools {
                    if system_tools.iter().any(|tool| tool.trim().is_empty()) {
                        bail!("environment.systemTools must contain non-empty names");
                    }
                }
                if let Some(network) = &environment.network {
                    if !matches!(
                        network.as_str(),
                        "disabled-by-default" | "allowed" | "required"
                    ) {
                        bail!("unsupported build network policy: {network}");
                    }
                }
                if let Some(cache_dirs) = &environment.cache_dirs {
                    for cache_dir in cache_dirs {
                        assert_path_within_root(root, cache_dir, "build cache dir")?;
                    }
                }
                if environment.max_build_seconds == Some(0) {
                    bail!("environment.maxBuildSeconds must be positive");
                }
            }
        }
        other => bail!("unsupported build mode: {other}"),
    }
    Ok(())
}

fn build_commands(build: &Build) -> Result<Vec<BuildCommandRef<'_>>> {
    if let Some(command) = &build.command {
        return Ok(vec![BuildCommandRef {
            name: "command",
            run: command,
            timeout_seconds: None,
        }]);
    }
    Ok(build
        .commands
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|command| BuildCommandRef {
            name: &command.name,
            run: &command.run,
            timeout_seconds: command.timeout_seconds,
        })
        .collect())
}

struct BuildCommandRef<'a> {
    name: &'a str,
    run: &'a str,
    timeout_seconds: Option<u64>,
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

fn validate_static_pool_puzzles(
    root: &Path,
    package: &GamePackageConfig,
    manifest: &ContentManifest,
) -> Result<()> {
    if let PuzzleResolverConfig::StaticPool { pool_versions, .. } = &manifest.puzzle_resolver {
        for version in pool_versions {
            for index in 0..version.pool_size {
                let rel = format_pattern(
                    &version.path_pattern,
                    &version.version,
                    index,
                    &version.start_date,
                );
                let puzzle_path = resolve_safe_file(root, &rel, "static-pool puzzle")?;
                let puzzle_value =
                    schema::validate_json_file("puzzle-common.schema.json", &puzzle_path)?;
                let puzzle_raw = serde_json::to_string(&puzzle_value)?;
                let puzzle: PuzzleCommon = serde_json::from_str(&puzzle_raw)?;
                validate_puzzle_common_loose(package, &puzzle)?;
                run_runtime_validation(
                    root,
                    &package.runtime.entry,
                    &root.join("daily-game.config.json"),
                    &root.join(&package.content.manifest),
                    None,
                    Some(&puzzle_path),
                )?;
            }
        }
    }
    Ok(())
}

fn validate_puzzle_common_loose(package: &GamePackageConfig, puzzle: &PuzzleCommon) -> Result<()> {
    if puzzle.schema_version != "daily-game-puzzle.v1" {
        bail!("unsupported puzzle schemaVersion")
    }
    if puzzle.game_id != package.game.id {
        bail!("puzzle game id mismatch")
    }
    if !puzzle.date.is_empty() {
        NaiveDate::parse_from_str(&puzzle.date, "%Y-%m-%d")?;
    }
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

fn run_command_build(root: &Path, package: &GamePackageConfig) -> Result<()> {
    let build = package
        .build
        .as_ref()
        .ok_or_else(|| anyhow!("missing build config"))?;
    let network = build
        .environment
        .as_ref()
        .and_then(|environment| environment.network.as_deref())
        .unwrap_or("disabled-by-default");
    println!(
        "package {} build mode command; network policy: {network}",
        package.game.id
    );
    for command in build_commands(build)? {
        let timeout = command
            .timeout_seconds
            .or_else(|| {
                build
                    .environment
                    .as_ref()
                    .and_then(|environment| environment.max_build_seconds)
            })
            .unwrap_or(1200);
        run_package_command(root, package, &command, timeout)?;
    }
    for output in build.outputs.as_deref().unwrap_or_default() {
        assert_declared_output(root, output).with_context(|| {
            format!(
                "package {} missing declared output: {output}",
                package.game.id
            )
        })?;
    }
    Ok(())
}

fn run_package_command(
    root: &Path,
    package: &GamePackageConfig,
    build_command: &BuildCommandRef<'_>,
    timeout_seconds: u64,
) -> Result<()> {
    let mut command = if cfg!(target_os = "windows") {
        let mut command = Command::new("cmd");
        command.args(["/C", build_command.run]);
        command
    } else {
        let mut command = Command::new("sh");
        command.args(["-c", build_command.run]);
        command
    };
    let mut child = command
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("spawn package build command {}", build_command.name))?;
    let started = Instant::now();
    let timeout = Duration::from_secs(timeout_seconds);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() > timeout {
            let _ = child.kill();
            let stdout = read_pipe(child.stdout.take());
            let stderr = read_pipe(child.stderr.take());
            bail!(
                "{}",
                build_failure_context(
                    package,
                    root,
                    build_command,
                    &format!("timed out after {timeout_seconds}s"),
                    &stdout,
                    &stderr,
                )
            );
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let stdout = read_pipe(child.stdout.take());
    let stderr = read_pipe(child.stderr.take());
    if !status.success() {
        bail!(
            "{}",
            build_failure_context(
                package,
                root,
                build_command,
                &format!(
                    "exit code {}",
                    status
                        .code()
                        .map_or_else(|| "unknown".into(), |code| code.to_string())
                ),
                &stdout,
                &stderr,
            )
        );
    }
    Ok(())
}

fn read_pipe(pipe: Option<impl Read>) -> String {
    let Some(mut pipe) = pipe else {
        return String::new();
    };
    let mut output = String::new();
    let _ = pipe.read_to_string(&mut output);
    output
}

fn build_failure_context(
    package: &GamePackageConfig,
    root: &Path,
    command: &BuildCommandRef<'_>,
    reason: &str,
    stdout: &str,
    stderr: &str,
) -> String {
    format!(
        "package build failed\npackage id: {}\nsource path: {}\nbuild mode: command\ncommand name: {}\ncommand string: {}\nfailure: {}\nstdout tail:\n{}\nstderr tail:\n{}",
        package.game.id,
        root.display(),
        command.name,
        command.run,
        reason,
        tail_lines(stdout, 40),
        tail_lines(stderr, 40)
    )
}

fn tail_lines(value: &str, max_lines: usize) -> String {
    let lines = value.lines().collect::<Vec<_>>();
    let start = lines.len().saturating_sub(max_lines);
    lines[start..].join("\n")
}

fn assert_declared_output(root: &Path, rel: &str) -> Result<()> {
    assert_path_within_root(root, rel, "build output")?;
    let path = root.join(rel);
    let meta = std::fs::symlink_metadata(&path)
        .with_context(|| format!("declared build output does not exist: {}", path.display()))?;
    if meta.file_type().is_symlink() {
        bail!("declared build output is a symlink: {}", path.display());
    }
    let canon = path.canonicalize()?;
    if !canon.starts_with(root.canonicalize()?) {
        bail!("declared build output escapes package root");
    }
    Ok(())
}

fn run_runtime_validation(
    root: &Path,
    runtime_entry: &str,
    package_config: &Path,
    manifest: &Path,
    date_index: Option<&Path>,
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
const dateIndex = dateIndexPath ? JSON.parse(readFileSync(dateIndexPath, 'utf8')) : null;
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
        .arg(manifest);
    if let Some(date_index) = date_index {
        command.arg(date_index);
    } else {
        command.arg("");
    }
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
            "{{\"schemaVersion\":\"daily-game-harness.v1\",\"site\":{{\"name\":\"Daily Games\",\"baseUrl\":\"https://example.com\",\"routePrefix\":\"\"}},\"games\":[{{\"source\":{{\"type\":\"local\",\"path\":\"{}\"}}}}],\"staticGeneration\":{{\"routeMode\":\"single-shell\"}},\"deployment\":{{\"target\":\"github-pages\"}}}}",
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

    fn with_build(body: &str, build: &str) -> String {
        body.replace(
            r#""extension":{}"#,
            &format!(r#""build":{build},"extension":{{}}"#),
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

    fn write_command_pkg(
        dir: &Path,
        script: &str,
        outputs: &[&str],
        timeout_seconds: u64,
    ) -> String {
        std::fs::create_dir_all(dir.join("tools")).expect("tools");
        std::fs::create_dir_all(dir.join("src")).expect("src");
        std::fs::write(dir.join("src/GameView.svelte"), "").expect("ui");
        std::fs::write(dir.join("tools/build.mjs"), script).expect("script");
        let outputs_json = outputs
            .iter()
            .map(|output| format!(r#""{output}""#))
            .collect::<Vec<_>>()
            .join(",");
        let body = format!(
            r#"{{"schemaVersion":"daily-game-package.v1","contractVersion":"daily-game-runtime.v1","game":{{"id":"command-test","slug":"command-test","displayName":"Command Test"}},"runtime":{{"entry":"dist/runtime/index.js"}},"ui":{{"entry":"src/GameView.svelte"}},"content":{{"manifest":"content/manifest.json","puzzlesDir":"content/puzzles","assetsDir":"content/assets"}},"build":{{"mode":"command","commands":[{{"name":"build","run":"node tools/build.mjs","timeoutSeconds":{timeout_seconds}}}],"outputs":[{outputs_json}]}},"extension":{{}}}}"#
        );
        std::fs::write(dir.join("daily-game.config.json"), &body).expect("pkg");
        body
    }

    fn valid_command_script() -> &'static str {
        r#"
import fs from 'node:fs';
import path from 'node:path';
const write = (rel, value) => {
  fs.mkdirSync(path.dirname(rel), { recursive: true });
  fs.writeFileSync(rel, value);
};
write('dist/runtime/index.js', `export async function createRuntime(){return{contractVersion:'daily-game-runtime.v1',async validateContent(){return{ok:true,warnings:[]}},async validatePuzzle(){return{ok:true,warnings:[]}},async createInitialState({puzzle,date}){return{schemaVersion:'daily-game-state.v1',gameId:puzzle.gameId,puzzleId:puzzle.puzzleId,date,status:'in_progress',guessCount:0,maxGuesses:1,currentStage:0,publicState:{}}},async submitGuess({state}){return{state,evaluation:{outcome:'invalid',consumedGuess:false,feedback:[]}}},async buildShareText(){return 'share'}}}`);
write('content/manifest.json', JSON.stringify({schemaVersion:'daily-game-content-manifest.v1',gameId:'command-test',inputModes:['text'],puzzleResolver:{mode:'static-pool',timezone:'America/New_York',startDate:'2026-01-01',poolVersions:[{version:'v1',startDate:'2026-01-01',poolSize:1,pathPattern:'content/puzzles/{version}/puzzle-{index:04}.json',selector:{type:'affine-permutation',a:1,b:0},cyclePolicy:'repeat'}]},archive:{mode:'rolling-window',days:30,includeToday:true,allowFutureDates:false,directAccess:'within-archive-window'},extension:{}}, null, 2));
write('content/puzzles/v1/puzzle-0000.json', JSON.stringify({schemaVersion:'daily-game-puzzle.v1',gameId:'command-test',puzzleId:'command-test-v1-0000',date:'2026-01-01',seed:'command-test',display:{title:'Command Test',initialPrompt:'Guess'},extension:{}}, null, 2));
write('content/assets/asset.txt', 'asset');
"#
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
    fn accepts_prebuilt_build_mode() {
        let tmp = PathBuf::from(format!("/tmp/pkg-prebuilt-{}", std::process::id()));
        write_pkg(
            &tmp,
            &with_build(&valid_pkg_body("g", "good-slug"), r#"{"mode":"prebuilt"}"#),
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_ok());
    }

    #[test]
    fn accepts_command_build_mode_with_commands() {
        let tmp = PathBuf::from(format!("/tmp/pkg-command-{}", std::process::id()));
        write_pkg(
            &tmp,
            &with_build(
                &valid_pkg_body("g", "good-slug"),
                r#"{"mode":"command","commands":[{"name":"build","run":"node tools/build.js"}],"outputs":["dist/runtime/index.js","content/manifest.json"]}"#,
            ),
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_ok());
    }

    #[test]
    fn accepts_command_build_mode_with_single_command() {
        let tmp = PathBuf::from(format!("/tmp/pkg-single-command-{}", std::process::id()));
        write_pkg(
            &tmp,
            &with_build(
                &valid_pkg_body("g", "good-slug"),
                r#"{"mode":"command","command":"node tools/build.js","outputs":["dist/runtime/index.js"]}"#,
            ),
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_ok());
    }

    #[test]
    fn rejects_command_mode_with_both_command_forms() {
        let tmp = PathBuf::from(format!("/tmp/pkg-both-commands-{}", std::process::id()));
        write_pkg(
            &tmp,
            &with_build(
                &valid_pkg_body("g", "good-slug"),
                r#"{"mode":"command","command":"echo one","commands":[{"name":"two","run":"echo two"}],"outputs":["dist/runtime/index.js"]}"#,
            ),
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn rejects_command_mode_without_outputs() {
        let tmp = PathBuf::from(format!("/tmp/pkg-no-outputs-{}", std::process::id()));
        write_pkg(
            &tmp,
            &with_build(
                &valid_pkg_body("g", "good-slug"),
                r#"{"mode":"command","command":"echo one","outputs":[]}"#,
            ),
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn rejects_command_mode_with_absolute_or_parent_outputs() {
        for (name, output) in [("absolute", "/tmp/out"), ("parent", "../out")] {
            let tmp = PathBuf::from(format!("/tmp/pkg-output-{name}-{}", std::process::id()));
            write_pkg(
                &tmp,
                &with_build(
                    &valid_pkg_body("g", "good-slug"),
                    &format!(r#"{{"mode":"command","command":"echo one","outputs":["{output}"]}}"#),
                ),
            );
            let h = write_harness_for(&tmp.display().to_string());
            assert!(validate_games(&h).is_err());
        }
    }

    #[test]
    fn rejects_prebuilt_mode_with_command() {
        let tmp = PathBuf::from(format!("/tmp/pkg-prebuilt-command-{}", std::process::id()));
        write_pkg(
            &tmp,
            &with_build(
                &valid_pkg_body("g", "good-slug"),
                r#"{"mode":"prebuilt","command":"echo no"}"#,
            ),
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(validate_games(&h).is_err());
    }

    #[test]
    fn rejects_unknown_build_mode_and_nonpositive_timeout() {
        for (name, build) in [
            ("mode", r#"{"mode":"other"}"#),
            (
                "timeout",
                r#"{"mode":"command","commands":[{"name":"build","run":"echo one","timeoutSeconds":0}],"outputs":["dist/runtime/index.js"]}"#,
            ),
        ] {
            let tmp = PathBuf::from(format!("/tmp/pkg-bad-build-{name}-{}", std::process::id()));
            write_pkg(&tmp, &with_build(&valid_pkg_body("g", "good-slug"), build));
            let h = write_harness_for(&tmp.display().to_string());
            assert!(validate_games(&h).is_err());
        }
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
                "{{\"schemaVersion\":\"daily-game-harness.v1\",\"site\":{{\"name\":\"Daily Games\",\"baseUrl\":\"https://example.com\",\"routePrefix\":\"\"}},\"games\":[{{\"source\":{{\"type\":\"local\",\"path\":\"{path}\"}}}},{{\"source\":{{\"type\":\"local\",\"path\":\"{path}\"}}}}],\"staticGeneration\":{{\"routeMode\":\"single-shell\"}},\"deployment\":{{\"target\":\"github-pages\"}}}}"
            ),
        )
        .expect("write harness");
        assert!(validate_games(&p).is_err());
    }

    #[test]
    fn command_build_generates_outputs_before_validation() {
        let tmp = PathBuf::from(format!("/tmp/pkg-command-build-{}", std::process::id()));
        write_command_pkg(
            &tmp,
            valid_command_script(),
            &[
                "dist/runtime/index.js",
                "content/manifest.json",
                "content/puzzles",
            ],
            10,
        );
        let h = write_harness_for(&tmp.display().to_string());
        assert!(build_or_verify_games(&h).is_ok());
        validate_content(&h).expect("generated content should validate");
        assert!(tmp.join("dist/runtime/index.js").exists());
        assert!(tmp.join("content/puzzles/v1/puzzle-0000.json").exists());
    }

    #[test]
    fn command_build_fails_for_missing_output() {
        let tmp = PathBuf::from(format!(
            "/tmp/pkg-command-missing-output-{}",
            std::process::id()
        ));
        write_command_pkg(
            &tmp,
            valid_command_script(),
            &["dist/runtime/index.js", "missing/output.txt"],
            10,
        );
        let h = write_harness_for(&tmp.display().to_string());
        let err = build_or_verify_games(&h).expect_err("missing output should fail");
        assert!(format!("{err:?}").contains("missing declared output"));
    }

    #[test]
    fn command_build_fails_for_nonzero_exit_with_output_context() {
        let tmp = PathBuf::from(format!("/tmp/pkg-command-nonzero-{}", std::process::id()));
        write_command_pkg(
            &tmp,
            "console.log('stdout marker'); console.error('stderr marker'); process.exit(7);",
            &["dist/runtime/index.js"],
            10,
        );
        let h = write_harness_for(&tmp.display().to_string());
        let err = build_or_verify_games(&h).expect_err("nonzero should fail");
        let text = format!("{err:?}");
        assert!(text.contains("exit code 7"));
        assert!(text.contains("stdout marker"));
        assert!(text.contains("stderr marker"));
    }

    #[test]
    fn command_build_fails_on_timeout() {
        let tmp = PathBuf::from(format!("/tmp/pkg-command-timeout-{}", std::process::id()));
        write_command_pkg(
            &tmp,
            "setTimeout(() => console.log('too late'), 3000);",
            &["dist/runtime/index.js"],
            1,
        );
        let h = write_harness_for(&tmp.display().to_string());
        let err = build_or_verify_games(&h).expect_err("timeout should fail");
        assert!(format!("{err:?}").contains("timed out"));
    }
}
