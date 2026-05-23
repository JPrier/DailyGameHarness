import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
write('dist/runtime/index.js', runtimeSource());
write('content/manifest.json', JSON.stringify(manifest(), null, 2) + '\n');
write('content/puzzles/v1/puzzle-0000.json', JSON.stringify(puzzle(), null, 2) + '\n');
write('content/assets/v1/puzzle-0000/initial.txt', 'generated command-mode asset\n');
console.log('command-build-game fixture package generated');

function write(rel, value) {
  const file = path.join(root, rel);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, value);
}

function manifest() {
  return {
    schemaVersion: 'daily-game-content-manifest.v1',
    gameId: 'command-build-game',
    displayName: 'Command Build Game',
    inputModes: ['text'],
    defaultMaxGuesses: 3,
    puzzleResolver: {
      mode: 'static-pool',
      timezone: 'America/New_York',
      startDate: '2026-01-01',
      poolVersions: [
        {
          version: 'v1',
          startDate: '2026-01-01',
          poolSize: 1,
          pathPattern: 'content/puzzles/{version}/puzzle-{index:04}.json',
          selector: { type: 'affine-permutation', a: 1, b: 0 },
          cyclePolicy: 'repeat',
        },
      ],
    },
    archive: {
      mode: 'rolling-window',
      days: 30,
      includeToday: true,
      allowFutureDates: false,
      directAccess: 'within-archive-window',
    },
    extension: {},
  };
}

function puzzle() {
  return {
    schemaVersion: 'daily-game-puzzle.v1',
    gameId: 'command-build-game',
    puzzleId: 'command-build-game-v1-0000',
    date: '2026-01-01',
    seed: 'command-build-game:v1:0000',
    display: {
      title: 'Command Build Puzzle',
      initialPrompt: 'Generated guess',
    },
    assets: {
      initial: ['content/assets/v1/puzzle-0000/initial.txt'],
    },
    extension: {
      answer: 'command',
    },
  };
}

function runtimeSource() {
  return `export async function createRuntime() {
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
`;
}
