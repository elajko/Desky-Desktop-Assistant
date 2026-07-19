export interface MessagePart {
  type: "text" | "action";
  text: string;
}

// Splits a message on *asterisk-wrapped* segments, which represent actions
// or scene descriptions and get displayed differently from spoken text.
// Segments are returned in the order they appear so they can be rendered as
// separate, sequential bubbles.
export function parseMessageParts(content: string): MessagePart[] {
  const parts: MessagePart[] = [];
  const regex = /\*([^*]+)\*/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = regex.exec(content))) {
    pushIfNotBlank(parts, "text", content.slice(lastIndex, match.index));
    pushIfNotBlank(parts, "action", match[1]);
    lastIndex = match.index + match[0].length;
  }
  pushIfNotBlank(parts, "text", content.slice(lastIndex));

  return parts;
}

function pushIfNotBlank(parts: MessagePart[], type: MessagePart["type"], raw: string) {
  const text = raw.trim();
  if (text) parts.push({ type, text });
}
