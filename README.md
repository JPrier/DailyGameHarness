# Daily Game Harness

The harness does not implement games. Games are external packages that conform to the daily-game package contract.

## What this is
A generic static harness for config-driven daily games with Rust tools + static web output.

## What this is not
Not a real game repository and not a place for game-specific logic.

## Commands
- `cargo run -p daily_game_tools -- validate-harness-config`
- `cargo run -p daily_game_tools -- sync-games`
- `cargo run -p daily_game_tools -- validate-games`
- `cargo run -p daily_game_tools -- generate-static-registry`
- `cargo run -p daily_game_tools -- prepare-public-assets`
- `cargo run -p daily_game_tools -- prepare-static-build`
- `cargo run -p daily_game_tools -- check-static-output --dist web/dist`

## Add a game
Edit `harness.config.json` and append a `games[].source` entry (local or git).

## Build
1. `cargo run -p daily_game_tools -- prepare-static-build`
2. `cd web && npm ci && npm run build`
3. `cargo run -p daily_game_tools -- check-static-output --dist web/dist`

## Deploy
Use `.github/workflows/deploy.yml` (OIDC + S3 upload of `web/dist`).
