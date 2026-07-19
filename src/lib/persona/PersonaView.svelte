<script lang="ts">
  import { onMount } from "svelte";
  import { personaStore } from "../stores/persona.svelte";
  import type { Persona } from "../ipc";

  onMount(() => {
    if (!personaStore.loaded) personaStore.load();
  });

  let draft = $state<Persona | null>(null);
  let isNewDraft = $state(false);

  function blankPersona(): Persona {
    return {
      id: crypto.randomUUID(),
      name: "",
      description: "",
      system_prompt: "You are Desky, a local desktop assistant.",
      traits: { formality: 0.5, humor: 0.5, verbosity: 0.5, proactivity: 0.5 },
      sprite_sheet: null,
      is_builtin: false,
      likes: "",
      dislikes: "",
      love: 0,
    };
  }

  function startNew() {
    draft = blankPersona();
    isNewDraft = true;
  }

  function startEdit(persona: Persona) {
    // persona here is a reactive $state proxy (from personaStore.personas) —
    // structuredClone() can throw DataCloneError on those. $state.snapshot()
    // is Svelte 5's own way to get a plain, already-disconnected copy.
    draft = $state.snapshot(persona);
    isNewDraft = false;
  }

  function cancelEdit() {
    draft = null;
  }

  async function saveDraft() {
    if (!draft || !draft.name.trim()) return;
    await personaStore.save(draft);
    draft = null;
  }

  async function remove(id: string) {
    await personaStore.remove(id);
    if (draft?.id === id) draft = null;
  }

  async function resetToDefault(id: string) {
    await personaStore.resetToDefault(id);
    if (draft?.id === id) draft = null;
  }

  const TRAIT_LABELS: { key: keyof Persona["traits"]; label: string }[] = [
    { key: "formality", label: "Formality" },
    { key: "humor", label: "Humor" },
    { key: "verbosity", label: "Verbosity" },
    { key: "proactivity", label: "Proactivity" },
  ];
</script>

<div class="persona-view">
  <div class="header">
    <h1>Persona</h1>
    <button class="new-btn" onclick={startNew}>+ New Persona</button>
  </div>

  {#if personaStore.error}
    <p class="error">{personaStore.error}</p>
  {/if}

  <div class="persona-list">
    {#each personaStore.personas as persona (persona.id)}
      <div class="persona-card" class:active={persona.id === personaStore.activeId}>
        <div class="persona-card-main">
          <div class="persona-title">
            <strong>{persona.name}</strong>
            {#if persona.id === personaStore.activeId}
              <span class="badge">Active</span>
            {/if}
            {#if persona.is_builtin}
              <span class="badge subtle">Built-in</span>
            {/if}
            <span class="badge love" title="Love meter">♥ {persona.love}</span>
          </div>
          <p class="persona-description">{persona.description}</p>
        </div>
        <div class="persona-actions">
          {#if persona.id !== personaStore.activeId}
            <button onclick={() => personaStore.activate(persona.id)}>Activate</button>
          {/if}
          <button onclick={() => startEdit(persona)}>Edit</button>
          {#if persona.is_builtin}
            <button onclick={() => resetToDefault(persona.id)}>Reset</button>
          {/if}
          <button
            class="danger"
            disabled={personaStore.personas.length <= 1}
            title={personaStore.personas.length <= 1
              ? "Can't delete the only remaining persona"
              : ""}
            onclick={() => remove(persona.id)}
          >
            Delete
          </button>
        </div>
      </div>
    {/each}
  </div>

  {#if draft}
    <div class="editor">
      <h2>{isNewDraft ? "New Persona" : `Editing ${draft.name || "Persona"}`}</h2>

      <label>
        Name
        <input bind:value={draft.name} placeholder="e.g. Cheerful" />
      </label>

      <label>
        Description
        <input bind:value={draft.description} placeholder="Short blurb shown in the list" />
      </label>

      <label>
        System prompt
        <textarea bind:value={draft.system_prompt} rows="3"></textarea>
      </label>

      <label>
        I respond positively to:
        <textarea
          bind:value={draft.likes}
          rows="2"
          placeholder="e.g. compliments, curiosity, being told a good joke"
        ></textarea>
      </label>

      <label>
        I respond negatively to:
        <textarea
          bind:value={draft.dislikes}
          rows="2"
          placeholder="e.g. rudeness, being told they're useless"
        ></textarea>
      </label>
      <p class="hint">
        Leave both blank to skip the love meter entirely for this persona — messages won't be
        judged and no extra time is spent classifying them.
      </p>

      <div class="traits">
        {#each TRAIT_LABELS as { key, label }}
          <label class="trait-slider">
            <span>{label}</span>
            <input type="range" min="0" max="1" step="0.05" bind:value={draft.traits[key]} />
            <span class="trait-value">{draft.traits[key].toFixed(2)}</span>
          </label>
        {/each}
      </div>

      <div class="editor-actions">
        <button class="save-btn" onclick={saveDraft} disabled={!draft.name.trim()}>Save</button>
        <button onclick={cancelEdit}>Cancel</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .persona-view {
    max-width: 640px;
    margin: 0 auto;
    padding: 1rem;
    padding-top: 3rem;
    box-sizing: border-box;
    height: 100vh;
    overflow-y: auto;
  }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }
  h1 {
    margin: 0;
    font-size: 1.4rem;
  }
  .new-btn {
    padding: 0.5rem 1rem;
    border-radius: 8px;
    border: none;
    background: var(--accent);
    color: var(--accent-text);
    cursor: pointer;
  }
  .persona-list {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .persona-card {
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 0.7rem 0.9rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .persona-card.active {
    border-color: var(--accent);
  }
  .persona-title {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }
  .badge {
    font-size: 0.7rem;
    padding: 0.1rem 0.5rem;
    border-radius: 999px;
    background: var(--accent);
    color: var(--accent-text);
  }
  .badge.subtle {
    background: var(--text-muted);
  }
  .badge.love {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--text-muted);
  }
  .hint {
    margin: -0.4rem 0 0;
    font-size: 0.78rem;
    color: var(--text-muted);
  }
  .persona-description {
    margin: 0.2rem 0 0;
    font-size: 0.85rem;
    opacity: 0.75;
  }
  .persona-actions {
    display: flex;
    gap: 0.4rem;
    flex-wrap: wrap;
  }
  .persona-actions button {
    padding: 0.3rem 0.7rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: transparent;
    color: inherit;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .persona-actions button.danger {
    border-color: var(--danger);
    color: var(--danger);
  }
  .persona-actions button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .editor {
    margin-top: 1.5rem;
    padding: 1rem;
    border: 1px solid var(--border);
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }
  .editor h2 {
    margin: 0;
    font-size: 1.1rem;
  }
  .editor label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.85rem;
  }
  .editor input,
  .editor textarea {
    padding: 0.5rem 0.6rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg-elevated);
    color: var(--text);
    font-family: inherit;
    font-size: 0.9rem;
  }
  .traits {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .trait-slider {
    display: grid;
    grid-template-columns: 6rem 1fr 3rem;
    align-items: center;
    gap: 0.5rem;
  }
  .trait-value {
    text-align: right;
    font-size: 0.8rem;
    opacity: 0.7;
  }
  .editor-actions {
    display: flex;
    gap: 0.5rem;
  }
  .save-btn {
    padding: 0.5rem 1rem;
    border-radius: 8px;
    border: none;
    background: var(--accent);
    color: var(--accent-text);
    cursor: pointer;
  }
  .save-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .editor-actions button:not(.save-btn) {
    padding: 0.5rem 1rem;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  .error {
    color: var(--danger);
    font-size: 0.9rem;
  }
</style>
