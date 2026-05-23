import type { ContentManifest, DateIndex, GameRuntime, RegisteredGame } from './types';

export type LoadedGameBundle = { game: RegisteredGame; manifest: ContentManifest; dateIndex: DateIndex | null };

export async function createRuntimeFor(game: RegisteredGame): Promise<GameRuntime> {
  return game.createRuntime();
}
