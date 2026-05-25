<script lang="ts">
  import ErrorPanel from './ErrorPanel.svelte';
  import GenericFeedback from './GenericFeedback.svelte';
  import GuessInput from './GuessInput.svelte';
  import ResultModal from './ResultModal.svelte';
  import ShareButton from './ShareButton.svelte';
  import ArchiveList from './ArchiveList.svelte';
  import { archiveDates, isDateAllowed, todayInTimezone } from '../lib/archive';
  import { withAppBase, withRoutePrefix } from '../lib/base-path';
  import { loadPuzzle } from '../lib/game-loader';
  import { getRegisteredGame } from '../lib/game-registry';
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

  export let game: RegisteredGame | null = null;
  export let gameSlug = '';
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
  let activeGame: RegisteredGame | null = null;
  let GameView: any = null;

  $: isComplete = state?.status === 'won' || state?.status === 'lost';
  $: viewOwnsControls = activeGame ? ['city-grid', 'flag-fade'].includes(activeGame.slug) : false;

  async function init() {
    loading = true;
    error = '';
    try {
      activeGame = game ?? getRegisteredGame(gameSlug);
      if (!activeGame) throw new Error('game_not_found');
      GameView = activeGame.GameView;
      const [manifestResponse, indexResponse] = await Promise.all([
        fetch(withAppBase(activeGame.contentManifestUrl)),
        activeGame.dateIndexUrl ? fetch(withAppBase(activeGame.dateIndexUrl)) : Promise.resolve(null),
      ]);
      if (!manifestResponse.ok || (indexResponse && !indexResponse.ok)) throw new Error('content_not_found');
      contentManifest = await manifestResponse.json();
      dateIndex = indexResponse ? await indexResponse.json() : null;
      runtime = await activeGame.createRuntime();
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
      puzzle = await loadPuzzle(activeGame, resolvedPuzzle);
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
    if (!activeGame) return '';
    const gamePath = withAppBase(withRoutePrefix(activeGame.routePrefix, `/games/${activeGame.slug}/`));
    const gameUrl = new URL(gamePath, window.location.origin).href;
    const shareText =
      typeof result === 'string' ? result : `${game.displayName} ${selectedDate} ${state.status}`;
    return shareText.includes(gameUrl) ? shareText : `${shareText}\n${gameUrl}`;
  }

  init();
</script>

{#if loading}
  <p data-testid="loading">Loading puzzle...</p>
{:else if error}
  {#if error === 'puzzle_unavailable'}
    <section class="error-panel" data-testid="not-found" role="alert">
      <h2>Puzzle unavailable</h2>
      <p>Puzzle unavailable</p>
    </section>
  {:else}
    <ErrorPanel message={error} />
  {/if}
{:else if state && puzzle && contentManifest && resolvedPuzzle && activeGame}
  <section class="game-panel" data-testid="game-shell">
    <p class="eyebrow">Daily puzzle</p>
    <h1>{activeGame.displayName}</h1>
    <ArchiveList
      gameSlug={activeGame.slug}
      routePrefix={activeGame.routePrefix}
      dates={archiveDates(contentManifest.archive, todayInTimezone(contentManifest.puzzleResolver.timezone))}
    />
    <ResultModal {state} />
    <svelte:component
      this={GameView}
      game={activeGame}
      {contentManifest}
      {puzzle}
      {resolvedPuzzle}
      {state}
      {latestEvaluation}
      {submitInput}
    />
    {#if !viewOwnsControls}
      <div class="guess-row">
        <GuessInput disabled={isComplete} {submitInput} />
      </div>
      <GenericFeedback feedback={latestEvaluation?.feedback ?? []} />
      <p data-testid="guess-count">{state.guessCount}</p>
    {/if}
    <ShareButton {buildShareText} />
  </section>
{/if}
