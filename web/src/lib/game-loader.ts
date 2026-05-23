import type { LoadedGameBundle } from './runtime';
import type { ContentManifest, DateIndex, RegisteredGame, ResolvedPuzzleRef } from './types';
import { getRegisteredGame } from './game-registry';
import { stripContentPrefix } from './base-path';

export async function loadGameBundle(slug: string): Promise<LoadedGameBundle> {
  const game = getRegisteredGame(slug);
  if (!game) throw new Error('game_not_found');
  const [manifest, dateIndex] = await Promise.all([
    fetchJson(game.contentManifestUrl),
    game.dateIndexUrl ? fetchJson(game.dateIndexUrl) : Promise.resolve(null),
  ]);
  return { game, manifest: manifest as ContentManifest, dateIndex: dateIndex as DateIndex | null };
}

export function puzzleUrlFor(game: Pick<RegisteredGame, 'puzzleBaseUrl'>, resolved: ResolvedPuzzleRef): string {
  return `${game.puzzleBaseUrl}/${stripContentPrefix(resolved.path, 'content/puzzles')}`;
}

export async function loadPuzzle(game: Pick<RegisteredGame, 'puzzleBaseUrl'>, resolved: ResolvedPuzzleRef): Promise<unknown> {
  return fetchJson(puzzleUrlFor(game, resolved));
}

export async function loadContentManifest(game: Pick<RegisteredGame, 'contentManifestUrl'>): Promise<ContentManifest> {
  return fetchJson(game.contentManifestUrl) as Promise<ContentManifest>;
}

export async function loadDateIndex(game: Pick<RegisteredGame, 'dateIndexUrl'>): Promise<DateIndex | null> {
  return game.dateIndexUrl ? (fetchJson(game.dateIndexUrl) as Promise<DateIndex>) : null;
}

async function fetchJson(url: string): Promise<unknown> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`fetch_failed:${url}`);
  }
  return response.json();
}
