<script lang="ts">
  export let buildShareText: () => Promise<string>;
  let text = '';
  $: link = text.match(/https?:\/\/\S+/)?.[0] ?? '';

  async function share() {
    text = await buildShareText();
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
    }
  }
</script>

<div class="share-block">
  <button data-testid="share-button" type="button" on:click={share}>Share</button>
  {#if text}
    <output data-testid="share-output">{text}</output>
    {#if link}
      <a data-testid="share-link" href={link}>{link}</a>
    {/if}
  {/if}
</div>
