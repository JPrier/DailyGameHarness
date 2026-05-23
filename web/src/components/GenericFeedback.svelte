<script lang="ts">
  import type { FeedbackItem } from '../lib/types';

  export let feedback: FeedbackItem[] = [];
</script>

{#if feedback.length > 0}
  <ul data-testid="feedback-list" class="feedback">
    {#each feedback as item}
      <li data-testid={`feedback-${item.kind}`} class={`feedback-item ${item.severity}`}>
        <strong>{item.label}</strong>: {String(item.value)}
      </li>
    {/each}
  </ul>
{/if}
