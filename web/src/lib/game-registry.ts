import { generatedGameRegistry } from '../generated/game-registry';
import type { RegisteredGame } from './types';

export function listRegisteredGames(): RegisteredGame[] {
  return Object.values(generatedGameRegistry) as RegisteredGame[];
}

export function getRegisteredGame(slug: string): RegisteredGame | null {
  return listRegisteredGames().find((g) => g.slug === slug) ?? null;
}
