import { sendChatMessage, onChatDelta } from "../ipc";

export interface ChatUiMessage {
  role: "user" | "assistant";
  content: string;
}

class ChatStore {
  messages = $state<ChatUiMessage[]>([]);
  streaming = $state(false);
  streamingText = $state("");
  error = $state<string | null>(null);

  async send(userText: string) {
    if (!userText.trim() || this.streaming) return;

    this.error = null;
    this.messages.push({ role: "user", content: userText });
    this.streaming = true;
    this.streamingText = "";

    const unlisten = await onChatDelta((delta) => {
      this.streamingText += delta;
    });

    try {
      const reply = await sendChatMessage(userText);
      this.messages.push({ role: "assistant", content: reply });
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      unlisten();
      this.streaming = false;
      this.streamingText = "";
    }
  }
}

export const chatStore = new ChatStore();
