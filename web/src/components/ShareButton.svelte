<script lang="ts">
  export let buildShareText: () => Promise<string>;
  let text = '';

  async function share() {
    text = await buildShareText();
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
    }
  }
</script>

<button data-testid="share-button" type="button" on:click={share}>Share</button>
{#if text}
  <output data-testid="share-output">{text}</output>
{/if}
