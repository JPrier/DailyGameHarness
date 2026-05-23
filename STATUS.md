# Daily Game Harness Status

## Implemented
- Rust workspace and CLI scaffold (`daily_game_tools`, `daily_game_core`, `daily_game_wasm_support`).
- JSON Schema validation plus semantic validation for harness config, package config, optional date-index discovery, content manifests, puzzle resolver configs, archive configs, puzzle common fields, and runtime-driven `validateContent` / `validatePuzzle`.
- Local and git source sync with lockfile coverage for resolved paths, requested refs, resolved SHAs, and changed refs.
- Static registry generation, deterministic public asset preparation, symlink/dotfile hardening, Astro single-shell static route generation, and static output checking.
- Generic Astro/Svelte shell components for loading, validation, static-pool puzzle resolution, archive lists, persistence, feedback, results, share behavior, responsive styling, and friendly errors.
- Starter game package template under `templates/game-package/minimal-svelte-js`.
- GitHub Pages limitations and static host options documented under `docs/deployment.md`.
- Fixture and second fixture packages proving the config-only add-game flow.
- CI, GitHub Pages deploy workflow, optional S3 deploy script, README contract documentation, Rust tests, Vitest unit/component tests, and Playwright E2E matrix.

## Verification
- `cargo fmt --all -- --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: pass.
- `cargo test --workspace`: pass, 43 Rust tests.
- `cd web && npm run typecheck`: pass.
- `cd web && npm run test`: pass, 32 Vitest tests.
- `cargo run -p daily_game_tools -- prepare-static-build`: pass.
- `cd web && npm run build`: pass.
- `cargo run -p daily_game_tools -- check-static-output --dist web/dist`: pass.
- `cd web && npm run test:e2e`: pass, 12 Playwright tests.
