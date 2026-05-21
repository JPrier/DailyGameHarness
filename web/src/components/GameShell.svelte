<script lang="ts">
  import ErrorPanel from './ErrorPanel.svelte';
  import GenericFeedback from './GenericFeedback.svelte';
  import GuessInput from './GuessInput.svelte';
  import ResultModal from './ResultModal.svelte';
  import ShareButton from './ShareButton.svelte';
  import { loadPuzzle } from '../lib/game-loader';
  import { keyFor, loadState, saveState } from '../lib/local-state';
  import type {
    GameRuntime,
    GameState,
    GuessEvaluation,
    PlayerInput,
    RegisteredGame,
  } from '../lib/types';

  export let game: RegisteredGame;
  export let date: string;

  let loading = true;
  let error = '';
  let contentManifest: unknown = null;
  let dateIndex: unknown = null;
  let puzzle: any = null;
  let runtime: GameRuntime | null = null;
  let state: GameState | null = null;
  let latestEvaluation: GuessEvaluation | null = null;
  let GameView: any = game.GameView;

  $: isComplete = state?.status === 'won' || state?.status === 'lost';

  async function init() {
    loading = true;
    error = '';
    try {
      const [manifestResponse, indexResponse] = await Promise.all([
        fetch(game.contentManifestUrl),
        fetch(game.dateIndexUrl),
      ]);
      if (!manifestResponse.ok || !indexResponse.ok) throw new Error('content_not_found');
      contentManifest = await manifestResponse.json();
      dateIndex = await indexResponse.json();
      runtime = await game.createRuntime();
      const contentValidation = await runtime.validateContent({
        packageConfig: null,
        contentManifest,
        dateIndex,
      });
      if (!contentValidation.ok) throw new Error(contentValidation.errors[0]?.message ?? 'content invalid');
      puzzle = await loadPuzzle(game, date);
      const puzzleValidation = await runtime.validatePuzzle({ contentManifest, puzzle });
      if (!puzzleValidation.ok) throw new Error(puzzleValidation.errors[0]?.message ?? 'puzzle invalid');
      const initial = await runtime.createInitialState({ contentManifest, puzzle, date });
      const key = keyFor(runtime.contractVersion, initial.gameId, initial.puzzleId);
      state = loadState(key, initial.gameId, initial.puzzleId, date) ?? initial;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Unable to load puzzle.';
    } finally {
      loading = false;
    }
  }

  async function submitInput(input: PlayerInput) {
    if (!runtime || !state || !puzzle || isComplete) return;
    const result = await runtime.submitGuess({ contentManifest, puzzle, state, input });
    latestEvaluation = result.evaluation;
    state = result.state;
    saveState(keyFor(runtime.contractVersion, state.gameId, state.puzzleId), state);
  }

  async function buildShareText() {
    if (!runtime || !state || !puzzle) return '';
    const result = await runtime.buildShareText({ contentManifest, puzzle, state });
    if (typeof result === 'string') return result;
    return `${game.displayName} ${date} ${state.status}`;
  }

  init();
</script>

{#if loading}
  <p data-testid="loading">Loading puzzle...</p>
{:else if error}
  <ErrorPanel message={error} />
{:else if state && puzzle}
  <section data-testid="game-shell">
    <h1>{game.displayName}</h1>
    <svelte:component
      this={GameView}
      {game}
      {contentManifest}
      {puzzle}
      {state}
      {latestEvaluation}
      {submitInput}
    />
    <GuessInput disabled={isComplete} {submitInput} />
    <GenericFeedback feedback={latestEvaluation?.feedback ?? []} />
    <p data-testid="guess-count">{state.guessCount}</p>
    <ResultModal {state} />
    <ShareButton {buildShareText} />
  </section>
{/if}
