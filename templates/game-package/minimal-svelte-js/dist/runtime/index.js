export async function createRuntime() {
  return {
    contractVersion: 'daily-game-runtime.v1',
    async validateContent({ contentManifest }) {
      if (contentManifest?.schemaVersion !== 'daily-game-content-manifest.v1') {
        return { ok: false, errors: [{ code: 'bad_manifest_schema', message: 'Unsupported content manifest schema' }], warnings: [] };
      }
      return { ok: true, warnings: [] };
    },
    async validatePuzzle({ puzzle }) {
      if (puzzle?.schemaVersion !== 'daily-game-puzzle.v1') {
        return { ok: false, errors: [{ code: 'bad_puzzle_schema', message: 'Unsupported puzzle schema' }], warnings: [] };
      }
      if (!puzzle?.extension?.answer) {
        return { ok: false, errors: [{ code: 'missing_answer', message: 'Missing extension.answer' }], warnings: [] };
      }
      return { ok: true, warnings: [] };
    },
    async createInitialState({ contentManifest, puzzle, date }) {
      return {
        schemaVersion: 'daily-game-state.v1',
        gameId: puzzle.gameId,
        puzzleId: puzzle.puzzleId,
        date,
        status: 'in_progress',
        guessCount: 0,
        maxGuesses: contentManifest.defaultMaxGuesses ?? 6,
        currentStage: 0,
        publicState: { history: [] }
      };
    },
    async submitGuess({ puzzle, state, input }) {
      if (state.status !== 'in_progress') {
        return { state, evaluation: { outcome: 'invalid', consumedGuess: false, feedback: [] } };
      }
      const value = String(input?.value ?? '').trim().toLowerCase();
      if (!value) return { state, evaluation: { outcome: 'invalid', consumedGuess: false, feedback: [] } };
      const guessCount = state.guessCount + 1;
      const correct = value === String(puzzle.extension.answer).toLowerCase();
      const nextState = {
        ...state,
        guessCount,
        currentStage: guessCount,
        status: correct ? 'won' : guessCount >= state.maxGuesses ? 'lost' : 'in_progress',
        publicState: { ...state.publicState, history: [...(state.publicState.history ?? []), value] }
      };
      return {
        state: nextState,
        evaluation: {
          outcome: correct ? 'correct' : 'incorrect',
          consumedGuess: true,
          feedback: [{ key: 'result', label: 'Result', kind: 'text', value: correct ? 'correct' : 'wrong', severity: correct ? 'good' : 'bad' }]
        }
      };
    },
    async buildShareText({ puzzle, state }) {
      return `🎮 ${puzzle.date} ${state.status} ${state.guessCount}/${state.maxGuesses}`;
    }
  };
}
