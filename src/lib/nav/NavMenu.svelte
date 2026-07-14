<script lang="ts">
  let { view = $bindable() }: { view: "chat" | "persona" } = $props();

  let open = $state(false);

  function select(next: "chat" | "persona") {
    view = next;
    open = false;
  }

  function handleWindowClick(event: MouseEvent) {
    if (!(event.target as HTMLElement)?.closest(".nav-menu")) open = false;
  }
</script>

<svelte:window onclick={handleWindowClick} />

<div class="nav-menu">
  <button class="nav-trigger" onclick={() => (open = !open)} aria-label="Menu">☰</button>
  {#if open}
    <div class="nav-dropdown">
      <button class:selected={view === "chat"} onclick={() => select("chat")}>Chat</button>
      <button class:selected={view === "persona"} onclick={() => select("persona")}>
        Persona
      </button>
    </div>
  {/if}
</div>

<style>
  .nav-menu {
    position: fixed;
    top: 0.6rem;
    left: 0.6rem;
    z-index: 100;
  }
  .nav-trigger {
    width: 2.2rem;
    height: 2.2rem;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--bg-elevated);
    color: var(--text);
    cursor: pointer;
    font-size: 1rem;
  }
  .nav-dropdown {
    position: absolute;
    top: 2.6rem;
    left: 0;
    display: flex;
    flex-direction: column;
    min-width: 8rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    background: var(--bg-elevated);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  }
  .nav-dropdown button {
    padding: 0.6rem 0.9rem;
    border: none;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
    font-size: 0.9rem;
  }
  .nav-dropdown button:hover {
    background: rgba(255, 255, 255, 0.08);
  }
  .nav-dropdown button.selected {
    background: var(--accent);
    color: var(--accent-text);
  }
</style>
