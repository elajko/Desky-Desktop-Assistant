<script lang="ts">
  import SentimentIcon from "./SentimentIcon.svelte";
  import type { Sentiment } from "../ipc";

  let {
    role,
    content,
    sentiment,
  }: { role: "user" | "assistant"; content: string; sentiment?: Sentiment } = $props();
</script>

<div
  class="bubble {role}"
  class:liked={sentiment === "liked"}
  class:disliked={sentiment === "disliked"}
>
  {#if sentiment === "liked" || sentiment === "disliked"}
    <SentimentIcon {sentiment} />
  {/if}
  <span class="content">{content}</span>
</div>

<style>
  .bubble {
    padding: 0.6rem 0.9rem;
    border-radius: 12px;
    max-width: 80%;
    white-space: pre-wrap;
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
  }
  .bubble.user {
    align-self: flex-end;
    background: var(--accent);
    color: var(--accent-text);
  }
  .bubble.assistant {
    align-self: flex-start;
    background: var(--bg-elevated);
    color: var(--text);
  }
  .bubble.assistant.liked {
    background: var(--liked-bg);
  }
  .bubble.assistant.disliked {
    background: var(--disliked-bg);
  }
</style>
