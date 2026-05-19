# Daily Game Harness Status

## Implemented so far
- Rust workspace and CLI scaffold (`daily_game_tools`, `daily_game_core`, `daily_game_wasm_support`).
- Harness config validation, package validation hardening, date-index validation hardening.
- Static registry generation, public asset preparation, and static output checker baseline.
- Basic web scaffold with generated registry usage and smoke E2E test.
- CI and deploy workflow scaffolds.

## What is still left (highest priority first)
1. Implement full runtime-driven content validation flow (`validateContent`, `validatePuzzle`) in harness pipeline.
2. Add comprehensive required tests from spec:
   - runtime contract determinism/error-path matrix
   - frontend shell and component tests
   - full Playwright 10-case E2E matrix
   - static output negative checks (`.git`/private cache exclusion, missing references, etc.)
3. Expand the config-only add-game proof from tooling-level assertions to full pipeline/route checks in tests.
4. Harden public asset copy against symlink escape deterministically in non-test runtime code.
5. Expand lockfile and git-source behavior tests (requested ref vs resolved SHA updates).
6. Complete README with full contract/pipeline/how-to documentation required by spec.
