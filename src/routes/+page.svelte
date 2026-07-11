<script lang="ts">
  import ChatPane from "$lib/chat/ChatPane.svelte";
</script>

<ChatPane />

<style>
  :global(:root) {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    color: #0f0f0f;
    background-color: #f6f6f6;
  }

  :global(body) {
    margin: 0;
  }

  @media (prefers-color-scheme: dark) {
    :global(:root) {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }
  }
</style>
