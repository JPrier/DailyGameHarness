<script>
  export let puzzle;
  export let state;
  export let submitInput;
  let guess = '';

  async function submit() {
    await submitInput({ kind: 'text', value: guess });
    guess = '';
  }
</script>

<section data-testid="command-build-game-root">
  <p data-testid="prompt">{puzzle.display.initialPrompt}</p>
  <form on:submit|preventDefault={submit}>
    <label for="command-build-guess">Guess</label>
    <input id="command-build-guess" data-testid="guess-input" bind:value={guess} disabled={state.status !== 'in_progress'} />
    <button data-testid="guess-submit" disabled={state.status !== 'in_progress'}>Submit</button>
  </form>
  <p data-testid="guess-count">{state.guessCount}</p>
  {#if state.publicState.feedback}
    <p data-testid="feedback">{state.publicState.feedback}</p>
  {/if}
</section>
