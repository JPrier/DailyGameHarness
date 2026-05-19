import type { ReturnTypeGame } from './runtime';
import { getRegisteredGame } from './game-registry';

export async function loadGameBundle(slug: string): Promise<ReturnTypeGame> {
  const game = getRegisteredGame(slug);
  if (!game) throw new Error('game_not_found');
  const [manifest, dateIndex] = await Promise.all([
    fetch(game.contentManifestUrl).then((r) => r.json()),
    fetch(game.dateIndexUrl).then((r) => r.json()),
  ]);
  return { game, manifest, dateIndex };
}

export function puzzleUrlFor(game: any, date: string): string {
  return `${game.puzzleBaseUrl}/${date}.json`;
}
