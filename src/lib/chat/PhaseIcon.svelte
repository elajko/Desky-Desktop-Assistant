<script lang="ts">
  import type { ChatPhase } from "../stores/chat.svelte";

  let { phase }: { phase: ChatPhase } = $props();
</script>

{#if phase === "waking_up"}
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M6 17a6 6 0 0 1 12 0" />
    <line x1="3" y1="17" x2="21" y2="17" />
    <line x1="12" y1="7" x2="12" y2="10" />
    <line x1="6" y1="10" x2="7.8" y2="11.8" />
    <line x1="18" y1="10" x2="16.2" y2="11.8" />
  </svg>
{:else if phase === "calling_tool"}
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <g transform="rotate(45 12 12)">
      <rect x="9" y="3" width="6" height="6" rx="1" />
      <rect x="10.5" y="9" width="3" height="11" rx="1" />
    </g>
  </svg>
{:else}
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <ellipse cx="12" cy="10" rx="7" ry="5.5" />
    <circle cx="6" cy="18" r="1.3" fill="currentColor" stroke="none" />
    <circle cx="4" cy="21" r="0.8" fill="currentColor" stroke="none" />
  </svg>
{/if}

<style>
  svg {
    width: 1.3rem;
    height: 1.3rem;
    display: block;
  }
</style>
