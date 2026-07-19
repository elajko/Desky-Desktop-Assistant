import { sendChatMessage, onChatDelta, onChatStatus, onChatSentiment, type Sentiment } from "../ipc";
import { personaStore } from "./persona.svelte";

export interface ChatUiMessage {
  role: "user" | "assistant";
  content: string;
  // Only ever set on assistant messages, reflecting the sentiment of the
  // user message that prompted them — drives the bubble's love-meter styling.
  sentiment?: Sentiment;
}

export type ChatPhase = "waking_up" | "thinking" | null;

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
  phase = $state<ChatPhase>(null);
  // Reactive so the live streaming-preview bubble can show the liked/
  // disliked styling immediately too, not just the final pushed message —
  // classification finishes well before the reply starts streaming in.
  sentiment = $state<Sentiment | undefined>(undefined);
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
    this.messages.push({ role: "user", content: userText });
    this.streaming = true;
    this.streamingText = "";
    this.#revealTarget = "";
    this.phase = null;
    this.sentiment = undefined;
    this.#startReveal();

    const unlistenDelta = await onChatDelta((delta) => {
      this.#revealTarget += delta;
      // Once real output starts flowing, the phase indicator is no longer
      // relevant for this turn.
      this.phase = null;
    });
    const unlistenStatus = await onChatStatus((status) => {
      this.phase = status as ChatPhase;
    });

    const unlistenSentiment = await onChatSentiment(({ sentiment, love }) => {
      this.sentiment = sentiment;
      // The persona objects personaStore already holds are the single
      // source of truth for love values (so the number is right whichever
      // view loaded them first) — just keep the active one in sync here.
      const persona = personaStore.personas.find((p) => p.id === personaStore.activeId);
      if (persona) persona.love = love;
    });

    try {
      const reply = await sendChatMessage(userText);
      this.messages.push({ role: "assistant", content: reply, sentiment: this.sentiment });
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      unlistenDelta();
      unlistenStatus();
      unlistenSentiment();
      this.#stopReveal();
      this.streaming = false;
      this.streamingText = "";
      this.#revealTarget = "";
      this.phase = null;
      this.sentiment = undefined;
    }
  }
}

export const chatStore = new ChatStore();
