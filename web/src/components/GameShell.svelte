<script lang="ts">
  import ErrorPanel from './ErrorPanel.svelte';
  import GenericFeedback from './GenericFeedback.svelte';
  import GuessInput from './GuessInput.svelte';
  import ResultModal from './ResultModal.svelte';
  import ShareButton from './ShareButton.svelte';
  import ArchiveList from './ArchiveList.svelte';
  import { archiveDates, isDateAllowed, todayInTimezone } from '../lib/archive';
  import { loadPuzzle } from '../lib/game-loader';
  import { keyFor, loadState, saveState } from '../lib/local-state';
  import { resolvePuzzle } from '../lib/puzzle-resolver';
  import type {
    ContentManifest,
    DateIndex,
    GameRuntime,
    GameState,
    GuessEvaluation,
    PlayerInput,
    RegisteredGame,
    ResolvedPuzzleRef,
  } from '../lib/types';

  export let game: RegisteredGame;
  export let date: string | undefined = undefined;

  let loading = true;
  let error = '';
  let selectedDate = '';
  let contentManifest: ContentManifest | null = null;
  let dateIndex: DateIndex | null = null;
  let resolvedPuzzle: ResolvedPuzzleRef | null = null;
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
        game.dateIndexUrl ? fetch(game.dateIndexUrl) : Promise.resolve(null),
      ]);
      if (!manifestResponse.ok || (indexResponse && !indexResponse.ok)) throw new Error('content_not_found');
      contentManifest = await manifestResponse.json();
      dateIndex = indexResponse ? await indexResponse.json() : null;
      runtime = await game.createRuntime();
      const contentValidation = await runtime.validateContent({
        packageConfig: null,
        contentManifest,
        dateIndex,
      });
      if (!contentValidation.ok) throw new Error(contentValidation.errors[0]?.message ?? 'content invalid');
      selectedDate =
        date ??
        new URL(window.location.href).searchParams.get('date') ??
        todayInTimezone(contentManifest.puzzleResolver.timezone);
      if (!/^\d{4}-\d{2}-\d{2}$/.test(selectedDate) || !isDateAllowed(contentManifest.archive, todayInTimezone(contentManifest.puzzleResolver.timezone), selectedDate)) {
        throw new Error('puzzle_unavailable');
      }
      resolvedPuzzle = resolvePuzzle(contentManifest, selectedDate, dateIndex);
      if (!resolvedPuzzle) throw new Error('puzzle_unavailable');
      puzzle = await loadPuzzle(game, resolvedPuzzle);
      const puzzleValidation = await runtime.validatePuzzle({ contentManifest, puzzle });
      if (!puzzleValidation.ok) throw new Error(puzzleValidation.errors[0]?.message ?? 'puzzle invalid');
      const initial = await runtime.createInitialState({ contentManifest, puzzle, date: selectedDate });
      const key = keyFor(runtime.contractVersion, initial.gameId, initial.puzzleId, selectedDate);
      state = loadState(key, initial.gameId, initial.puzzleId, selectedDate) ?? initial;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Unable to load puzzle.';
    } finally {
      loading = false;
    }
  }

  async function submitInput(input: PlayerInput) {
    if (!runtime || !state || !puzzle || !contentManifest || isComplete) return;
    const result = await runtime.submitGuess({ contentManifest, puzzle, state, input });
    latestEvaluation = result.evaluation;
    state = result.state;
    saveState(keyFor(runtime.contractVersion, state.gameId, state.puzzleId, state.date), state);
  }

  async function buildShareText() {
    if (!runtime || !state || !puzzle || !contentManifest) return '';
    const result = await runtime.buildShareText({ contentManifest, puzzle, state });
    const gameUrl = new URL(`${game.routePrefix}/games/${game.slug}/`, window.location.origin).href;
    const shareText =
      typeof result === 'string' ? result : `${game.displayName} ${selectedDate} ${state.status}`;
    return shareText.includes(gameUrl) ? shareText : `${shareText}\n${gameUrl}`;
  }

  init();
</script>

{#if loading}
  <p data-testid="loading">Loading puzzle...</p>
{:else if error}
  <ErrorPanel message={error} />
{:else if state && puzzle && contentManifest && resolvedPuzzle}
  <section data-testid="game-shell">
    <h1>{game.displayName}</h1>
    <ArchiveList
      gameSlug={game.slug}
      routePrefix={game.routePrefix}
      dates={archiveDates(contentManifest.archive, todayInTimezone(contentManifest.puzzleResolver.timezone))}
    />
    <svelte:component
      this={GameView}
      {game}
      {contentManifest}
      {puzzle}
      {resolvedPuzzle}
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
