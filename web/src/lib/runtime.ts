export type ReturnTypeGame = { game: any; manifest: any; dateIndex: any };
export async function createRuntimeFor(game: any) {
  return game.createRuntime();
}
