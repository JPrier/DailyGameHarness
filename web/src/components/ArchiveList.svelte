<script lang="ts">
  export let gameSlug: string;
  export let routePrefix = '';
  export let dates: string[] = [];
</script>

{#if dates.length > 0}
  <nav class="archive-list" data-testid="archive-list" aria-label="Puzzle archive">
    {#each dates as archiveDate}
      <a data-testid="archive-date" href={`${routePrefix}/games/${gameSlug}/?date=${archiveDate}`}>{archiveDate}</a>
    {/each}
  </nav>
{/if}
