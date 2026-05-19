export type PlayerInput = { kind: 'text'; value: string };
export type GameState = { schemaVersion: 'daily-game-state.v1'; gameId: string; puzzleId: string; date: string; status: 'in_progress' | 'won' | 'lost'; guessCount: number; maxGuesses: number; currentStage: number; publicState: Record<string, unknown> };
