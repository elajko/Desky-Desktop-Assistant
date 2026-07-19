<script lang="ts">
  let { sentiment }: { sentiment: "liked" | "disliked" } = $props();
</script>

{#if sentiment === "liked"}
  <svg viewBox="0 0 24 24" fill="currentColor" stroke="none">
    <path
      d="M12 21s-6.7-4.35-9.3-8.28C.5 9.5 1.8 5.6 5.3 5c2.1-.35 3.9.9 4.7 2.4.8-1.5 2.6-2.75 4.7-2.4 3.5.6 4.8 4.5 2.6 7.72C18.7 16.65 12 21 12 21z"
    />
  </svg>
{:else}
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M7 16a4 4 0 0 1 .7-7.94 5 5 0 0 1 9.79 1.2A3.5 3.5 0 0 1 17 16H7z" />
    <line x1="8" y1="19" x2="8" y2="21" />
    <line x1="12" y1="19" x2="12" y2="21" />
    <line x1="16" y1="19" x2="16" y2="21" />
  </svg>
{/if}

<style>
  svg {
    width: 1.1rem;
    height: 1.1rem;
    display: block;
    flex-shrink: 0;
  }
</style>
