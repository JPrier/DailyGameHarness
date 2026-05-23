export async function createRuntime() {
  return {
    contractVersion: 'daily-game-runtime.v1',
    async validateContent({ packageConfig, contentManifest, dateIndex }) {
      const errors = [];
      if (packageConfig?.game?.id && contentManifest?.gameId !== packageConfig.game.id) errors.push({ code: 'manifest_game_mismatch', message: 'Manifest gameId must match package config' });
      if (contentManifest?.schemaVersion !== 'daily-game-content-manifest.v1') errors.push({ code: 'bad_manifest_schema', message: 'Unsupported content manifest schema' });
      if (dateIndex && dateIndex.schemaVersion !== 'daily-game-date-index.v1') errors.push({ code: 'bad_date_index_schema', message: 'Unsupported date index schema' });
      if (dateIndex && dateIndex.gameId !== contentManifest?.gameId) errors.push({ code: 'date_index_game_mismatch', message: 'Date index gameId must match manifest' });
      return errors.length ? { ok: false, errors, warnings: [] } : { ok: true, warnings: [] };
    },
    async validatePuzzle({ puzzle }) {
      if (puzzle?.schemaVersion !== 'daily-game-puzzle.v1') return { ok: false, errors: [{ code: 'bad_puzzle_schema', message: 'Unsupported puzzle schema' }], warnings: [] };
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
      const newState = { ...state, guessCount, currentStage: guessCount, status, publicState: { ...state.publicState, history: [...(state.publicState.history || []), value] } };
      return { state: newState, evaluation: { outcome: correct ? 'correct' : 'incorrect', consumedGuess: true, feedback: [{ key: 'result', label: 'Result', kind: 'text', value: correct ? 'correct' : 'wrong', severity: correct ? 'good' : 'bad' }] } };
    },
    async buildShareText({ state, puzzle }) { return `🎮 ${puzzle.date} ${state.status} ${state.guessCount}/${state.maxGuesses}`; },
  };
}
