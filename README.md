# Daily Game Harness

Daily Game Harness is a generic static-site harness for hosting Wordle-style daily games supplied as external packages.

The harness does not implement games. Games are external packages that conform to the daily-game package contract.

## What It Is

The harness owns the static site shell, game discovery, static route generation, local progress persistence, share UI, package validation, contract tests, CI, and static deployment.

It is designed so a conforming package can be added through `harness.config.json` without adding game-specific source imports or routes to the harness.

## What It Is Not

This repository is not a real game repository and does not contain product game logic. The fixture packages under `fixtures/games/*` exist only to prove the contract and integration behavior.

## Package Model

Games are declared in `harness.config.json`:

```json
{
  "schemaVersion": "daily-game-harness.v1",
  "site": { "name": "Daily Games", "baseUrl": "https://example.com", "routePrefix": "" },
  "games": [
    { "source": { "type": "local", "path": "fixtures/games/minimal-text-game" } },
    { "source": { "type": "git", "repo": "https://github.com/example/game.git", "ref": "v1.0.0" } }
  ]
}
```

Supported source types are local paths and git repositories pinned to a branch, tag, or SHA. NPM packages and tarball/artifact URLs are documented extension points: the source sync layer can add resolvers that materialize those packages into `.harness/external-games` before the existing validation and generation steps run.

## Game Package Contract

Every game package must provide `daily-game.config.json` at its root. It declares the runtime adapter, Svelte component, content manifest, puzzle directory, optional assets directory, optional date index, and build mode.

Required package fields include `schemaVersion`, `contractVersion`, `game.id`, `game.slug`, `game.displayName`, `runtime.entry`, `ui.entry`, `content.manifest`, `content.puzzlesDir`, and `extension`.

Unknown top-level package fields are rejected. Unknown fields inside `extension` are allowed for game-owned metadata.

## Runtime Adapter Contract

Each package exposes a JavaScript-compatible runtime factory:

```ts
export type GameRuntimeFactory = () => Promise<GameRuntime>;
```

The runtime must implement `validateContent`, `validatePuzzle`, `createInitialState`, `submitGuess`, and `buildShareText`. The harness never infers answer correctness; it submits player input to the runtime and renders the returned state and feedback.

## Puzzle Resolution

Each content manifest declares `puzzleResolver`. The default fixture uses `static-pool`, where the browser computes the selected puzzle from the requested game date, pool version, pool size, and affine selector:

```json
{
  "mode": "static-pool",
  "timezone": "America/New_York",
  "startDate": "2026-01-01",
  "poolVersions": [
    {
      "version": "v1",
      "startDate": "2026-01-01",
      "poolSize": 1000,
      "pathPattern": "content/puzzles/v1/puzzle-{index:04}.json",
      "selector": { "type": "affine-permutation", "a": 137, "b": 431 },
      "cyclePolicy": "repeat"
    }
  ]
}
```

The harness also validates `dated-files` and `date-index` resolver shapes. `static-pool` does not require daily rebuilds and does not download a date index or the whole puzzle pool; the client fetches the manifest and the single resolved puzzle JSON. Static puzzle pools are not spoiler-proof because published JSON files are public static assets. Use obfuscation, delayed publishing, or a backend only if spoiler resistance is a hard product requirement.

## Archive Behavior

Each manifest declares `archive`. Rolling windows are computed in the browser from the game timezone, so archive lists update without daily regeneration. Games can use different windows, `includeToday`, `allowFutureDates`, and direct-access policies. `disabled` hides the archive list.

## Static Generation

Run the full pre-build pipeline:

```sh
cargo run -p daily_game_tools -- prepare-static-build
```

This validates the harness config, syncs games, validates package configs, verifies prebuilt artifacts or runs package build commands, discovers dates, validates content and puzzles, calls runtime validation hooks, generates `web/src/generated/game-registry.ts`, and copies public assets into `web/public/_games`.

Then build and verify the static output:

```sh
cd web
npm ci
npm run build
cd ..
cargo run -p daily_game_tools -- check-static-output --dist web/dist
```

The generated registry statically imports each configured Svelte view and runtime adapter. Do not manually edit `web/src/generated/game-registry.ts`.

`npm run build` runs Astro directly. The built site comes from `web/src/pages` and the Svelte components, not a hand-written static output script.

## Adding Games

To add a local package, add one config entry:

```json
{ "source": { "type": "local", "path": "../my-daily-game" } }
```

To add a git package, add:

```json
{ "source": { "type": "git", "repo": "https://github.com/example/my-daily-game.git", "ref": "v1.2.3" } }
```

The package must already conform to the package, content, puzzle, runtime, and Svelte view contracts.

To start a new package from a working skeleton, copy `templates/game-package/minimal-svelte-js` and rename the game IDs, slug, puzzles, runtime logic, and Svelte view.

## Local Development

Useful commands:

```sh
cargo test --workspace
cd web && npm run typecheck
cd web && npm run test
cargo run -p daily_game_tools -- prepare-static-build
cd web && npm run test:e2e
```

## Deployment

GitHub Pages is the default deployment target. `.github/workflows/deploy-pages.yml` builds from source, runs validation and tests, uploads `web/dist` as a Pages artifact, and deploys with `pages: write` and `id-token: write` permissions. Set `site.routePrefix` to `""` for owner pages or `"/repo-name"` for project pages so generated links and static content URLs work under both hosting modes.

S3-compatible deployment remains an optional secondary target through `scripts/deploy-s3.sh`; it does not change the game package contract.

See `docs/deployment.md` for GitHub Pages limitations and alternative static host options.

## Creating A Conforming Game Package

A package should provide:

- `daily-game.config.json` with runtime, UI, content, and date discovery metadata.
- A runtime adapter that implements the daily-game runtime v1 methods.
- A Svelte component that receives game, manifest, puzzle, state, latest evaluation, and `submitInput`.
- `content/manifest.json`, puzzle JSON files, optional `content/date-index.json`, and any public assets.
- Runtime assets under the configured runtime entry directory.

The game package owns its own puzzle-specific `extension` schema and validation. The harness validates only shared fields and delegates game-specific checks to the runtime.

The CLI validates JSON files against the schemas in `schemas/` before applying semantic checks. Schema errors include the file being validated and the failing JSON path.

## Tests

The test suite proves:

- Rust config, package, date-index, lockfile, static generation, and output validation behavior.
- Runtime contract behavior using the minimal fixture.
- Frontend loader, static-pool resolver, archive windows, localStorage, feedback, share, shell, and fixture view behavior.
- Playwright E2E gameplay, date-query routes, one-puzzle static-pool fetch behavior, refresh persistence, share output, missing-route UI, and static-server behavior.
- Adding a second fixture game through config generates registry entries and routes without manual imports.
