# One-Shot Agent Spec: Generic Static Daily Game Harness

You are implementing a generic, reusable static daily-game harness.

This harness must be usable by anyone to host Wordle-style daily games. It must not hardcode any specific games. It must not implement real games inside the harness repository. It must provide a stable contract that independent game packages can implement.

The harness must support games that are developed as separate repositories/packages and pulled into the harness through configuration.

The expected model is:

- The harness repository owns:
  - the static site shell
  - game discovery
  - static route generation
  - generic UI shell
  - local progress persistence
  - share-result display
  - package validation
  - CI/CD
  - deployment
  - contract tests

- Each game package owns:
  - its own Rust game logic
  - its own Rust-to-WASM build
  - its own Svelte game view
  - its own puzzle/content files
  - its own assets
  - its own package config
  - its own game-specific validation rules

The harness must be able to consume any conforming game package without adding game-specific source files to the harness.

---

## 1. Hard technology constraints

Use:

- Rust for game logic contracts, validation tooling, static generation tooling, and shared type definitions.
- Rust compiled to WebAssembly for game runtime logic.
- `wasm-bindgen` or an equivalent Rust/WASM binding layer for the game package runtime.
- Astro for the static site shell.
- Svelte for DOM/game-state management.
- TypeScript for frontend glue code.
- GitHub Actions for CI.
- Static deployment to object storage.
- S3 as the first supported deploy target.

The built site must be static. Runtime gameplay must not require a backend.

The browser must not perform live network calls for answer validation, scoring, or clue generation. Everything required for a given puzzle must be available through static files bundled into the site output.

---

## 2. Primary goal

Build the generic harness only.

Do not implement real games.

Do not hardcode a list of games.

Do not create game-specific Rust modules inside the harness.

Do not create game-specific Svelte pages inside the harness.

Do not mention or depend on any particular puzzle concept.

The only game package that may exist in this repository is a minimal contract fixture used to prove the harness works. It must be named generically, such as:

```text
fixtures/games/minimal-text-game
```

This fixture exists only for tests and documentation. It is not a real product game.

---

## 3. Expected end-state behavior

After implementation, a user can clone the harness repo, add one or more external game packages to a config file, run the build, and receive a static site containing those games.

The user can visit:

```text
/
```

and see all configured games.

The user can visit:

```text
/games/{gameSlug}
```

and play today’s puzzle for that configured game.

The user can visit:

```text
/games/{gameSlug}/{yyyy-mm-dd}
```

and play or review that specific puzzle date, if available.

For each configured game:

- The harness reads the game package config.
- The harness validates the game package against the shared contract.
- The harness statically generates routes for available puzzle dates.
- The harness copies the game’s static content, WASM runtime assets, UI assets, and puzzle assets into the final site output.
- The page loads the game’s runtime adapter.
- The page loads the selected puzzle JSON.
- The game runtime validates the puzzle before play starts.
- The player can submit guesses.
- The game runtime evaluates guesses.
- The generic shell stores progress in localStorage.
- Refreshing the page restores progress.
- The result/share UI works through the generic shell.
- The game-specific Svelte component renders clues and any custom visual UI.
- The game-specific component never decides correctness unless that decision happens by calling the game package runtime.

---

## 4. Package model

Games are independent packages.

A game package may be consumed through one of these source types:

```text
local path
git repository
npm package
tarball/artifact URL
```

The harness must support at least:

```text
local path
git repository pinned to a branch/tag/SHA
```

NPM package support may be implemented now or left as a documented extension point, but the architecture must not prevent it.

The harness must use a single root config file to declare the games to include.

Example:

```json
{
  "schemaVersion": "daily-game-harness.v1",
  "site": {
    "name": "Daily Games",
    "baseUrl": "https://example.com",
    "routePrefix": ""
  },
  "games": [
    {
      "source": {
        "type": "local",
        "path": "../my-daily-game"
      }
    },
    {
      "source": {
        "type": "git",
        "repo": "https://github.com/example/some-game.git",
        "ref": "v1.2.3"
      }
    }
  ],
  "staticGeneration": {
    "outputDir": "web/dist",
    "generatedDir": ".harness/generated",
    "externalGamesDir": ".harness/external-games"
  },
  "deployment": {
    "target": "s3",
    "bucketEnv": "DAILY_GAMES_BUCKET",
    "regionEnv": "AWS_REGION",
    "cloudFrontDistributionIdEnv": "CLOUDFRONT_DISTRIBUTION_ID"
  }
}
```

Adding a game must require only adding one entry to this config, assuming the game package already conforms to the contract.

---

## 5. Game package contract

Every game package must include a package config file at its root:

```text
daily-game.config.json
```

This file is the primary contract between the harness and the game.

Example:

```json
{
  "schemaVersion": "daily-game-package.v1",
  "contractVersion": "daily-game-runtime.v1",
  "game": {
    "id": "minimal-text-game",
    "slug": "minimal-text-game",
    "displayName": "Minimal Text Game",
    "shortDescription": "A tiny fixture game used to validate the harness.",
    "category": "fixture",
    "status": "playable"
  },
  "runtime": {
    "type": "wasm-js-adapter",
    "entry": "dist/runtime/index.js",
    "wasm": "dist/runtime/game_bg.wasm",
    "exports": {
      "createRuntime": "createRuntime"
    }
  },
  "ui": {
    "type": "svelte-component",
    "entry": "src/GameView.svelte",
    "exportName": "default"
  },
  "content": {
    "manifest": "content/manifest.json",
    "puzzlesDir": "content/puzzles",
    "assetsDir": "content/assets",
    "dateIndex": "content/date-index.json"
  },
  "staticGeneration": {
    "dateDiscovery": {
      "mode": "date-index",
      "path": "content/date-index.json"
    },
    "prerender": true,
    "includeAssets": true
  },
  "build": {
    "commands": [
      "cargo build --release --target wasm32-unknown-unknown",
      "npm run build:runtime"
    ],
    "distDir": "dist"
  },
  "extension": {}
}
```

Rules:

- `schemaVersion` is required.
- `contractVersion` is required.
- `game.id` is required and must be stable.
- `game.slug` is required and must be URL-safe.
- `runtime.entry` is required.
- `ui.entry` is required.
- `content.manifest` is required.
- `content.puzzlesDir` is required.
- `staticGeneration.dateDiscovery` is required.
- `extension` is required and may contain game-specific package metadata.
- Unknown top-level fields are rejected.
- Unknown fields inside `extension` are allowed.

---

## 6. Game content contract

Each game package owns its own content format, but the harness needs enough standardization to statically generate routes and load puzzles.

Each game package must provide a content manifest.

Example:

```json
{
  "schemaVersion": "daily-game-content-manifest.v1",
  "gameId": "minimal-text-game",
  "defaultMaxGuesses": 6,
  "inputModes": ["text"],
  "share": {
    "emoji": "🎮",
    "includeGuessCount": true,
    "includeDate": true
  },
  "puzzleSchema": {
    "type": "game-owned",
    "path": "content/schemas/puzzle.schema.json"
  },
  "extension": {}
}
```

Each game package must provide a date index.

Example:

```json
{
  "schemaVersion": "daily-game-date-index.v1",
  "gameId": "minimal-text-game",
  "dates": [
    {
      "date": "2026-01-01",
      "puzzlePath": "content/puzzles/2026-01-01.json",
      "assetsPrefix": "content/assets/2026-01-01"
    }
  ]
}
```

Each puzzle file must contain at least these shared fields:

```json
{
  "schemaVersion": "daily-game-puzzle.v1",
  "gameId": "minimal-text-game",
  "puzzleId": "minimal-text-game-2026-01-01",
  "date": "2026-01-01",
  "seed": "minimal-text-game:2026-01-01:v1",
  "display": {
    "title": "Puzzle for 2026-01-01",
    "initialPrompt": "Guess the hidden answer."
  },
  "extension": {}
}
```

Rules:

- The harness validates only the common fields.
- The game runtime validates the game-specific `extension`.
- Unknown top-level puzzle fields are rejected by the harness schema unless explicitly allowed.
- Unknown fields inside `extension` are allowed.
- The harness must not assume what is inside `extension`.

This gives standardization without forcing every game into the same clue model.

---

## 7. Runtime adapter contract

Each game package must expose a JavaScript runtime adapter. The adapter may be a thin wrapper around Rust/WASM, but the public interface must be JavaScript/TypeScript-compatible so the generic Svelte shell can call it.

Required TypeScript interface:

```ts
export type GameRuntimeFactory = () => Promise<GameRuntime>;

export interface GameRuntime {
  contractVersion: "daily-game-runtime.v1";

  validateContent(args: {
    packageConfig: unknown;
    contentManifest: unknown;
    dateIndex: unknown;
  }): Promise<ValidationResult>;

  validatePuzzle(args: {
    contentManifest: unknown;
    puzzle: unknown;
  }): Promise<ValidationResult>;

  createInitialState(args: {
    contentManifest: unknown;
    puzzle: unknown;
    date: string;
  }): Promise<GameState>;

  submitGuess(args: {
    contentManifest: unknown;
    puzzle: unknown;
    state: GameState;
    input: PlayerInput;
  }): Promise<SubmitGuessResult>;

  buildShareText(args: {
    contentManifest: unknown;
    puzzle: unknown;
    state: GameState;
  }): Promise<string>;

  getAutocompleteSuggestions?(args: {
    contentManifest: unknown;
    puzzle: unknown;
    query: string;
    limit: number;
  }): Promise<AutocompleteSuggestion[]>;
}
```

Shared types:

```ts
export type ValidationResult =
  | {
      ok: true;
      warnings: ValidationWarning[];
    }
  | {
      ok: false;
      errors: ValidationError[];
      warnings: ValidationWarning[];
    };

export type PlayerInput =
  | {
      kind: "text";
      value: string;
    }
  | {
      kind: "point";
      x: number;
      y: number;
      metadata?: Record<string, unknown>;
    }
  | {
      kind: "choice";
      value: string;
      metadata?: Record<string, unknown>;
    };

export type GameState = {
  schemaVersion: "daily-game-state.v1";
  gameId: string;
  puzzleId: string;
  date: string;
  status: "in_progress" | "won" | "lost";
  guessCount: number;
  maxGuesses: number;
  currentStage: number;
  publicState: Record<string, unknown>;
  privateState?: never;
};

export type SubmitGuessResult = {
  state: GameState;
  evaluation: GuessEvaluation;
};

export type GuessEvaluation = {
  outcome: "correct" | "incorrect" | "invalid";
  consumedGuess: boolean;
  message?: string;
  feedback: FeedbackItem[];
  reveal?: RevealInstruction;
};

export type FeedbackItem = {
  key: string;
  label: string;
  kind: "text" | "number" | "direction" | "distance" | "comparison" | "boolean" | "custom";
  value: unknown;
  severity: "neutral" | "good" | "warning" | "bad";
};

export type RevealInstruction = {
  stage: number;
  assets?: string[];
  text?: string;
  metadata?: Record<string, unknown>;
};

export type AutocompleteSuggestion = {
  id: string;
  label: string;
  subtitle?: string;
  metadata?: Record<string, unknown>;
};

export type ValidationError = {
  code: string;
  message: string;
  path?: string;
};

export type ValidationWarning = {
  code: string;
  message: string;
  path?: string;
};
```

Rules:

- The adapter must return structured errors.
- The adapter must not throw raw unstructured exceptions for normal validation failures.
- Guess correctness must be determined by the game runtime.
- The generic harness must not infer correctness.
- The generic harness may display feedback, persist state, and manage routing.
- The runtime must be deterministic for the same puzzle, state, and input.

---

## 8. Rust/WASM package SDK

Create a small SDK in the harness repo that game packages can depend on.

The SDK should be publishable later, but local workspace use is acceptable for the initial implementation.

Suggested packages:

```text
crates/daily_game_core
crates/daily_game_wasm_support
crates/daily_game_tools
```

`daily_game_core` owns shared serializable types.

`daily_game_wasm_support` owns helpers for exposing the runtime adapter from Rust/WASM.

`daily_game_tools` owns harness-side CLI commands.

The SDK must not contain game-specific logic.

The SDK should make it easy for a game package to implement the runtime contract in Rust and expose it through WASM.

---

## 9. Harness repository structure

Implement this structure:

```text
.
├── Cargo.toml
├── harness.config.json
├── crates/
│   ├── daily_game_core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs
│   │       ├── validation.rs
│   │       ├── errors.rs
│   │       └── share.rs
│   ├── daily_game_wasm_support/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── bindings.rs
│   └── daily_game_tools/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── config.rs
│           ├── discover.rs
│           ├── fetch.rs
│           ├── validate.rs
│           ├── generate.rs
│           ├── static_check.rs
│           └── lockfile.rs
├── schemas/
│   ├── harness-config.schema.json
│   ├── game-package-config.schema.json
│   ├── content-manifest.schema.json
│   ├── date-index.schema.json
│   ├── puzzle-common.schema.json
│   ├── game-state.schema.json
│   └── runtime-result.schema.json
├── fixtures/
│   └── games/
│       └── minimal-text-game/
│           ├── daily-game.config.json
│           ├── Cargo.toml
│           ├── package.json
│           ├── src/
│           │   ├── lib.rs
│           │   └── GameView.svelte
│           ├── content/
│           │   ├── manifest.json
│           │   ├── date-index.json
│           │   ├── puzzles/
│           │   │   └── 2026-01-01.json
│           │   └── assets/
│           └── tests/
├── web/
│   ├── package.json
│   ├── astro.config.mjs
│   ├── tsconfig.json
│   ├── src/
│   │   ├── pages/
│   │   │   ├── index.astro
│   │   │   └── games/
│   │   │       └── [gameSlug]/
│   │   │           ├── index.astro
│   │   │           └── [date].astro
│   │   ├── generated/
│   │   │   └── game-registry.ts
│   │   ├── components/
│   │   │   ├── GameShell.svelte
│   │   │   ├── GenericFeedback.svelte
│   │   │   ├── GuessInput.svelte
│   │   │   ├── ResultModal.svelte
│   │   │   ├── ShareButton.svelte
│   │   │   └── ErrorPanel.svelte
│   │   ├── lib/
│   │   │   ├── game-registry.ts
│   │   │   ├── game-loader.ts
│   │   │   ├── local-state.ts
│   │   │   ├── dates.ts
│   │   │   ├── runtime.ts
│   │   │   └── types.ts
│   │   └── styles/
│   │       └── global.css
│   └── tests/
│       ├── unit/
│       └── e2e/
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── deploy.yml
├── scripts/
│   ├── sync-games.sh
│   ├── generate-static-registry.sh
│   ├── validate-games.sh
│   ├── build-static.sh
│   └── deploy-s3.sh
└── README.md
```

The file `web/src/generated/game-registry.ts` must be generated by the harness tooling. It must not be manually edited.

---

## 10. Static generation design

Static generation is central to this project.

The harness must perform static generation in a deterministic multi-step pipeline.

### Step 1: Read harness config

Read:

```text
harness.config.json
```

Validate it against:

```text
schemas/harness-config.schema.json
```

### Step 2: Sync external game packages

For each configured game source:

- local path: verify the path exists
- git source: clone or update into `.harness/external-games/{safeName}`
- pin the resolved commit SHA in a lockfile
- reject dirty or ambiguous external state unless explicitly allowed

Generate or update:

```text
harness.lock.json
```

Example:

```json
{
  "schemaVersion": "daily-game-harness-lock.v1",
  "games": [
    {
      "sourceType": "git",
      "repo": "https://github.com/example/some-game.git",
      "requestedRef": "v1.2.3",
      "resolvedSha": "abc123",
      "localPath": ".harness/external-games/some-game"
    }
  ]
}
```

### Step 3: Read each game package config

For each synced package, read:

```text
daily-game.config.json
```

Validate it against:

```text
schemas/game-package-config.schema.json
```

Reject:

- duplicate game IDs
- duplicate slugs
- missing runtime entry
- missing UI entry
- missing content manifest
- missing date index
- unsupported contract versions
- paths escaping the package root

### Step 4: Build each game package

Each game package may provide build commands in its config.

The harness must support two modes:

```text
prebuilt
build-from-source
```

In `build-from-source` mode, run the package’s configured build commands from the package root.

In `prebuilt` mode, verify the configured runtime and UI files already exist.

The harness must fail if required files are missing after build.

### Step 5: Discover puzzle dates

Each game config defines its date discovery mode.

Supported modes:

#### `date-index`

Read the configured date index file.

```json
{
  "dateDiscovery": {
    "mode": "date-index",
    "path": "content/date-index.json"
  }
}
```

#### `glob`

Discover puzzle files by glob.

```json
{
  "dateDiscovery": {
    "mode": "glob",
    "pattern": "content/puzzles/*.json",
    "dateFromFilename": true
  }
}
```

#### `command`

Run a package-owned command that emits JSON to stdout.

```json
{
  "dateDiscovery": {
    "mode": "command",
    "command": "npm run export-dates --silent"
  }
}
```

The command output must match the date-index schema.

The initial implementation must support `date-index`. `glob` and `command` may be implemented now or documented as extension points, but the code structure must leave room for them.

### Step 6: Validate content

For each game:

- validate content manifest common fields
- validate date index common fields
- validate every listed puzzle common fields
- call the game runtime’s `validateContent`
- call the game runtime’s `validatePuzzle` for every included puzzle
- verify all referenced static assets exist
- verify dates are valid ISO dates
- verify puzzle date matches date-index entry
- verify puzzle game ID matches package config game ID

### Step 7: Generate static registry

Generate:

```text
web/src/generated/game-registry.ts
```

This file must contain static imports for every configured game UI component and runtime adapter.

Example shape:

```ts
import GameView_0 from "../../../.harness/external-games/minimal-text-game/src/GameView.svelte";
import { createRuntime as createRuntime_0 } from "../../../.harness/external-games/minimal-text-game/dist/runtime/index.js";

export const generatedGameRegistry = {
  "minimal-text-game": {
    id: "minimal-text-game",
    slug: "minimal-text-game",
    displayName: "Minimal Text Game",
    category: "fixture",
    contentManifestUrl: "/_games/minimal-text-game/content/manifest.json",
    dateIndexUrl: "/_games/minimal-text-game/content/date-index.json",
    puzzleBaseUrl: "/_games/minimal-text-game/content/puzzles",
    assetBaseUrl: "/_games/minimal-text-game/content/assets",
    runtimeAssetBaseUrl: "/_games/minimal-text-game/runtime",
    GameView: GameView_0,
    createRuntime: createRuntime_0,
    dates: ["2026-01-01"]
  }
} as const;
```

The exact import paths may differ, but the generated registry must allow Astro/Vite to statically bundle the configured Svelte components and runtime adapters.

The harness must not require manually editing this generated file.

### Step 8: Copy public static files

Copy each game’s content and runtime assets into the web public directory before Astro build.

Suggested output:

```text
web/public/_games/{gameSlug}/content/manifest.json
web/public/_games/{gameSlug}/content/date-index.json
web/public/_games/{gameSlug}/content/puzzles/{date}.json
web/public/_games/{gameSlug}/content/assets/...
web/public/_games/{gameSlug}/runtime/...
```

Rules:

- do not copy source files unnecessarily
- do not expose private package files
- preserve relative asset references where possible
- reject path traversal
- reject symlink escapes
- ensure all copied content is deterministic

### Step 9: Astro static path generation

Astro routes must use the generated registry.

For:

```text
web/src/pages/games/[gameSlug]/index.astro
```

`getStaticPaths` must generate one route per configured game.

For:

```text
web/src/pages/games/[gameSlug]/[date].astro
```

`getStaticPaths` must generate one route per configured game/date pair.

The date-less route should resolve to today’s puzzle client-side or to the latest available puzzle according to the configured behavior.

The date route should be fully static.

### Step 10: Static output verification

After `astro build`, run a static checker.

It must verify:

- `dist/index.html` exists
- every configured game route exists
- every configured game/date route exists
- every copied content manifest exists
- every copied date index exists
- every listed puzzle file exists
- every listed asset exists
- every runtime adapter asset exists
- no generated route references a missing file
- no generated registry entry points outside allowed package paths
- the build output can be served by a dumb static server

---

## 11. Generic frontend shell behavior

The generic shell owns:

- loading content manifest
- loading date index
- loading selected puzzle JSON
- creating runtime
- validating puzzle
- creating initial state
- restoring localStorage state
- submitting guesses
- storing updated state
- rendering generic feedback
- result modal
- share button
- error handling
- missing game UI
- missing puzzle UI

The game-specific Svelte component owns:

- visual clue rendering
- specialized input UI if needed
- game-specific display of feedback metadata
- game-specific explanation layout

The generic shell passes these props to the game view:

```ts
export type GameViewProps = {
  game: RegisteredGame;
  contentManifest: unknown;
  puzzle: unknown;
  state: GameState;
  latestEvaluation: GuessEvaluation | null;
  submitInput: (input: PlayerInput) => Promise<void>;
};
```

The game view must not mutate state directly. It must call `submitInput`.

---

## 12. Local persistence

Use localStorage.

Key format:

```text
daily-game:{contractVersion}:{gameId}:{puzzleId}
```

Rules:

- state must be JSON
- state must include gameId, puzzleId, date, status, guessCount, maxGuesses, currentStage, and publicState
- state must never contain private answer-only data unless the game runtime intentionally puts it in publicState after completion
- mismatched gameId/puzzleId/date must be discarded
- invalid state schema must be discarded
- state from unsupported schema versions must be discarded
- completed games remain stored

---

## 13. Share result behavior

The generic shell calls:

```ts
runtime.buildShareText(...)
```

The harness must not build game-specific share results itself.

The harness may provide a fallback generic share text only when the runtime returns a structured “share unsupported” response.

Share text must be spoiler-safe unless the runtime explicitly marks the puzzle as completed and intentionally includes answer data.

---

## 14. CLI commands

Implement these commands in `daily_game_tools`.

```text
daily-game-tools validate-harness-config
daily-game-tools sync-games
daily-game-tools validate-games
daily-game-tools generate-static-registry
daily-game-tools prepare-public-assets
daily-game-tools check-static-output --dist web/dist
daily-game-tools build-report
```

Also provide a single convenience command:

```text
daily-game-tools prepare-static-build
```

This command must run:

1. validate harness config
2. sync games
3. validate game package configs
4. build games if configured
5. discover dates
6. validate content
7. generate registry
8. copy public assets

The frontend build should then run:

```text
npm run build
```

The static checker should then run:

```text
daily-game-tools check-static-output --dist web/dist
```

---

## 15. Required tests

Do not claim the implementation is complete until all tests below exist and pass.

### 15.1 Rust unit tests

Test harness config validation:

- accepts valid local game source
- accepts valid git game source
- rejects unknown source type
- rejects missing source path
- rejects duplicate game entries when resolved package configs duplicate IDs
- rejects invalid route prefix

Test game package config validation:

- accepts valid fixture package config
- rejects missing game ID
- rejects missing slug
- rejects non-URL-safe slug
- rejects missing runtime entry
- rejects missing UI entry
- rejects missing content manifest
- rejects missing date index
- rejects unsupported contract version
- rejects paths escaping package root
- allows unknown fields inside extension

Test date discovery:

- reads date-index mode
- rejects invalid date
- rejects duplicate date
- rejects missing puzzle path
- rejects path traversal
- rejects date-index game ID mismatch

Test static generation:

- generated registry contains fixture game
- generated registry contains static imports
- generated registry contains correct content URLs
- generated registry contains all fixture dates
- public asset preparation copies manifest
- public asset preparation copies date index
- public asset preparation copies puzzle files
- public asset preparation copies runtime files
- public asset preparation rejects symlink escape
- public asset preparation is deterministic

Test lockfile behavior:

- local source records resolved path
- git source records requested ref and resolved SHA
- changed git ref updates lockfile
- lockfile schema validates

### 15.2 Runtime contract tests

Use the fixture game package.

Test:

- runtime adapter can be imported from generated registry
- `validateContent` accepts valid fixture content
- `validatePuzzle` accepts valid fixture puzzle
- `validatePuzzle` rejects invalid fixture puzzle
- `createInitialState` returns valid shared state
- `submitGuess` returns incorrect result for wrong guess
- `submitGuess` returns correct result for correct guess
- invalid guess does not consume guess
- guess after completion returns structured error or unchanged terminal state according to contract
- `buildShareText` returns non-empty spoiler-safe text
- all runtime calls are deterministic for same inputs

### 15.3 Frontend unit tests

Test:

- home page registry rendering logic lists configured games
- game loader resolves content manifest URL
- game loader resolves date index URL
- game loader resolves puzzle URL
- date-less route picks a stable date according to configured behavior
- explicit date route uses the requested date
- localStorage save/load round-trips valid state
- localStorage rejects mismatched game ID
- localStorage rejects mismatched puzzle ID
- localStorage rejects unsupported state schema
- generic feedback renderer handles every shared feedback kind
- share button calls runtime share function
- shell does not mark a guess correct unless runtime returns correct

### 15.4 Svelte component tests

Test `GameShell.svelte` with the fixture game:

- renders loading state
- renders loaded puzzle
- submits text input
- displays feedback after wrong guess
- advances public state after wrong guess
- displays win result
- displays loss result
- disables input after completion
- restores progress after simulated reload
- displays friendly error when runtime validation fails

Test fixture game view:

- receives props from shell
- renders puzzle prompt
- calls `submitInput`
- does not mutate state directly

### 15.5 Playwright E2E tests

Run against the built Astro site.

Required tests:

1. Home page lists the fixture game.
2. Fixture game route loads without console errors.
3. Fixture date route loads without console errors.
4. Player can win the fixture puzzle.
5. Player can lose the fixture puzzle.
6. Refresh after one wrong guess preserves progress.
7. Share button produces non-empty text.
8. Invalid game route shows friendly not-found UI.
9. Missing puzzle date shows friendly puzzle-not-found UI.
10. Built site works when served by a static file server.

### 15.6 Static output tests

After static build, verify:

- `dist/index.html` exists
- fixture game route exists
- fixture game/date route exists
- copied content manifest exists
- copied date index exists
- copied puzzle exists
- copied runtime assets exist
- generated JS/CSS assets exist
- no route references missing files
- no generated URL points to the source package path
- no build output contains `.git`
- no build output contains private harness cache files
- static server can serve all generated routes

### 15.7 “Add a game by config only” proof test

Create a second minimal fixture game package under:

```text
fixtures/games/second-minimal-game
```

Do not manually add any source-code imports for it.

Create a test that:

1. writes a temporary harness config containing both fixture games
2. runs `prepare-static-build`
3. verifies the generated registry contains both games
4. verifies both games appear on the home page
5. verifies both game routes exist
6. verifies no manually maintained registry file was edited

This test proves the harness is truly config-driven.

---

## 16. CI requirements

Create:

```text
.github/workflows/ci.yml
```

It must run on:

```yaml
on:
  pull_request:
  push:
    branches: [main]
```

CI jobs:

### `rust`

Run:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### `frontend`

Run:

```text
cd web
npm ci
npm run typecheck
npm run test
```

### `static-build`

Run:

```text
cargo run -p daily_game_tools -- prepare-static-build
cd web
npm ci
npm run build
cd ..
cargo run -p daily_game_tools -- check-static-output --dist web/dist
```

### `e2e`

Run:

```text
cd web
npm ci
npm run test:e2e
```

### `artifact`

Upload:

```text
web/dist
```

as a GitHub Actions artifact.

CI must fail on:

- formatting failure
- Clippy warning
- Rust test failure
- frontend type error
- frontend unit test failure
- package validation failure
- static generation failure
- Astro build failure
- static output check failure
- Playwright failure

---

## 17. Deployment requirements

Create:

```text
.github/workflows/deploy.yml
```

It must support S3 deployment.

Rules:

- build from source
- run all validation before upload
- upload only `web/dist`
- use GitHub OIDC for AWS credentials
- do not require long-lived AWS access keys
- require configurable:
  - AWS role ARN
  - AWS region
  - S3 bucket
  - optional CloudFront distribution ID
- use long cache headers for hashed assets
- use short cache headers for HTML and JSON puzzle files
- optionally invalidate CloudFront HTML/JSON paths

The deployment code must be isolated enough that another static object storage target can be added later.

---

## 18. README requirements

README must document:

- what the harness is
- what the harness is not
- the package model
- the runtime adapter contract
- the game package config file
- how static generation works
- how to add a local game package
- how to add a git game package
- how to run validation
- how to run local development
- how to build static output
- how to deploy to S3
- how to create a conforming game package
- what tests prove the harness is working

The README must explicitly state:

```text
The harness does not implement games. Games are external packages that conform to the daily-game package contract.
```

---

## 19. Acceptance criteria

The implementation is complete only when all of this is true:

1. The harness repo contains no hardcoded real game list.
2. The harness repo contains no real game implementations.
3. The fixture game exists only for contract testing.
4. Games are discovered from `harness.config.json`.
5. Game package configs are validated.
6. Game content manifests are validated.
7. Date indexes are validated.
8. Static routes are generated from configured packages.
9. The generated registry is created automatically.
10. Svelte game components are imported through generated registry code.
11. Runtime adapters are imported through generated registry code.
12. The generic shell can play the fixture game.
13. Adding the second fixture game requires only config changes.
14. Static output contains all expected routes and assets.
15. CI validates Rust, frontend, game packages, static generation, and E2E.
16. Deployment workflow uploads static output to S3.
17. README explains the package contract clearly.
18. All required tests pass.

---

## 20. Implementation order

Use this order:

1. Create Rust workspace.
2. Create shared type crate.
3. Create tools crate.
4. Define JSON schemas.
5. Implement harness config validation.
6. Implement game package config validation.
7. Implement local source discovery.
8. Implement git source discovery and lockfile.
9. Implement date-index discovery.
10. Implement fixture game package.
11. Implement runtime contract types.
12. Implement minimal fixture runtime in Rust/WASM.
13. Implement Astro/Svelte shell.
14. Implement generated registry pipeline.
15. Implement public asset preparation.
16. Implement date route generation.
17. Implement generic gameplay shell.
18. Implement localStorage persistence.
19. Implement share UI.
20. Implement static output checker.
21. Implement second fixture game.
22. Implement “add game by config only” proof test.
23. Implement CI.
24. Implement S3 deploy workflow.
25. Write README.
26. Run all tests.
27. Fix all failures.
28. Produce final report.

Do not reduce scope because this is large. The goal is to build a durable generic harness, not a demo.

---

## 21. Final report required

At completion, produce:

```text
Implemented:
- ...

Architecture:
- ...

Package contract:
- ...

Static generation pipeline:
- ...

Validation implemented:
- ...

Tests implemented:
- ...

Commands run:
- command: ...
  result: pass/fail
  evidence: ...

How to add a game:
- ...

Known limitations:
- ...

Next recommended work:
- ...
```

If anything fails, state exactly what failed. Do not claim the harness is complete without passing validation and test evidence.
