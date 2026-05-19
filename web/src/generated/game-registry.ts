import GameView_0 from '../../../fixtures/games/minimal-text-game/src/GameView.svelte';
import { createRuntime as createRuntime_0 } from '../../../fixtures/games/minimal-text-game/dist/runtime/index.js';

export const generatedGameRegistry = {
  "minimal-text-game": { id: "minimal-text-game", slug: "minimal-text-game", displayName: "Minimal Text Game", category: "fixture", contentManifestUrl: "/_games/minimal-text-game/content/manifest.json", dateIndexUrl: "/_games/minimal-text-game/content/date-index.json", puzzleBaseUrl: "/_games/minimal-text-game/content/puzzles", assetBaseUrl: "/_games/minimal-text-game/content/assets", runtimeAssetBaseUrl: "/_games/minimal-text-game/runtime", GameView: GameView_0, createRuntime: createRuntime_0, dates: ["2026-01-01"] },
} as const;
