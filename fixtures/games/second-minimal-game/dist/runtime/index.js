export async function createRuntime() {
  return {
    contractVersion: 'daily-game-runtime.v1',
    async validateContent() { return { ok: true, warnings: [] }; },
    async validatePuzzle({ puzzle }) {
      if (!puzzle?.extension?.answer) return { ok: false, errors: [{ code: 'missing_answer', message: 'Missing extension.answer' }], warnings: [] };
      return { ok: true, warnings: [] };
    },
    async createInitialState({ contentManifest, puzzle, date }) {
      return { schemaVersion: 'daily-game-state.v1', gameId: puzzle.gameId, puzzleId: puzzle.puzzleId, date, status: 'in_progress', guessCount: 0, maxGuesses: contentManifest.defaultMaxGuesses ?? 6, currentStage: 0, publicState: { history: [] } };
    },
    async submitGuess({ puzzle, state, input }) {
      if (state.status !== 'in_progress') return { state, evaluation: { outcome: 'invalid', consumedGuess: false, message: 'already complete', feedback: [] } };
      const value = (input?.value ?? '').trim().toLowerCase();
      if (!value) return { state, evaluation: { outcome: 'invalid', consumedGuess: false, message: 'empty', feedback: [] } };
      const answer = String(puzzle.extension.answer).toLowerCase();
      const guessCount = state.guessCount + 1;
      const correct = value === answer;
      const status = correct ? 'won' : (guessCount >= state.maxGuesses ? 'lost' : 'in_progress');
      const newState = { ...state, guessCount, status, publicState: { ...state.publicState, history: [...(state.publicState.history || []), value] } };
      return { state: newState, evaluation: { outcome: correct ? 'correct' : 'incorrect', consumedGuess: true, feedback: [{ key: 'result', label: 'Result', kind: 'text', value: correct ? 'correct' : 'wrong', severity: correct ? 'good' : 'bad' }] } };
    },
    async buildShareText({ state, puzzle }) { return `🎮 ${puzzle.date} ${state.status} ${state.guessCount}/${state.maxGuesses}`; },
  };
}
