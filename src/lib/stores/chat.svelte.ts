import {
  sendChatMessage,
  onChatDelta,
  onChatStatus,
  onChatMessageComplete,
  onChatPanel,
} from "../ipc";

export interface ChatTextMessage {
  kind: "text";
  role: "user" | "assistant";
  content: string;
}

export interface ChatPanelMessage {
  kind: "panel";
  tool: string;
  data: unknown;
}

export type ChatUiMessage = ChatTextMessage | ChatPanelMessage;

export type ChatPhase = "waking_up" | "thinking" | "calling_tool";

export interface PhaseStep {
  phase: ChatPhase;
  toolName?: string;
}

// Ticking a reveal buffer ourselves (rather than just showing whatever text
// has arrived) is what gives a smooth letter-by-letter typewriter effect —
// the underlying stream delivers whole tokens (often word-sized) per chunk,
// so rendering deltas directly looks like word-by-word, not character-by-
// character.
const REVEAL_TICK_MS = 16;
const REVEAL_CHARS_PER_TICK = 1;
const REVEAL_CATCHUP_THRESHOLD = 40;
const REVEAL_CATCHUP_CHARS_PER_TICK = 3;

class ChatStore {
  messages = $state<ChatUiMessage[]>([]);
  streaming = $state(false);
  streamingText = $state("");
  phaseSteps = $state<PhaseStep[]>([]);
  error = $state<string | null>(null);

  // Full text received so far for the segment currently streaming, ahead of
  // what's actually been revealed to `streamingText`.
  #revealTarget = "";
  #revealTimer: ReturnType<typeof setInterval> | null = null;

  #startReveal() {
    if (this.#revealTimer) return;
    this.#revealTimer = setInterval(() => {
      const behind = this.#revealTarget.length - this.streamingText.length;
      if (behind <= 0) return;
      const step = behind > REVEAL_CATCHUP_THRESHOLD ? REVEAL_CATCHUP_CHARS_PER_TICK : REVEAL_CHARS_PER_TICK;
      this.streamingText = this.#revealTarget.slice(0, this.streamingText.length + step);
    }, REVEAL_TICK_MS);
  }

  #stopReveal() {
    if (this.#revealTimer) {
      clearInterval(this.#revealTimer);
      this.#revealTimer = null;
    }
  }

  async send(userText: string) {
    if (!userText.trim() || this.streaming) return;

    this.error = null;
    this.messages.push({ kind: "text", role: "user", content: userText });
    this.streaming = true;
    this.streamingText = "";
    this.#revealTarget = "";
    this.phaseSteps = [];
    this.#startReveal();

    const unlistenDelta = await onChatDelta((delta) => {
      this.#revealTarget += delta;
      // Icons only represent what's happening during a silent gap — the
      // moment real output resumes, clear the trail so far. If there's
      // another gap later (e.g. a further tool call), it starts a fresh
      // trail from empty rather than piling up the whole turn's history.
      this.phaseSteps = [];
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
    // Each segment (e.g. a "let me check that for you..." preamble, then
    // later the real answer) becomes its own permanent bubble the moment
    // it's complete, rather than one growing bubble that later gets
    // silently replaced by just the final text.
    const unlistenMessageComplete = await onChatMessageComplete((segment) => {
      this.messages.push({ kind: "text", role: "assistant", content: segment });
      this.streamingText = "";
      this.#revealTarget = "";
    });
    // Some tools (e.g. get_system_info) render as a hardcoded panel instead
    // of being narrated by the model — see Tool::is_display_panel on the
    // backend. The model never even sees the real values for these.
    const unlistenPanel = await onChatPanel((panel) => {
      this.messages.push({ kind: "panel", tool: panel.tool, data: panel.data });
    });

    try {
      await sendChatMessage(userText);
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      unlistenDelta();
      unlistenStatus();
      unlistenMessageComplete();
      unlistenPanel();
      this.#stopReveal();
      this.streaming = false;
      this.streamingText = "";
      this.#revealTarget = "";
      this.phaseSteps = [];
    }
  }
}

export const chatStore = new ChatStore();
