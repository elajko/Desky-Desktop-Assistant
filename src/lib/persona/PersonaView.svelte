<script lang="ts">
  import { onMount } from "svelte";
  import { personaStore } from "../stores/persona.svelte";
  import { SHORT_FIELD_MAX_LEN, type Persona } from "../ipc";

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
      sprite_sheet: null,
      is_builtin: false,
      love: 0,
      example_dialogue: "",
      first_message: "",
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
        Introduction message
        <p class="section-tip">What they say first, before you've sent anything.</p>
        <textarea
          bind:value={draft.first_message}
          rows="2"
          maxlength={SHORT_FIELD_MAX_LEN}
          placeholder="What they say first, before you send anything — e.g. &quot;Oh! You're here. I was just about to make some tea.&quot;"
        ></textarea>
        <span class="char-count">{draft.first_message.length} / {SHORT_FIELD_MAX_LEN}</span>
      </label>

      <label>
        System prompt
        <p class="section-tip">
          The "tell" layer — who they are: identity, backstory, personality. Keep it short,
          concrete, and specific ("stern only when someone threatens her forest" beats "stern"),
          and write them as a real person, not an AI playing a role.
        </p>
        <textarea bind:value={draft.system_prompt} rows="3"></textarea>
      </label>

      <label>
        Example dialogue
        <p class="section-tip">
          The "show" layer — how they actually talk teaches voice better than any description.
          Also doubles as the love meter's reference: include a line reacting to something they
          like or dislike, or leave blank to skip the love meter entirely.
        </p>
        <textarea
          bind:value={draft.example_dialogue}
          rows="3"
          maxlength={SHORT_FIELD_MAX_LEN}
          placeholder={"User: hey!\n" +
            draft.name +
            ": *tilts head* well hello there.\nUser: you're an idiot.\n" +
            draft.name +
            ": ...excuse me?"}
        ></textarea>
        <span class="char-count">{draft.example_dialogue.length} / {SHORT_FIELD_MAX_LEN}</span>
      </label>

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
  .section-tip {
    margin: 0;
    font-size: 0.78rem;
    color: var(--text-muted);
    line-height: 1.4;
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
  .char-count {
    align-self: flex-end;
    font-size: 0.75rem;
    color: var(--text-muted);
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
