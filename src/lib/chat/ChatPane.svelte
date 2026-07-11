<script lang="ts">
  import { chatStore } from "../stores/chat.svelte";
  import MessageBubble from "./MessageBubble.svelte";

  let input = $state("");

  async function handleSubmit(event: Event) {
    event.preventDefault();
    if (!input.trim() || chatStore.streaming) return;
    const text = input;
    input = "";
    await chatStore.send(text);
  }
</script>

<div class="chat-pane">
  <div class="messages">
    {#each chatStore.messages as message}
      <MessageBubble role={message.role} content={message.content} />
    {/each}
    {#if chatStore.streaming}
      <MessageBubble role="assistant" content={chatStore.streamingText || "…"} />
    {/if}
  </div>

  {#if chatStore.error}
    <p class="error">{chatStore.error}</p>
  {/if}

  <form class="composer" onsubmit={handleSubmit}>
    <input
      bind:value={input}
      placeholder="Ask Desky anything…"
      disabled={chatStore.streaming}
    />
    <button type="submit" disabled={chatStore.streaming || !input.trim()}>Send</button>
  </form>
</div>

<style>
  .chat-pane {
    display: flex;
    flex-direction: column;
    height: 100vh;
    max-width: 640px;
    margin: 0 auto;
    padding: 1rem;
    box-sizing: border-box;
  }
  .messages {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .composer {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }
  .composer input {
    flex: 1;
    padding: 0.6rem 0.8rem;
    border-radius: 8px;
    border: 1px solid #ccc;
  }
  .composer button {
    padding: 0.6rem 1.2rem;
    border-radius: 8px;
    border: none;
    background: #396cd8;
    color: white;
    cursor: pointer;
  }
  .composer button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .error {
    color: #c0392b;
    font-size: 0.9rem;
  }
</style>
