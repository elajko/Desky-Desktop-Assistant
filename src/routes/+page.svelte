<script lang="ts">
  import ChatPane from "$lib/chat/ChatPane.svelte";
  import PersonaView from "$lib/persona/PersonaView.svelte";
  import NavMenu from "$lib/nav/NavMenu.svelte";

  let view = $state<"chat" | "persona">("chat");
</script>

<NavMenu bind:view />

{#if view === "chat"}
  <ChatPane />
{:else}
  <PersonaView />
{/if}

<style>
  /* Desky is dark-only by design — no light theme, regardless of OS setting. */
  :global(:root) {
    color-scheme: dark only;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;

    --bg: #242424;
    --bg-elevated: #2f2f2f;
    --text: #f0f0f0;
    --text-muted: #9a9a9a;
    --accent: #4d7dea;
    --accent-text: #ffffff;
    --border: #4a4a4a;
    --danger: #e0554a;
    --success: #4caf50;

    color: var(--text);
    background-color: var(--bg);
  }

  :global(body) {
    margin: 0;
  }
</style>
