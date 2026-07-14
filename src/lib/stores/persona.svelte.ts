import {
  listPersonas,
  savePersona,
  deletePersona,
  setActivePersona,
  resetPersona,
  getSettings,
  type Persona,
} from "../ipc";

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
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    }
  }

  async save(persona: Persona) {
    try {
      await savePersona(persona);
      await this.load();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    }
  }

  async remove(id: string) {
    try {
      await deletePersona(id);
      await this.load();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    }
  }

  async resetToDefault(id: string) {
    try {
      await resetPersona(id);
      await this.load();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    }
  }
}

export const personaStore = new PersonaStore();
