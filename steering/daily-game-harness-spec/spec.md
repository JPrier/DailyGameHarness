# One-Shot Agent Spec: Generic Static Daily Game Harness

## Goal
Build a generic, reusable static daily-game harness that consumes external game packages via configuration.

## Core principles
- Harness owns shell, discovery, static generation, persistence, validation, CI/CD.
- Game packages own logic, WASM runtime, UI component, content/assets, package config, game-specific validation.
- No hardcoded real games in harness.
- Fixture games exist only for contract and integration testing.

## Required architecture
- Rust crates: `daily_game_core`, `daily_game_wasm_support`, `daily_game_tools`.
- Static frontend shell using Astro + Svelte + TypeScript.
- CLI pipeline for validation, sync, generation, static checks.
- S3 deploy workflow with OIDC credentials.

## Required pipeline (high level)
1. Validate harness config.
2. Sync local/git game sources and write lockfile.
3. Validate package config contract.
4. Build/verify package artifacts.
5. Discover dates.
6. Validate content + runtime validations.
7. Generate static game registry.
8. Copy public assets.
9. Build static site.
10. Verify static output integrity.

## Required tests (high level)
- Rust unit tests for config/package/date/static/lockfile behavior.
- Runtime contract tests (validation/state/guess/share/determinism).
- Frontend unit tests (loader, routes, localStorage, feedback, share behavior).
- Svelte component tests for shell and fixture view behavior.
- Playwright E2E matrix for game and route behavior.
- Add-game-by-config-only proof test.

## Reference
This file tracks the active implementation target from the project conversation.
