<script lang="ts">
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { chatStore } from "../stores/chat.svelte";
  import { personaStore } from "../stores/persona.svelte";
  import MessageBubble from "./MessageBubble.svelte";
  import PhaseIcon from "./PhaseIcon.svelte";

  onMount(() => {
    if (!personaStore.loaded) personaStore.load();
  });

  let love = $derived(
    personaStore.personas.find((p) => p.id === personaStore.activeId)?.love ?? 0,
  );

  let input = $state("");

  async function handleSubmit(event: Event) {
    event.preventDefault();
    if (!input.trim() || chatStore.streaming) return;
    const text = input;
    input = "";
    await chatStore.send(text);
  }
</script>

<div class="love-meter" title="Love meter">
  <svg viewBox="0 0 24 24" fill="currentColor" stroke="none">
    <path
      d="M12 21s-6.7-4.35-9.3-8.28C.5 9.5 1.8 5.6 5.3 5c2.1-.35 3.9.9 4.7 2.4.8-1.5 2.6-2.75 4.7-2.4 3.5.6 4.8 4.5 2.6 7.72C18.7 16.65 12 21 12 21z"
    />
  </svg>
  <span>{love}</span>
</div>

<div class="chat-pane">
  <div class="messages">
    {#each chatStore.messages as message}
      <MessageBubble role={message.role} content={message.content} sentiment={message.sentiment} />
    {/each}
    {#if chatStore.streaming}
      {#if chatStore.streamingText}
        <MessageBubble
          role="assistant"
          content={chatStore.streamingText}
          sentiment={chatStore.sentiment}
        />
      {:else if chatStore.phase}
        <span class="phase-icon" title={chatStore.phase}>
          <PhaseIcon phase={chatStore.phase} />
        </span>
      {/if}
    {/if}
  </div>

  {#if chatStore.error}
    <p class="error">{chatStore.error}</p>
  {/if}

  {#if !chatStore.streaming}
    <form
      class="composer"
      onsubmit={handleSubmit}
      transition:fly={{ y: 40, duration: 220 }}
    >
      <input bind:value={input} placeholder="Ask Desky anything…" />
      <button type="submit" disabled={!input.trim()}>Send</button>
    </form>
  {/if}
</div>

<style>
  .love-meter {
    position: fixed;
    top: 0.6rem;
    right: 0.6rem;
    z-index: 100;
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem 0.7rem;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--bg-elevated);
    color: var(--text);
    font-size: 0.9rem;
    font-weight: 600;
  }
  .love-meter svg {
    width: 1rem;
    height: 1rem;
    color: #e0759a;
  }
  .chat-pane {
    display: flex;
    flex-direction: column;
    height: 100vh;
    max-width: 640px;
    margin: 0 auto;
    padding: 1rem;
    padding-top: 3rem;
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
    border: 1px solid var(--border);
    background: var(--bg-elevated);
    color: var(--text);
  }
  .composer button {
    padding: 0.6rem 1.2rem;
    border-radius: 8px;
    border: none;
    background: var(--accent);
    color: var(--accent-text);
    cursor: pointer;
  }
  .composer button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .error {
    color: var(--danger);
    font-size: 0.9rem;
  }
  .phase-icon {
    align-self: flex-start;
    display: flex;
    padding: 0.6rem 0.9rem;
    color: var(--text-muted);
    animation:
      phase-pop 0.25s ease-out,
      phase-pulse 2s ease-in-out infinite;
  }
  @keyframes phase-pop {
    from {
      opacity: 0;
      transform: scale(0.5);
    }
    to {
      opacity: 0.85;
      transform: scale(1);
    }
  }
  @keyframes phase-pulse {
    0%,
    100% {
      opacity: 0.6;
    }
    50% {
      opacity: 1;
    }
  }
</style>
