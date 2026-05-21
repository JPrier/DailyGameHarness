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

export type RegisteredGame = {
  id: string;
  slug: string;
  displayName: string;
  category: string;
  contentManifestUrl: string;
  dateIndexUrl: string;
  puzzleBaseUrl: string;
  assetBaseUrl: string;
  runtimeAssetBaseUrl: string;
  dates: readonly string[];
  GameView: unknown;
  createRuntime: () => Promise<GameRuntime>;
};
