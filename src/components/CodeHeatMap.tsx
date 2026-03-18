import { useState, useEffect, useCallback } from "react";
import { listen } from "@/tauri/events";
import { Flame, FolderTree, FileCode2, Bug } from "lucide-react";
import { cn } from "@/lib/utils";
import type { FileHeatEntry, DirectoryHeatEntry } from "@/types";
import { getFileHeatMap, getDirectoryHeatMap } from "@/tauri/commands";

export interface CodeHeatMapProps {
  projectId: number | null;
  projectName?: string | null;
}

type Tab = "files" | "directories";

export function CodeHeatMap({ projectId, projectName }: CodeHeatMapProps) {
  const [tab, setTab] = useState<Tab>("files");
  const [fileEntries, setFileEntries] = useState<FileHeatEntry[]>([]);
  const [dirEntries, setDirEntries] = useState<DirectoryHeatEntry[]>([]);
  const [depth, setDepth] = useState(2);
  const [loading, setLoading] = useState(false);

  const load = useCallback(async () => {
    if (!projectId) return;
    setLoading(true);
    try {
      const [files, dirs] = await Promise.all([
        getFileHeatMap(projectId, 50),
        getDirectoryHeatMap(projectId, depth),
      ]);
      setFileEntries(files);
      setDirEntries(dirs);
    } catch (e) {
      console.error("Failed to load heat map", e);
    } finally {
      setLoading(false);
    }
  }, [projectId, depth]);

  useEffect(() => { load(); }, [load]);

  useEffect(() => {
    const unlisten = listen("db-changed", () => load());
    return () => { unlisten.then(fn => fn()); };
  }, [load]);

  if (!projectId) {
    return (
      <div className="flex flex-1 items-center justify-center text-muted-foreground">
        Select a project to view code heat map
      </div>
    );
  }

  const maxIssueCount = Math.max(...fileEntries.map(e => e.issue_count), 1);
  const maxDirIssueCount = Math.max(...dirEntries.map(e => e.issue_count), 1);

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border/50 px-6 py-4">
        <div>
          <h1 className="text-lg font-semibold">Code Heat Map</h1>
          {projectName && (
            <p className="text-sm text-muted-foreground">{projectName}</p>
          )}
        </div>
        <div className="flex items-center gap-1 rounded-lg bg-muted p-1">
          <button
            onClick={() => setTab("files")}
            className={cn(
              "rounded-md px-3 py-1.5 text-xs font-medium transition-colors",
              tab === "files" ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground"
            )}
          >
            <FileCode2 className="mr-1.5 inline-block h-3.5 w-3.5" />
            Files
          </button>
          <button
            onClick={() => setTab("directories")}
            className={cn(
              "rounded-md px-3 py-1.5 text-xs font-medium transition-colors",
              tab === "directories" ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground"
            )}
          >
            <FolderTree className="mr-1.5 inline-block h-3.5 w-3.5" />
            Directories
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-6 py-4">
        {loading ? (
          <div className="flex items-center justify-center py-12 text-muted-foreground">
            Loading...
          </div>
        ) : tab === "files" ? (
          <div>
            {fileEntries.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
                <Flame className="h-10 w-10 mb-3 opacity-30" />
                <p className="text-sm">No file links found</p>
                <p className="text-xs mt-1">Link files to issues to see the heat map</p>
              </div>
            ) : (
              <div className="space-y-1">
                {/* Header */}
                <div className="flex items-center gap-3 px-3 py-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50">
                  <span className="flex-1">File</span>
                  <span className="w-16 text-right">Issues</span>
                  <span className="w-16 text-right">Bugs</span>
                  <span className="w-48">Heat</span>
                </div>
                {fileEntries.map((entry) => {
                  const heatPct = (entry.issue_count / maxIssueCount) * 100;
                  const bugPct = entry.bug_count > 0 ? (entry.bug_count / entry.issue_count) * 100 : 0;
                  return (
                    <div
                      key={entry.file_path}
                      className="flex items-center gap-3 rounded-lg px-3 py-2.5 hover:bg-muted/50 transition-colors"
                    >
                      <FileCode2 className="h-4 w-4 text-muted-foreground/40 flex-shrink-0" />
                      <span className="flex-1 font-mono text-xs truncate">{entry.file_path}</span>
                      <span className="w-16 text-right text-sm font-medium tabular-nums">{entry.issue_count}</span>
                      <span className={cn("w-16 text-right text-sm tabular-nums", entry.bug_count > 0 ? "text-red-400 font-medium" : "text-muted-foreground/40")}>
                        {entry.bug_count > 0 && <Bug className="inline-block h-3 w-3 mr-0.5" />}
                        {entry.bug_count}
                      </span>
                      <div className="w-48 h-2 rounded-full bg-muted overflow-hidden">
                        <div
                          className="h-full rounded-full transition-all"
                          style={{
                            width: `${heatPct}%`,
                            backgroundColor: bugPct > 50 ? "#ef4444" : bugPct > 0 ? "#f59e0b" : "#3b82f6",
                          }}
                        />
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        ) : (
          <div>
            <div className="mb-4 flex items-center gap-2">
              <label className="text-xs text-muted-foreground">Depth:</label>
              <select
                value={depth}
                onChange={(e) => setDepth(Number(e.target.value))}
                className="rounded-lg border border-border bg-background px-2 py-1 text-xs"
              >
                {[1, 2, 3, 4, 5].map(d => (
                  <option key={d} value={d}>{d}</option>
                ))}
              </select>
            </div>
            {dirEntries.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
                <FolderTree className="h-10 w-10 mb-3 opacity-30" />
                <p className="text-sm">No directory data</p>
              </div>
            ) : (
              <div className="space-y-1">
                <div className="flex items-center gap-3 px-3 py-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50">
                  <span className="flex-1">Directory</span>
                  <span className="w-16 text-right">Issues</span>
                  <span className="w-16 text-right">Files</span>
                  <span className="w-48">Heat</span>
                </div>
                {dirEntries.map((entry) => {
                  const heatPct = (entry.issue_count / maxDirIssueCount) * 100;
                  return (
                    <div
                      key={entry.directory}
                      className="flex items-center gap-3 rounded-lg px-3 py-2.5 hover:bg-muted/50 transition-colors"
                    >
                      <FolderTree className="h-4 w-4 text-muted-foreground/40 flex-shrink-0" />
                      <span className="flex-1 font-mono text-xs truncate">{entry.directory}</span>
                      <span className="w-16 text-right text-sm font-medium tabular-nums">{entry.issue_count}</span>
                      <span className="w-16 text-right text-sm text-muted-foreground tabular-nums">{entry.file_count}</span>
                      <div className="w-48 h-2 rounded-full bg-muted overflow-hidden">
                        <div
                          className="h-full rounded-full bg-orange-400 transition-all"
                          style={{ width: `${heatPct}%` }}
                        />
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
