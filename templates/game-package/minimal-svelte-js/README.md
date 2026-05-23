# Minimal Svelte JS Game Package

Copy this directory to start a new external game package.

Steps:

1. Rename `game.id`, `game.slug`, `game.displayName`, and all `gameId` fields.
2. Replace the sample puzzle files in `content/puzzles/v1`.
3. Implement game-specific validation and scoring in `dist/runtime/index.js`.
4. Implement the game UI in `src/GameView.svelte`.
5. Add the package to `harness.config.json` with a local or git source.
6. Run `cargo run -p daily_game_tools -- prepare-static-build`.

The starter uses static-pool mode, so no date index or daily rebuild is required.
