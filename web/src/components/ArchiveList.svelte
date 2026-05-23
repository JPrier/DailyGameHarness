<script lang="ts">
  import { withAppBase, withRoutePrefix } from '../lib/base-path';

  export let gameSlug: string;
  export let routePrefix = '';
  export let dates: string[] = [];

  $: gamePath = withAppBase(withRoutePrefix(routePrefix, `/games/${gameSlug}/`));
</script>

{#if dates.length > 0}
  <nav class="archive-list" data-testid="archive-list" aria-label="Puzzle archive">
    {#each dates as archiveDate}
      <a data-testid="archive-date" href={`${gamePath}?date=${archiveDate}`}>{archiveDate}</a>
    {/each}
  </nav>
{/if}
