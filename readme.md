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

Every game package must provide `daily-game.config.json` at its root. It declares the runtime adapter, Svelte component, content manifest, puzzle directory, date index, and static generation settings.

Required package fields include `schemaVersion`, `contractVersion`, `game.id`, `game.slug`, `game.displayName`, `runtime.entry`, `ui.entry`, `content.manifest`, `content.puzzlesDir`, `content.dateIndex`, `staticGeneration.dateDiscovery`, and `extension`.

Unknown top-level package fields are rejected. Unknown fields inside `extension` are allowed for game-owned metadata.

## Runtime Adapter Contract

Each package exposes a JavaScript-compatible runtime factory:

```ts
export type GameRuntimeFactory = () => Promise<GameRuntime>;
```

The runtime must implement `validateContent`, `validatePuzzle`, `createInitialState`, `submitGuess`, and `buildShareText`. The harness never infers answer correctness; it submits player input to the runtime and renders the returned state and feedback.

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

The S3 deployment workflow builds from source, runs validation, uploads only `web/dist`, uses GitHub OIDC through `aws-actions/configure-aws-credentials`, applies long cache headers to static assets, short cache headers to HTML/JSON, and optionally invalidates CloudFront.

Required GitHub secrets are `AWS_ROLE_ARN`, `AWS_REGION`, and `S3_BUCKET`. `CLOUDFRONT_DISTRIBUTION_ID` is optional.

## Creating A Conforming Game Package

A package should provide:

- `daily-game.config.json` with runtime, UI, content, and date discovery metadata.
- A runtime adapter that implements the daily-game runtime v1 methods.
- A Svelte component that receives game, manifest, puzzle, state, latest evaluation, and `submitInput`.
- `content/manifest.json`, `content/date-index.json`, puzzle JSON files, and any public assets.
- Runtime assets under the configured runtime entry directory.

The game package owns its own puzzle-specific `extension` schema and validation. The harness validates only shared fields and delegates game-specific checks to the runtime.

## Tests

The test suite proves:

- Rust config, package, date-index, lockfile, static generation, and output validation behavior.
- Runtime contract behavior using the minimal fixture.
- Frontend loader, date, localStorage, feedback, share, shell, and fixture view behavior.
- Playwright E2E gameplay, refresh persistence, share output, missing-route UI, and static-server behavior.
- Adding a second fixture game through config generates registry entries and routes without manual imports.
