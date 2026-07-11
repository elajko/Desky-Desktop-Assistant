import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface Settings {
  llama_server_path: string | null;
  model_path: string | null;
  port: number;
  context_size: number;
}

export type LlmStatus =
  | { state: "stopped" }
  | { state: "starting" }
  | { state: "ready" }
  | { state: "crashed"; message: string };

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
