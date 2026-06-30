export type AgentProtocol = "ag_ui" | "a2a" | "a2ui";

export type AgentEventKind =
  | "run_start"
  | "run_end"
  | "text_delta"
  | "tool_call"
  | "tool_result"
  | "state_snapshot"
  | "error";

export type ContentBlock =
  | { type: "text_delta"; delta: string }
  | { type: "tool_call"; tool_name: string; input: unknown }
  | { type: "tool_result"; tool_name: string; output: unknown; is_error: boolean }
  | { type: "state_snapshot"; state: unknown }
  | { type: "run_start"; model?: string }
  | { type: "run_end"; stop_reason?: string }
  | { type: "error"; message: string; code?: string }
  | { type: string; [key: string]: unknown };

export interface AgentEvent {
  agent_id: string;
  tenant_id: string;
  session_id: string;
  protocol: AgentProtocol;
  kind: AgentEventKind;
  run_id: string;
  content: ContentBlock;
  timestamp: string;
}
