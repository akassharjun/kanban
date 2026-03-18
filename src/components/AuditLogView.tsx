import { useState, useEffect, useCallback } from "react";
import { cn } from "@/lib/utils";
import type { AuditLogEntry, Member, Status } from "@/types";
import * as api from "@/tauri/commands";
import { Filter, RotateCcw } from "lucide-react";

interface AuditLogViewProps {
  projectId: number;
  statuses: Status[];
  members: Member[];
}

const PAGE_SIZE = 50;

function formatAction(entry: AuditLogEntry, statuses: Status[]): string {
  const field = entry.field_changed;
  if (field === "status_id") {
    const oldS = statuses.find((s) => s.id.toString() === entry.old_value);
    const newS = statuses.find((s) => s.id.toString() === entry.new_value);
    return `changed status from ${oldS?.name ?? entry.old_value ?? "?"} to ${newS?.name ?? entry.new_value ?? "?"}`;
  }
  if (field === "priority") {
    return `changed priority from ${entry.old_value ?? "none"} to ${entry.new_value ?? "none"}`;
  }
  if (field === "assignee_id") {
    return `changed assignee`;
  }
  if (field === "title") {
    return `renamed issue`;
  }
  if (field === "description") {
    return `updated description`;
  }
  if (field === "comment") {
    return entry.new_value ?? "left a comment";
  }
  return `changed ${field}${entry.old_value ? ` from "${entry.old_value}"` : ""}${entry.new_value ? ` to "${entry.new_value}"` : ""}`;
}

export function AuditLogView({ projectId, statuses, members }: AuditLogViewProps) {
  const [entries, setEntries] = useState<AuditLogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [hasMore, setHasMore] = useState(false);
  const [offset, setOffset] = useState(0);

  // Filters
  const [filterActorId, setFilterActorId] = useState<number | undefined>();
  const [filterField, setFilterField] = useState<string | undefined>();
  const [showFilters, setShowFilters] = useState(false);

  const loadEntries = useCallback(async (append = false) => {
    setLoading(true);
    try {
      const currentOffset = append ? offset : 0;
      const data = await api.getAuditLog({
        project_id: projectId,
        actor_id: filterActorId,
        field_changed: filterField,
        limit: PAGE_SIZE,
        offset: currentOffset,
      });
      if (append) {
        setEntries((prev) => [...prev, ...data]);
      } else {
        setEntries(data);
      }
      setHasMore(data.length === PAGE_SIZE);
      if (!append) setOffset(0);
    } catch (e) {
      console.error("Failed to load audit log", e);
    } finally {
      setLoading(false);
    }
  }, [projectId, filterActorId, filterField, offset]);

  useEffect(() => {
    loadEntries(false);
  }, [projectId, filterActorId, filterField]);

  const loadMore = () => {
    const newOffset = offset + PAGE_SIZE;
    setOffset(newOffset);
    // We need to load with the new offset
    setLoading(true);
    api.getAuditLog({
      project_id: projectId,
      actor_id: filterActorId,
      field_changed: filterField,
      limit: PAGE_SIZE,
      offset: newOffset,
    }).then((data) => {
      setEntries((prev) => [...prev, ...data]);
      setHasMore(data.length === PAGE_SIZE);
    }).catch(console.error).finally(() => setLoading(false));
  };

  const resetFilters = () => {
    setFilterActorId(undefined);
    setFilterField(undefined);
  };

  const fieldOptions = ["status_id", "priority", "assignee_id", "title", "description", "comment"];

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border/50 px-6 py-4">
        <h2 className="text-lg font-semibold">Audit Log</h2>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowFilters(!showFilters)}
            className={cn(
              "flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs transition-colors",
              showFilters ? "border-primary bg-primary/5 text-primary" : "border-border hover:bg-muted"
            )}
          >
            <Filter className="h-3 w-3" />
            Filters
          </button>
          {(filterActorId || filterField) && (
            <button
              onClick={resetFilters}
              className="flex items-center gap-1 rounded-lg border border-border px-3 py-1.5 text-xs hover:bg-muted transition-colors"
            >
              <RotateCcw className="h-3 w-3" />
              Reset
            </button>
          )}
        </div>
      </div>

      {/* Filters panel */}
      {showFilters && (
        <div className="flex items-center gap-4 border-b border-border/50 px-6 py-3 bg-muted/30">
          {/* Member filter */}
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground">Member:</span>
            <select
              value={filterActorId ?? ""}
              onChange={(e) => setFilterActorId(e.target.value ? Number(e.target.value) : undefined)}
              className="rounded-md border border-border bg-background px-2 py-1 text-xs outline-none"
            >
              <option value="">All</option>
              {members.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.display_name || m.name}
                </option>
              ))}
            </select>
          </div>

          {/* Field filter */}
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground">Action:</span>
            <select
              value={filterField ?? ""}
              onChange={(e) => setFilterField(e.target.value || undefined)}
              className="rounded-md border border-border bg-background px-2 py-1 text-xs outline-none"
            >
              <option value="">All</option>
              {fieldOptions.map((f) => (
                <option key={f} value={f}>
                  {f.replace("_id", "").replace("_", " ")}
                </option>
              ))}
            </select>
          </div>
        </div>
      )}

      {/* Entries list */}
      <div className="flex-1 overflow-y-auto">
        {entries.length === 0 && !loading && (
          <div className="py-12 text-center text-sm text-muted-foreground/40">
            No activity found
          </div>
        )}

        <div className="divide-y divide-border/30">
          {entries.map((entry) => (
            <div key={entry.id} className="flex items-start gap-3 px-6 py-3 hover:bg-muted/30 transition-colors">
              {/* Actor avatar */}
              {entry.actor_name ? (
                <div
                  className="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full text-[10px] font-semibold text-white mt-0.5"
                  style={{ backgroundColor: entry.actor_avatar_color || "#6366f1" }}
                >
                  {entry.actor_name.charAt(0).toUpperCase()}
                </div>
              ) : (
                <div className="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-muted text-[10px] font-medium text-muted-foreground mt-0.5">
                  S
                </div>
              )}

              {/* Content */}
              <div className="flex-1 min-w-0">
                <div className="flex items-baseline gap-2 flex-wrap">
                  <span className="text-[13px] font-medium">
                    {entry.actor_name || "System"}
                  </span>
                  <span className="text-[13px] text-muted-foreground/70">
                    {formatAction(entry, statuses)}
                  </span>
                  <span className="text-[13px] text-primary/80 font-medium">
                    {entry.issue_identifier}
                  </span>
                </div>
                <div className="mt-0.5 text-[11px] text-muted-foreground/40">
                  {entry.timestamp.slice(0, 16).replace("T", " ")}
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Load more */}
        {hasMore && (
          <div className="px-6 py-4">
            <button
              onClick={loadMore}
              disabled={loading}
              className="w-full rounded-lg border border-border py-2 text-xs hover:bg-muted transition-colors disabled:opacity-40"
            >
              {loading ? "Loading..." : "Load more"}
            </button>
          </div>
        )}

        {loading && entries.length === 0 && (
          <div className="py-12 text-center text-xs text-muted-foreground/40">Loading...</div>
        )}
      </div>
    </div>
  );
}
