import {
  listPersonas,
  savePersona,
  deletePersona,
  setActivePersona,
  resetPersona,
  getSettings,
  type Persona,
} from "../ipc";
import { chatStore } from "./chat.svelte";

class PersonaStore {
  personas = $state<Persona[]>([]);
  activeId = $state<string | null>(null);
  loaded = $state(false);
  error = $state<string | null>(null);

  async load() {
    try {
      const [personas, settings] = await Promise.all([listPersonas(), getSettings()]);
      this.personas = personas;
      this.activeId = settings.active_persona_id;
      this.loaded = true;
      this.error = null;
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    }
  }

  async activate(id: string) {
    try {
      await setActivePersona(id);
      this.activeId = id;
      const persona = this.personas.find((p) => p.id === id);
      chatStore.resetConversation(persona?.first_message);
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    }
  }

  async save(persona: Persona) {
    try {
      const wasActive = persona.id === this.activeId;
      await savePersona(persona);
      await this.load();
      if (wasActive) chatStore.resetConversation(persona.first_message);
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    }
  }

  async remove(id: string) {
    try {
      const wasActive = id === this.activeId;
      await deletePersona(id);
      await this.load();
      if (wasActive) {
        const fallback = this.personas.find((p) => p.id === this.activeId);
        chatStore.resetConversation(fallback?.first_message);
      }
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    }
  }

  async resetToDefault(id: string) {
    try {
      const wasActive = id === this.activeId;
      await resetPersona(id);
      await this.load();
      if (wasActive) {
        const persona = this.personas.find((p) => p.id === id);
        chatStore.resetConversation(persona?.first_message);
      }
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    }
  }
}

export const personaStore = new PersonaStore();
