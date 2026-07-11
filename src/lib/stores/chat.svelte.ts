import { sendChatMessage, onChatDelta, onChatStatus } from "../ipc";

export interface ChatUiMessage {
  role: "user" | "assistant";
  content: string;
}

export type ChatPhase = "waking_up" | "thinking" | "calling_tool";

export interface PhaseStep {
  phase: ChatPhase;
  toolName?: string;
}

class ChatStore {
  messages = $state<ChatUiMessage[]>([]);
  streaming = $state(false);
  streamingText = $state("");
  phaseSteps = $state<PhaseStep[]>([]);
  error = $state<string | null>(null);

  async send(userText: string) {
    if (!userText.trim() || this.streaming) return;

    this.error = null;
    this.messages.push({ role: "user", content: userText });
    this.streaming = true;
    this.streamingText = "";
    this.phaseSteps = [];

    const unlistenDelta = await onChatDelta((delta) => {
      this.streamingText += delta;
    });
    const unlistenStatus = await onChatStatus((status) => {
      const step: PhaseStep = status.startsWith("calling_tool:")
        ? { phase: "calling_tool", toolName: status.slice("calling_tool:".length) }
        : { phase: status as ChatPhase };

      // Skip appending a duplicate if it's identical to the last step
      // (same phase and tool), so we don't show e.g. two thought bubbles
      // in a row for no visible reason.
      const last = this.phaseSteps.at(-1);
      if (last && last.phase === step.phase && last.toolName === step.toolName) return;

      this.phaseSteps.push(step);
    });

    try {
      const reply = await sendChatMessage(userText);
      this.messages.push({ role: "assistant", content: reply });
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      unlistenDelta();
      unlistenStatus();
      this.streaming = false;
      this.streamingText = "";
      this.phaseSteps = [];
    }
  }
}

export const chatStore = new ChatStore();
