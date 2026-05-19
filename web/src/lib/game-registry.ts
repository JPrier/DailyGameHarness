import { generatedGameRegistry } from '../generated/game-registry';

export function listRegisteredGames() {
  return Object.values(generatedGameRegistry);
}

export function getRegisteredGame(slug: string) {
  return Object.values(generatedGameRegistry).find((g) => g.slug === slug) ?? null;
}
