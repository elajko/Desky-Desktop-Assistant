import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface Settings {
  llama_server_path: string | null;
  model_path: string | null;
  port: number;
  context_size: number;
  active_persona_id: string | null;
}

export interface PersonaTraits {
  formality: number;
  humor: number;
  verbosity: number;
  proactivity: number;
}

export interface Persona {
  id: string;
  name: string;
  description: string;
  system_prompt: string;
  traits: PersonaTraits;
  // Placeholder for the future avatar system (spritesheet-based) — not read
  // or rendered anywhere yet.
  sprite_sheet: string | null;
  is_builtin: boolean;
}

export type LlmStatus =
  | { state: "stopped" }
  | { state: "starting" }
  | { state: "ready" }
  | { state: "crashed"; message: string };

export function sendChatMessage(message: string): Promise<void> {
  return invoke("send_chat_message", { message });
}

export function getLlmStatus(): Promise<LlmStatus> {
  return invoke("get_llm_status");
}

export function getSettings(): Promise<Settings> {
  return invoke("get_settings");
}

export function saveSettings(settings: Settings): Promise<void> {
  return invoke("save_settings", { settings });
}

export function onChatDelta(callback: (delta: string) => void): Promise<UnlistenFn> {
  return listen<string>("chat-delta", (event) => callback(event.payload));
}

export function onChatStatus(callback: (status: string) => void): Promise<UnlistenFn> {
  return listen<string>("chat-status", (event) => callback(event.payload));
}

export function onChatMessageComplete(callback: (segment: string) => void): Promise<UnlistenFn> {
  return listen<string>("chat-message-complete", (event) => callback(event.payload));
}

export function listPersonas(): Promise<Persona[]> {
  return invoke("list_personas");
}

export function savePersona(persona: Persona): Promise<void> {
  return invoke("save_persona", { persona });
}

export function deletePersona(id: string): Promise<void> {
  return invoke("delete_persona", { id });
}

export function setActivePersona(id: string): Promise<void> {
  return invoke("set_active_persona", { id });
}

export function resetPersona(id: string): Promise<Persona> {
  return invoke("reset_persona", { id });
}
