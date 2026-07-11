<script lang="ts">
  let { role, content }: { role: "user" | "assistant"; content: string } = $props();
</script>

<div class="bubble {role}">
  <span class="content">{content}</span>
</div>

<style>
  .bubble {
    padding: 0.6rem 0.9rem;
    border-radius: 12px;
    max-width: 80%;
    white-space: pre-wrap;
  }
  .bubble.user {
    align-self: flex-end;
    background: #396cd8;
    color: white;
  }
  .bubble.assistant {
    align-self: flex-start;
    background: #ececec;
    color: #0f0f0f;
  }
</style>
