<script lang="ts">
  import type { PlayerInput } from '../lib/types';

  export let disabled = false;
  export let submitInput: (input: PlayerInput) => Promise<void>;
  let value = '';

  async function submit() {
    await submitInput({ kind: 'text', value });
    value = '';
  }
</script>

<form
  data-testid="guess-form"
  on:submit|preventDefault={submit}
>
  <input data-testid="guess-input" bind:value disabled={disabled} aria-label="Guess" />
  <button data-testid="guess-submit" type="submit" disabled={disabled}>Submit</button>
</form>
