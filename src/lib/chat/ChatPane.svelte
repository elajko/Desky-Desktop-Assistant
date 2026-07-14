<script lang="ts">
  import { fly } from "svelte/transition";
  import { chatStore } from "../stores/chat.svelte";
  import MessageBubble from "./MessageBubble.svelte";
  import PhaseIcon from "./PhaseIcon.svelte";
  import SystemInfoPanel from "./SystemInfoPanel.svelte";
  import type { SystemInfoData } from "../ipc";

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
      {#if message.kind === "text"}
        <MessageBubble role={message.role} content={message.content} />
      {:else if message.tool === "get_system_info"}
        <SystemInfoPanel data={message.data as SystemInfoData} />
      {/if}
    {/each}
    {#if chatStore.streaming}
      {#if chatStore.streamingText}
        <MessageBubble role="assistant" content={chatStore.streamingText} />
      {/if}
      {#if chatStore.phaseSteps.length > 0}
        <p class="phase-trail">
          {#each chatStore.phaseSteps as step, i}
            {@const isActive = i === chatStore.phaseSteps.length - 1}
            <span
              class="phase-icon"
              class:active={isActive}
              title={step.toolName ?? step.phase}
            >
              <PhaseIcon phase={step.phase} />
              {#if !isActive}
                <span class="phase-check">
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="3"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path d="M5 13l4 4L19 7" />
                  </svg>
                </span>
              {/if}
            </span>
          {/each}
        </p>
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
  .phase-trail {
    align-self: flex-start;
    padding: 0.6rem 0.9rem;
    margin: 0;
    display: flex;
    gap: 0.4rem;
  }
  .phase-icon {
    position: relative;
    display: flex;
    color: var(--text-muted);
    opacity: 0.85;
    animation: phase-pop 0.25s ease-out;
  }
  .phase-icon:not(.active) {
    color: var(--success);
  }
  .phase-icon.active {
    animation:
      phase-pop 0.25s ease-out,
      phase-wobble 1.4s ease-in-out 0.25s infinite,
      phase-pulse 2.8s ease-in-out 0.25s infinite;
  }
  .phase-check {
    position: absolute;
    bottom: -0.15rem;
    right: -0.35rem;
    width: 0.9rem;
    height: 0.9rem;
    border-radius: 999px;
    background: var(--bg);
    color: var(--success);
    display: flex;
    align-items: center;
    justify-content: center;
    animation: phase-check-pop 0.2s ease-out;
  }
  .phase-check svg {
    width: 0.65rem;
    height: 0.65rem;
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
  @keyframes phase-check-pop {
    from {
      opacity: 0;
      transform: scale(0.5);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }
  @keyframes phase-wobble {
    0%,
    100% {
      transform: rotate(-8deg);
    }
    50% {
      transform: rotate(8deg);
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
