import type { GameState } from './types';

export const keyFor = (contractVersion: string, gameId: string, puzzleId: string) =>
  `daily-game:${contractVersion}:${gameId}:${puzzleId}`;

export function saveState(key: string, state: GameState): void {
  localStorage.setItem(key, JSON.stringify(state));
}

export function loadState(
  key: string,
  gameId: string,
  puzzleId: string,
  date: string,
): GameState | null {
  const raw = localStorage.getItem(key);
  if (!raw) return null;
  try {
    const state = JSON.parse(raw) as Partial<GameState> & Record<string, unknown>;
    if (!isValidStateShape(state)) return null;
    if (state.gameId !== gameId || state.puzzleId !== puzzleId || state.date !== date) return null;
    return state;
  } catch {
    return null;
  }
}

export function isValidStateShape(state: unknown): state is GameState {
  if (!state || typeof state !== 'object') return false;
  const candidate = state as Partial<GameState> & Record<string, unknown>;
  return (
    candidate.schemaVersion === 'daily-game-state.v1' &&
    typeof candidate.gameId === 'string' &&
    typeof candidate.puzzleId === 'string' &&
    typeof candidate.date === 'string' &&
    (candidate.status === 'in_progress' || candidate.status === 'won' || candidate.status === 'lost') &&
    Number.isInteger(candidate.guessCount) &&
    Number.isInteger(candidate.maxGuesses) &&
    Number.isInteger(candidate.currentStage) &&
    !!candidate.publicState &&
    typeof candidate.publicState === 'object' &&
    !('privateState' in candidate)
  );
}
