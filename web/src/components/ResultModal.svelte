<script lang="ts">
  import type { GameState } from '../lib/types';

  export let state: GameState | null = null;
</script>

{#if state && state.status !== 'in_progress'}
  <section class={`result-modal result-modal--${state.status}`} data-testid="result-modal" aria-live="polite">
    <h2>{state.status === 'won' ? 'Solved' : 'Game over'}</h2>
  </section>
{/if}
