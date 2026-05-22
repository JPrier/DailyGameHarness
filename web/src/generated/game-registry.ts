import GameView_0 from '../../../fixtures/games/minimal-text-game/src/GameView.svelte';
import { createRuntime as createRuntime_0 } from '../../../fixtures/games/minimal-text-game/dist/runtime/index.js';
import GameView_1 from '../../../fixtures/games/second-minimal-game/src/GameView.svelte';
import { createRuntime as createRuntime_1 } from '../../../fixtures/games/second-minimal-game/dist/runtime/index.js';

export const generatedGameRegistry = {
  "minimal-text-game": { id: "minimal-text-game", slug: "minimal-text-game", displayName: "Minimal Text Game", category: "fixture", routePrefix: "", contentManifestUrl: "/_games/minimal-text-game/content/manifest.json", dateIndexUrl: null, puzzleBaseUrl: "/_games/minimal-text-game/content/puzzles", assetBaseUrl: "/_games/minimal-text-game/content/assets", runtimeAssetBaseUrl: "/_games/minimal-text-game/runtime", GameView: GameView_0, createRuntime: createRuntime_0, dates: [] },
  "second-minimal-game": { id: "second-minimal-game", slug: "second-minimal-game", displayName: "Second Minimal Game", category: "fixture", routePrefix: "", contentManifestUrl: "/_games/second-minimal-game/content/manifest.json", dateIndexUrl: null, puzzleBaseUrl: "/_games/second-minimal-game/content/puzzles", assetBaseUrl: "/_games/second-minimal-game/content/assets", runtimeAssetBaseUrl: "/_games/second-minimal-game/runtime", GameView: GameView_1, createRuntime: createRuntime_1, dates: [] },
} as const;
