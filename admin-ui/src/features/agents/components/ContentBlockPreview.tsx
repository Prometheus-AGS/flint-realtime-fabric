import type { ContentBlock } from "../types/agent.js";

interface ContentBlockPreviewProps {
  block: ContentBlock;
}

function safeJson(value: unknown): string {
  return JSON.stringify(value) ?? "null";
}

export function ContentBlockPreview({ block }: ContentBlockPreviewProps) {
  if (block.type === "text_delta") {
    const b = block as Extract<ContentBlock, { type: "text_delta" }>;
    return <span className="content-text">{b.delta}</span>;
  }

  if (block.type === "tool_call") {
    const b = block as Extract<ContentBlock, { type: "tool_call" }>;
    return (
      <span className="content-tool">
        <strong>{b.tool_name}</strong>
        {" → "}
        <code>{safeJson(b.input)}</code>
      </span>
    );
  }

  if (block.type === "tool_result") {
    const b = block as Extract<ContentBlock, { type: "tool_result" }>;
    return (
      <span className={`content-tool-result${b.is_error ? " content-error" : ""}`}>
        <strong>{b.tool_name}</strong>
        {" ← "}
        <code>{safeJson(b.output)}</code>
      </span>
    );
  }

  if (block.type === "state_snapshot") {
    const b = block as Extract<ContentBlock, { type: "state_snapshot" }>;
    return <code className="content-state">{safeJson(b.state)}</code>;
  }

  if (block.type === "run_start") {
    const b = block as Extract<ContentBlock, { type: "run_start" }>;
    return (
      <span className="content-meta">
        {b.model ? `model: ${b.model}` : "started"}
      </span>
    );
  }

  if (block.type === "run_end") {
    const b = block as Extract<ContentBlock, { type: "run_end" }>;
    return (
      <span className="content-meta">
        {b.stop_reason ? `stop: ${b.stop_reason}` : "ended"}
      </span>
    );
  }

  if (block.type === "error") {
    const b = block as Extract<ContentBlock, { type: "error" }>;
    return (
      <span className="content-error">
        {b.message}
        {b.code ? ` [${b.code}]` : ""}
      </span>
    );
  }

  const { type, ...rest } = block as { type: string; [key: string]: unknown };
  return (
    <span className="content-unknown">
      {type}: {safeJson(rest)}
    </span>
  );
}
