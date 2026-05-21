# Daily Game Harness Spec Checklist

Status legend:
- [ ] Not started
- [~] In progress / partial
- [x] Complete

## A. Foundations
- [x] Rust workspace and crate scaffold exists.
- [x] Basic web scaffold exists.
- [x] Generic shell/component architecture complete.

## B. Config + source model
- [x] Harness config validation complete.
- [x] Local source support complete.
- [x] Git source sync + lockfile behavior complete.
- [x] NPM/artifact extension points documented.

## C. Validation pipeline
- [x] Package config contract validation complete.
- [x] Date-index discovery validation complete.
- [x] Runtime-driven `validateContent` in harness pipeline.
- [x] Runtime-driven `validatePuzzle` for included puzzles.
- [x] Full schema-driven validation flow wired end-to-end.

## D. Static generation
- [x] Generated game registry pipeline works.
- [x] Public asset copy pipeline works.
- [x] Static output checker exists.
- [x] Deterministic/symlink-safe copy hardening complete.
- [x] Full static-output negative checks complete.

## E. Frontend behavior
- [x] Basic routes exist.
- [x] Full generic gameplay shell behavior implemented.
- [x] Share behavior fallback contract implemented.
- [x] Full local-state rejection matrix implemented.

## F. Required tests
- [x] Rust validation tests implemented.
- [x] Rust lockfile behavior matrix complete.
- [x] Runtime contract tests implemented.
- [x] Frontend unit test matrix complete.
- [x] Svelte shell/component test matrix complete.
- [x] Playwright E2E test coverage complete.
- [x] Add-game-by-config-only proof test complete.

## G. CI/CD and docs
- [x] CI workflow exists.
- [x] Deploy workflow exists.
- [x] CI enforces full required test matrix.
- [x] README complete per required spec sections.
