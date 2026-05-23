export type PlayerInput =
  | { kind: 'text'; value: string }
  | { kind: 'point'; x: number; y: number; metadata?: Record<string, unknown> }
  | { kind: 'choice'; value: string; metadata?: Record<string, unknown> };

export type GameState = {
  schemaVersion: 'daily-game-state.v1';
  gameId: string;
  puzzleId: string;
  date: string;
  status: 'in_progress' | 'won' | 'lost';
  guessCount: number;
  maxGuesses: number;
  currentStage: number;
  publicState: Record<string, unknown>;
};

export type ValidationResult =
  | { ok: true; warnings: ValidationWarning[] }
  | { ok: false; errors: ValidationError[]; warnings: ValidationWarning[] };

export type ValidationError = { code: string; message: string; path?: string };
export type ValidationWarning = { code: string; message: string; path?: string };

export type FeedbackItem = {
  key: string;
  label: string;
  kind: 'text' | 'number' | 'direction' | 'distance' | 'comparison' | 'boolean' | 'custom';
  value: unknown;
  severity: 'neutral' | 'good' | 'warning' | 'bad';
};

export type GuessEvaluation = {
  outcome: 'correct' | 'incorrect' | 'invalid';
  consumedGuess: boolean;
  message?: string;
  feedback: FeedbackItem[];
};

export type SubmitGuessResult = {
  state: GameState;
  evaluation: GuessEvaluation;
};

export type GameRuntime = {
  contractVersion: 'daily-game-runtime.v1';
  validateContent(args: {
    packageConfig: unknown;
    contentManifest: unknown;
    dateIndex: unknown;
  }): Promise<ValidationResult>;
  validatePuzzle(args: { contentManifest: unknown; puzzle: unknown }): Promise<ValidationResult>;
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
  }): Promise<string | { unsupported: true; reason?: string }>;
};

export type AffineSelector = {
  type: 'affine-permutation';
  a: number;
  b: number;
};

export type PuzzleResolverConfig =
  | {
      mode: 'static-pool';
      timezone: string;
      startDate: string;
      poolVersions: Array<{
        version: string;
        startDate: string;
        poolSize: number;
        pathPattern: string;
        assetPathPattern?: string;
        selector: AffineSelector | { type: 'seeded-shuffle'; seed: string };
        cyclePolicy: 'repeat' | 'error-after-exhaustion' | 'next-version-required';
      }>;
    }
  | { mode: 'dated-files'; timezone: string; pathPattern: string; assetPathPattern?: string }
  | { mode: 'date-index'; timezone: string; indexPath: string };

export type ArchiveConfig =
  | {
      mode: 'rolling-window';
      days: number;
      includeToday: boolean;
      allowFutureDates: boolean;
      directAccess?: 'within-archive-window' | 'any-resolvable-date' | 'disabled';
    }
  | {
      mode: 'fixed-list';
      dates: string[];
      allowFutureDates?: boolean;
      directAccess?: 'within-archive-window' | 'any-resolvable-date' | 'disabled';
    }
  | { mode: 'all-published'; allowFutureDates?: boolean; directAccess?: 'within-archive-window' | 'any-resolvable-date' | 'disabled' }
  | { mode: 'disabled'; directAccess?: 'disabled' };

export type ContentManifest = {
  schemaVersion: 'daily-game-content-manifest.v1';
  gameId: string;
  displayName?: string;
  defaultMaxGuesses?: number;
  inputModes: string[];
  puzzleResolver: PuzzleResolverConfig;
  archive: ArchiveConfig;
  extension: Record<string, unknown>;
};

export type DateIndex = {
  schemaVersion: 'daily-game-date-index.v1';
  gameId: string;
  dates: Array<{ date: string; puzzlePath: string; assetsPrefix?: string }>;
};

export type ResolvedPuzzleRef = {
  date: string;
  path: string;
  assetPath?: string;
  index?: number;
};

export type RegisteredGame = {
  id: string;
  slug: string;
  displayName: string;
  category: string;
  routePrefix: string;
  contentManifestUrl: string;
  dateIndexUrl: string | null;
  puzzleBaseUrl: string;
  assetBaseUrl: string;
  runtimeAssetBaseUrl: string;
  dates: readonly string[];
  GameView: unknown;
  createRuntime: () => Promise<GameRuntime>;
};
