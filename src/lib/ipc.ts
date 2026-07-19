import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface Settings {
  llama_server_path: string | null;
  model_path: string | null;
  port: number;
  context_size: number;
  active_persona_id: string | null;
}

// Recommended cap for example_dialogue/first_message — enforced via the
// textareas' maxlength in the Persona editor (single-user local app, no
// server-side validation needed).
export const SHORT_FIELD_MAX_LEN = 280;

export interface Persona {
  id: string;
  name: string;
  description: string;
  system_prompt: string;
  // Placeholder for the future avatar system (spritesheet-based) — not read
  // or rendered anywhere yet.
  sprite_sheet: string | null;
  is_builtin: boolean;
  // Unbounded in both directions, persisted per-persona.
  love: number;
  // A short sample of how this persona actually talks — shown to the model
  // as a style reference, and also the source the love meter's sentiment
  // classifier reasons from to judge incoming messages. A good example
  // should show the character reacting to something they like or dislike,
  // not just neutral small talk. Empty means "don't judge" for the love
  // meter.
  example_dialogue: string;
  // What the persona says first, before the user sends anything.
  first_message: string;
}

export type LlmStatus =
  | { state: "stopped" }
  | { state: "starting" }
  | { state: "ready" }
  | { state: "crashed"; message: string };

export type Sentiment = "liked" | "disliked" | "neutral";

export interface ChatSentimentEvent {
  sentiment: Sentiment;
  love: number;
}

export function sendChatMessage(message: string): Promise<string> {
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

export function onChatSentiment(
  callback: (event: ChatSentimentEvent) => void,
): Promise<UnlistenFn> {
  return listen<ChatSentimentEvent>("chat-sentiment", (event) => callback(event.payload));
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
