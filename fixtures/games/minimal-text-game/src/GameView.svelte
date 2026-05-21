<script lang="ts">
  export let puzzle: any;
  export let state: any;
  export let latestEvaluation: any;
  export let submitInput: (input: any) => Promise<void>;
  let guess = "";
  $: complete = state?.status === 'won' || state?.status === 'lost';
</script>

<p data-testid="prompt">{puzzle.display.initialPrompt}</p>
<input data-testid="guess" bind:value={guess} disabled={complete} />
<button data-testid="submit" disabled={complete} on:click={() => submitInput({ kind: 'text', value: guess })}>Submit</button>
{#if latestEvaluation}
<p data-testid="feedback">{latestEvaluation.outcome}</p>
{/if}
