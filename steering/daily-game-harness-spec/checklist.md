# Daily Game Harness Spec Checklist

Status legend:
- [ ] Not started
- [~] In progress / partial
- [x] Complete

## A. Foundations
- [x] Rust workspace and crate scaffold exists.
- [x] Basic web scaffold exists.
- [~] Generic shell/component architecture complete.

## B. Config + source model
- [~] Harness config validation complete.
- [~] Local source support complete.
- [~] Git source sync + lockfile behavior complete.
- [ ] NPM/artifact extension points documented.

## C. Validation pipeline
- [~] Package config contract validation complete.
- [~] Date-index discovery validation complete.
- [ ] Runtime-driven `validateContent` in harness pipeline.
- [ ] Runtime-driven `validatePuzzle` for included puzzles.
- [ ] Full schema-driven validation flow wired end-to-end.

## D. Static generation
- [~] Generated game registry pipeline works.
- [~] Public asset copy pipeline works.
- [~] Static output checker exists.
- [ ] Deterministic/symlink-safe copy hardening complete.
- [ ] Full static-output negative checks complete.

## E. Frontend behavior
- [~] Basic routes exist.
- [ ] Full generic gameplay shell behavior implemented.
- [ ] Share behavior fallback contract implemented.
- [ ] Full local-state rejection matrix implemented.

## F. Required tests
- [~] Rust validation tests partially implemented.
- [ ] Rust lockfile behavior matrix complete.
- [~] Runtime contract tests partially implemented.
- [ ] Frontend unit test matrix complete.
- [ ] Svelte shell/component test matrix complete.
- [~] Playwright E2E test coverage (currently smoke only).
- [ ] Add-game-by-config-only proof test complete.

## G. CI/CD and docs
- [~] CI workflow exists.
- [~] Deploy workflow exists.
- [ ] CI enforces full required test matrix.
- [ ] README complete per required spec sections.

## Current priorities
1. Wire runtime-driven content/puzzle validation into tooling pipeline.
2. Expand test matrix (runtime/frontend/svelte/e2e/static-negative).
3. Complete add-game-by-config-only proof test.
4. Harden asset-copy safety and lockfile/git behavior coverage.
5. Finish README spec-level documentation.
