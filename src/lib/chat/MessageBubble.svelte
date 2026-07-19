<script lang="ts">
  import SentimentIcon from "./SentimentIcon.svelte";
  import { parseMessageParts } from "./messageParts";
  import type { Sentiment } from "../ipc";

  let {
    role,
    content,
    sentiment,
  }: { role: "user" | "assistant"; content: string; sentiment?: Sentiment } = $props();

  let parts = $derived(parseMessageParts(content));
  // The rest of the conversation shows newest content at the top — a single
  // message's own parts follow the same rule, so the part that was "said"
  // last in the string renders on top and earlier parts sink below it.
  let displayParts = $derived([...parts].reverse());
  // Only the last spoken (non-action) segment carries the sentiment
  // styling/icon — action segments are always neutral scene description,
  // regardless of how the surrounding reply was judged. Compared by
  // reference since displayParts is just a reversed copy of parts.
  let lastTextPart = $derived(parts.findLast((p) => p.type === "text"));
</script>

{#each displayParts as part}
  {#if part.type === "action"}
    <div class="bubble action">
      <span class="content">{part.text}</span>
    </div>
  {:else if role === "user"}
    <div class="bubble user">
      <span class="content">“{part.text}”</span>
    </div>
  {:else}
    <div
      class="bubble assistant"
      class:liked={sentiment === "liked" && part === lastTextPart}
      class:disliked={sentiment === "disliked" && part === lastTextPart}
    >
      {#if (sentiment === "liked" || sentiment === "disliked") && part === lastTextPart}
        <SentimentIcon {sentiment} />
      {/if}
      <span class="content">{part.text}</span>
    </div>
  {/if}
{/each}

<style>
  .bubble {
    width: 100%;
    box-sizing: border-box;
    padding: 0.6rem 0.9rem;
    border-radius: 12px;
    white-space: pre-wrap;
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
  }
  .bubble.user,
  .bubble.action {
    background: transparent;
    color: var(--text-muted);
    font-style: italic;
  }
  .bubble.assistant {
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
