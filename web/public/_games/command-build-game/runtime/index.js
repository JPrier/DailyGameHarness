export async function createRuntime() {
  return {
    contractVersion: 'daily-game-runtime.v1',
    async validateContent({ packageConfig, contentManifest }) {
      const errors = [];
      if (packageConfig?.game?.id && packageConfig.game.id !== 'command-build-game') errors.push({ code: 'bad_package', message: 'Unexpected package id' });
      if (contentManifest?.gameId !== 'command-build-game') errors.push({ code: 'bad_manifest', message: 'Unexpected manifest game id' });
      return errors.length ? { ok: false, errors, warnings: [] } : { ok: true, warnings: [] };
    },
    async validatePuzzle({ puzzle }) {
      const errors = [];
      if (puzzle?.gameId !== 'command-build-game') errors.push({ code: 'bad_puzzle', message: 'Unexpected puzzle game id' });
      if (puzzle?.extension?.answer !== 'command') errors.push({ code: 'bad_answer', message: 'Unexpected answer' });
      return errors.length ? { ok: false, errors, warnings: [] } : { ok: true, warnings: [] };
    },
    async createInitialState({ contentManifest, puzzle, date }) {
      return { schemaVersion: 'daily-game-state.v1', gameId: puzzle.gameId, puzzleId: puzzle.puzzleId, date, status: 'in_progress', guessCount: 0, maxGuesses: contentManifest.defaultMaxGuesses ?? 3, currentStage: 0, publicState: {} };
    },
    async submitGuess({ state, input }) {
      if (state.status !== 'in_progress') return { state, evaluation: { outcome: 'invalid', consumedGuess: false, feedback: [] } };
      const correct = String(input?.value ?? '').trim().toLowerCase() === 'command';
      const guessCount = state.guessCount + 1;
      const status = correct ? 'won' : guessCount >= state.maxGuesses ? 'lost' : 'in_progress';
      const next = { ...state, status, guessCount, publicState: { feedback: correct ? 'correct' : 'incorrect' } };
      return { state: next, evaluation: { outcome: correct ? 'correct' : 'incorrect', consumedGuess: true, feedback: [{ key: 'result', label: 'Result', kind: 'text', value: next.publicState.feedback, severity: correct ? 'good' : 'bad' }] } };
    },
    async buildShareText({ state }) {
      return 'Command Build Game ' + state.date + ' ' + state.status;
    },
  };
}
