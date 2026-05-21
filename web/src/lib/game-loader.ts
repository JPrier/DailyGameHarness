import type { LoadedGameBundle } from './runtime';
import type { RegisteredGame } from './types';
import { getRegisteredGame } from './game-registry';

export async function loadGameBundle(slug: string): Promise<LoadedGameBundle> {
  const game = getRegisteredGame(slug);
  if (!game) throw new Error('game_not_found');
  const [manifest, dateIndex] = await Promise.all([
    fetchJson(game.contentManifestUrl),
    fetchJson(game.dateIndexUrl),
  ]);
  return { game, manifest, dateIndex };
}

export function puzzleUrlFor(game: Pick<RegisteredGame, 'puzzleBaseUrl'>, date: string): string {
  return `${game.puzzleBaseUrl}/${date}.json`;
}

export async function loadPuzzle(game: Pick<RegisteredGame, 'puzzleBaseUrl'>, date: string): Promise<unknown> {
  return fetchJson(puzzleUrlFor(game, date));
}

async function fetchJson(url: string): Promise<unknown> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`fetch_failed:${url}`);
  }
  return response.json();
}
