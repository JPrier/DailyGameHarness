import type { GameRuntime, RegisteredGame } from './types';

export type LoadedGameBundle = { game: RegisteredGame; manifest: unknown; dateIndex: unknown };

export async function createRuntimeFor(game: RegisteredGame): Promise<GameRuntime> {
  return game.createRuntime();
}
