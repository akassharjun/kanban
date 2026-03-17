import { describe, it, expect } from "vitest";
import { filterAgents, filterLogs } from "../agent-filters";
import type { Agent, ExecutionLog } from "@/types";

function makeAgent(overrides: Partial<Agent> = {}): Agent {
  return {
    id: "agent-1",
    name: "Test Agent",
    agent_type: "claude",
    skills: [],
    task_types: [],
    max_concurrent: 1,
    max_complexity: "medium",
    member_id: null,
    status: "online",
    registered_at: "2026-03-16",
    last_heartbeat: "2026-03-16 12:00:00",
    last_activity_at: null,
    worktree_path: null,
    ...overrides,
  };
}

function makeLog(overrides: Partial<ExecutionLog> = {}): ExecutionLog {
  return {
    id: 1,
    issue_id: 1,
    agent_id: "agent-1",
    attempt_number: 1,
    entry_type: "file_edit",
    message: "test",
    metadata: null,
    timestamp: "2026-03-16 12:00:00",
    ...overrides,
  };
}

describe("filterAgents", () => {
  it("hides offline when showInactive=false", () => {
    const agents = [
      makeAgent({ id: "1", status: "online" }),
      makeAgent({ id: "2", status: "offline" }),
    ];
    expect(filterAgents(agents, false, "all")).toHaveLength(1);
    expect(filterAgents(agents, false, "all")[0].id).toBe("1");
  });

  it("shows offline when showInactive=true", () => {
    const agents = [
      makeAgent({ id: "1", status: "online" }),
      makeAgent({ id: "2", status: "offline" }),
    ];
    expect(filterAgents(agents, true, "all")).toHaveLength(2);
  });

  it("type 'claude' includes claude-code agents", () => {
    const agents = [
      makeAgent({ id: "1", agent_type: "claude-code" }),
      makeAgent({ id: "2", agent_type: "codex" }),
    ];
    const result = filterAgents(agents, true, "claude");
    expect(result).toHaveLength(1);
    expect(result[0].id).toBe("1");
  });

  it("type 'all' returns everything", () => {
    const agents = [
      makeAgent({ id: "1", agent_type: "claude" }),
      makeAgent({ id: "2", agent_type: "codex" }),
      makeAgent({ id: "3", agent_type: "gemini" }),
    ];
    expect(filterAgents(agents, true, "all")).toHaveLength(3);
  });

  it("combines inactive + type filter", () => {
    const agents = [
      makeAgent({ id: "1", agent_type: "claude", status: "online" }),
      makeAgent({ id: "2", agent_type: "claude", status: "offline" }),
      makeAgent({ id: "3", agent_type: "codex", status: "online" }),
    ];
    const result = filterAgents(agents, false, "claude");
    expect(result).toHaveLength(1);
    expect(result[0].id).toBe("1");
  });

  it("returns empty for empty input", () => {
    expect(filterAgents([], true, "all")).toEqual([]);
  });
});

describe("filterLogs", () => {
  const logs = [
    makeLog({ id: 1, entry_type: "error" }),
    makeLog({ id: 2, entry_type: "file_edit" }),
    makeLog({ id: 3, entry_type: "error" }),
    makeLog({ id: 4, entry_type: "result" }),
  ];

  it("'all' returns everything", () => {
    expect(filterLogs(logs, "all")).toHaveLength(4);
  });

  it("'error' filters to errors only", () => {
    const result = filterLogs(logs, "error");
    expect(result).toHaveLength(2);
    expect(result.every((l) => l.entry_type === "error")).toBe(true);
  });

  it("'file_edit' filters to edits only", () => {
    const result = filterLogs(logs, "file_edit");
    expect(result).toHaveLength(1);
    expect(result[0].entry_type).toBe("file_edit");
  });

  it("no matches returns empty", () => {
    expect(filterLogs(logs, "command")).toEqual([]);
  });
});
