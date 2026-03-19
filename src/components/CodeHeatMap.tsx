import React, { useState, useEffect, useCallback } from "react";
import { listen } from "@/tauri/events";
import {
  Flame,
  FolderTree,
  FileCode2,
  Bug,
  ChevronRight,
  ChevronDown,
  Folder,
  FolderOpen,
  File,
  FileText,
  Settings,
  X,
  GitBranch as GitBranchIcon,
  GitCommit as GitCommitIcon,
  GitFork,
  Circle,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type { FileHeatEntry, DirectoryHeatEntry, FileTreeNode, GitStatus, GitCommit, GitBranch, GitWorktree } from "@/types";
import {
  getFileHeatMap,
  getDirectoryHeatMap,
  listProjectFiles,
  readProjectFile,
  getGitStatus,
  listGitCommits,
  listGitBranches,
  listGitWorktrees,
} from "@/tauri/commands";

export interface CodeHeatMapProps {
  projectId: number | null;
  projectName?: string | null;
}

type Tab = "files" | "directories" | "explorer" | "git";

// Config files we detect and can display
const CONFIG_FILES = [
  "CLAUDE.md",
  "AGENTS.md",
  ".claude/settings.json",
  ".codex/config.json",
];

function isConfigFile(path: string): boolean {
  return CONFIG_FILES.includes(path) || path.startsWith(".claude/") || path.startsWith(".codex/");
}

function getFileIcon(path: string, isDir: boolean, isOpen?: boolean) {
  if (isDir) {
    return isOpen
      ? <FolderOpen className="h-4 w-4 text-amber-400 flex-shrink-0" />
      : <Folder className="h-4 w-4 text-amber-400 flex-shrink-0" />;
  }
  if (path.endsWith(".md")) return <FileText className="h-4 w-4 text-blue-400 flex-shrink-0" />;
  if (path.endsWith(".json") || path.endsWith(".toml") || path.endsWith(".yaml") || path.endsWith(".yml")) {
    return <Settings className="h-4 w-4 text-green-400 flex-shrink-0" />;
  }
  if (path.endsWith(".ts") || path.endsWith(".tsx") || path.endsWith(".js") || path.endsWith(".jsx")) {
    return <FileCode2 className="h-4 w-4 text-sky-400 flex-shrink-0" />;
  }
  if (path.endsWith(".rs")) return <FileCode2 className="h-4 w-4 text-orange-400 flex-shrink-0" />;
  return <File className="h-4 w-4 text-muted-foreground/60 flex-shrink-0" />;
}

function getHeatDot(issueCount: number, bugCount: number) {
  if (issueCount === 0) return <span className="h-2 w-2 rounded-full bg-muted-foreground/20 flex-shrink-0" />;
  if (bugCount > 0) return <span className="h-2 w-2 rounded-full bg-red-500 flex-shrink-0" title={`${bugCount} bug(s)`} />;
  return <span className="h-2 w-2 rounded-full bg-blue-500 flex-shrink-0" title={`${issueCount} issue(s)`} />;
}

interface TreeNodeProps {
  node: FileTreeNode;
  depth: number;
  heatMap: Record<string, { issue_count: number; bug_count: number }>;
  onFileClick: (path: string) => void;
}

function TreeNodeRow({ node, depth, heatMap, onFileClick }: TreeNodeProps) {
  const [open, setOpen] = useState(depth === 0);
  const isDir = node.type === "dir";
  const heat = heatMap[node.path] ?? { issue_count: 0, bug_count: 0 };

  // Aggregate counts for directories
  const aggHeat = isDir ? aggregateHeat(node, heatMap) : heat;
  const baseName = node.path.split("/").pop() ?? node.path;
  const clickable = !isDir && (isConfigFile(node.path) || aggHeat.issue_count > 0);

  return (
    <div>
      <div
        className={cn(
          "flex items-center gap-1.5 rounded-md py-1 px-2 text-sm transition-colors",
          isDir ? "cursor-pointer hover:bg-muted/60" : clickable ? "cursor-pointer hover:bg-muted/60" : "cursor-default",
        )}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
        onClick={() => {
          if (isDir) setOpen(o => !o);
          else if (clickable) onFileClick(node.path);
        }}
      >
        {isDir ? (
          <span className="h-3.5 w-3.5 flex-shrink-0 text-muted-foreground">
            {open ? <ChevronDown className="h-3.5 w-3.5" /> : <ChevronRight className="h-3.5 w-3.5" />}
          </span>
        ) : (
          <span className="h-3.5 w-3.5 flex-shrink-0" />
        )}
        {getFileIcon(node.path, isDir, open)}
        <span className={cn(
          "flex-1 font-mono text-xs truncate",
          isDir ? "font-medium text-foreground" : "text-muted-foreground",
        )}>
          {baseName}
        </span>
        {aggHeat.issue_count > 0 && (
          <span className="ml-auto flex items-center gap-1">
            {getHeatDot(aggHeat.issue_count, aggHeat.bug_count)}
            <span className="text-[10px] tabular-nums text-muted-foreground">{aggHeat.issue_count}</span>
          </span>
        )}
        {aggHeat.issue_count === 0 && isDir && (
          <span className="ml-auto">
            <span className="h-2 w-2 rounded-full bg-muted-foreground/10 flex-shrink-0" />
          </span>
        )}
        {aggHeat.issue_count === 0 && !isDir && (
          <span className="ml-auto">
            {getHeatDot(0, 0)}
          </span>
        )}
        {!isDir && node.size && (
          <span className="text-[10px] text-muted-foreground/40 ml-1 tabular-nums">
            {node.size >= 1000 ? `${(node.size / 1000).toFixed(0)}k` : `${node.size}b`}
          </span>
        )}
      </div>
      {isDir && open && node.children && (
        <div>
          {node.children.map(child => (
            <TreeNodeRow
              key={child.path}
              node={child}
              depth={depth + 1}
              heatMap={heatMap}
              onFileClick={onFileClick}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function aggregateHeat(
  node: FileTreeNode,
  heatMap: Record<string, { issue_count: number; bug_count: number }>,
): { issue_count: number; bug_count: number } {
  if (node.type === "file") {
    return heatMap[node.path] ?? { issue_count: 0, bug_count: 0 };
  }
  let issue_count = 0;
  let bug_count = 0;
  for (const child of node.children ?? []) {
    const h = aggregateHeat(child, heatMap);
    issue_count += h.issue_count;
    bug_count += h.bug_count;
  }
  return { issue_count, bug_count };
}

function collectConfigFiles(nodes: FileTreeNode[]): string[] {
  const found: string[] = [];
  function walk(n: FileTreeNode) {
    if (n.type === "file" && isConfigFile(n.path)) found.push(n.path);
    if (n.children) n.children.forEach(walk);
  }
  nodes.forEach(walk);
  return found;
}

function renderFileContent(path: string, content: string) {
  const isMarkdown = path.endsWith(".md");
  const isJson = path.endsWith(".json");

  if (isMarkdown) {
    // Simple markdown rendering — convert headings/bold/code blocks to styled spans
    const lines = content.split("\n");
    return (
      <div className="space-y-1">
        {lines.map((line, i) => {
          if (line.startsWith("## ")) {
            return <h2 key={i} className="text-sm font-semibold text-foreground mt-3 mb-1">{line.slice(3)}</h2>;
          }
          if (line.startsWith("# ")) {
            return <h1 key={i} className="text-base font-bold text-foreground mt-2 mb-1">{line.slice(2)}</h1>;
          }
          if (line.startsWith("### ")) {
            return <h3 key={i} className="text-xs font-semibold text-foreground mt-2 mb-0.5 uppercase tracking-wider">{line.slice(4)}</h3>;
          }
          if (line.startsWith("```")) {
            return <div key={i} className="text-[10px] font-mono text-muted-foreground/60">{line}</div>;
          }
          if (line.startsWith("- ") || line.startsWith("* ")) {
            return (
              <div key={i} className="flex gap-1.5 text-xs text-muted-foreground">
                <span className="text-muted-foreground/40 flex-shrink-0">•</span>
                <span>{line.slice(2)}</span>
              </div>
            );
          }
          if (line.trim() === "") return <div key={i} className="h-1" />;
          return <p key={i} className="text-xs text-muted-foreground leading-relaxed">{line}</p>;
        })}
      </div>
    );
  }

  if (isJson) {
    let formatted = content;
    try { formatted = JSON.stringify(JSON.parse(content), null, 2); } catch { /* keep raw */ }
    return (
      <pre className="text-[11px] font-mono text-muted-foreground whitespace-pre-wrap break-words leading-relaxed">
        {formatted}
      </pre>
    );
  }

  return (
    <pre className="text-[11px] font-mono text-muted-foreground whitespace-pre-wrap break-words leading-relaxed">
      {content}
    </pre>
  );
}

function formatRelativeTime(timestamp: string): string {
  const diff = Date.now() - new Date(timestamp).getTime();
  const minutes = Math.floor(diff / 60_000);
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

function renderCommitMessage(message: string): React.ReactNode {
  const parts: React.ReactNode[] = [];
  const refPattern = /KAN-\d+/g;
  let lastIdx = 0;
  let m: RegExpExecArray | null;
  while ((m = refPattern.exec(message)) !== null) {
    if (m.index > lastIdx) parts.push(message.slice(lastIdx, m.index));
    parts.push(
      <span key={m.index} className="inline-flex items-center rounded-sm bg-blue-500/15 px-1 py-0.5 text-[10px] font-medium text-blue-600 dark:text-blue-400 mx-0.5">
        {m[0]}
      </span>
    );
    lastIdx = m.index + m[0].length;
  }
  if (lastIdx < message.length) parts.push(message.slice(lastIdx));
  return parts.length > 0 ? parts : message;
}

export function CodeHeatMap({ projectId, projectName }: CodeHeatMapProps) {
  const [tab, setTab] = useState<Tab>("files");
  const [fileEntries, setFileEntries] = useState<FileHeatEntry[]>([]);
  const [dirEntries, setDirEntries] = useState<DirectoryHeatEntry[]>([]);
  const [depth, setDepth] = useState(2);
  const [loading, setLoading] = useState(false);

  // Explorer state
  const [treeNodes, setTreeNodes] = useState<FileTreeNode[]>([]);
  const [heatMap, setHeatMap] = useState<Record<string, { issue_count: number; bug_count: number }>>({});
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [fileContent, setFileContent] = useState<string | null>(null);
  const [fileContentLoading, setFileContentLoading] = useState(false);
  const [configFiles, setConfigFiles] = useState<string[]>([]);

  // Git state
  const [gitStatus, setGitStatus] = useState<GitStatus | null>(null);
  const [gitCommits, setGitCommits] = useState<GitCommit[]>([]);
  const [gitBranches, setGitBranches] = useState<GitBranch[]>([]);
  const [gitWorktrees, setGitWorktrees] = useState<GitWorktree[]>([]);
  const [gitLoading, setGitLoading] = useState(false);

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

      // Build heat map index for the explorer
      const hm: Record<string, { issue_count: number; bug_count: number }> = {};
      for (const f of files) {
        hm[f.file_path] = { issue_count: f.issue_count, bug_count: f.bug_count };
      }
      setHeatMap(hm);
    } catch (e) {
      console.error("Failed to load heat map", e);
    } finally {
      setLoading(false);
    }
  }, [projectId, depth]);

  const loadTree = useCallback(async () => {
    if (!projectId) return;
    try {
      const nodes = await listProjectFiles(projectId);
      setTreeNodes(nodes);
      setConfigFiles(collectConfigFiles(nodes));
    } catch (e) {
      console.error("Failed to load file tree", e);
    }
  }, [projectId]);

  const loadGit = useCallback(async () => {
    if (!projectId) return;
    setGitLoading(true);
    try {
      const [status, commits, branches, worktrees] = await Promise.all([
        getGitStatus(projectId),
        listGitCommits(projectId, 10),
        listGitBranches(projectId),
        listGitWorktrees(projectId),
      ]);
      setGitStatus(status);
      setGitCommits(commits);
      setGitBranches(branches);
      setGitWorktrees(worktrees);
    } catch (e) {
      console.error("Failed to load git data", e);
    } finally {
      setGitLoading(false);
    }
  }, [projectId]);

  useEffect(() => { load(); }, [load]);
  useEffect(() => { loadTree(); }, [loadTree]);
  useEffect(() => { if (tab === "git") loadGit(); }, [tab, loadGit]);

  useEffect(() => {
    const unlisten = listen("db-changed", () => load());
    return () => { unlisten.then(fn => fn()); };
  }, [load]);

  const handleFileClick = useCallback(async (path: string) => {
    if (!projectId) return;
    setSelectedFile(path);
    setFileContent(null);
    setFileContentLoading(true);
    try {
      const result = await readProjectFile(projectId, path);
      setFileContent(result.content);
    } catch {
      setFileContent(null);
    } finally {
      setFileContentLoading(false);
    }
  }, [projectId]);

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
          <button
            onClick={() => setTab("explorer")}
            className={cn(
              "rounded-md px-3 py-1.5 text-xs font-medium transition-colors",
              tab === "explorer" ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground"
            )}
          >
            <Folder className="mr-1.5 inline-block h-3.5 w-3.5" />
            Explorer
          </button>
          <button
            onClick={() => setTab("git")}
            className={cn(
              "rounded-md px-3 py-1.5 text-xs font-medium transition-colors",
              tab === "git" ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground"
            )}
          >
            <GitBranchIcon className="mr-1.5 inline-block h-3.5 w-3.5" />
            Git
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden">
        {loading && tab !== "explorer" && tab !== "git" ? (
          <div className="flex items-center justify-center py-12 text-muted-foreground">
            Loading...
          </div>
        ) : tab === "files" ? (
          <div className="overflow-y-auto h-full px-6 py-4">
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
        ) : tab === "directories" ? (
          <div className="overflow-y-auto h-full px-6 py-4">
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
        ) : tab === "git" ? (
          /* Git tab */
          <div className="overflow-y-auto h-full px-6 py-4 space-y-6">
            {gitLoading ? (
              <div className="flex items-center justify-center py-12 text-muted-foreground">
                Loading git data...
              </div>
            ) : (
              <>
                {/* Section 1: Status Bar */}
                {gitStatus && (
                  <div className="flex items-center gap-3 rounded-lg border border-border/50 bg-muted/30 px-4 py-3">
                    <GitBranchIcon className="h-4 w-4 text-muted-foreground flex-shrink-0" />
                    <span className="font-mono text-sm font-medium text-foreground">{gitStatus.branch}</span>
                    <div className="flex items-center gap-2 ml-2">
                      {gitStatus.ahead > 0 && (
                        <span className="inline-flex items-center gap-1 rounded-full bg-green-500/15 px-2 py-0.5 text-[11px] font-medium text-green-600 dark:text-green-400">
                          &#8593; {gitStatus.ahead} ahead
                        </span>
                      )}
                      {gitStatus.behind > 0 && (
                        <span className="inline-flex items-center gap-1 rounded-full bg-red-500/15 px-2 py-0.5 text-[11px] font-medium text-red-600 dark:text-red-400">
                          &#8595; {gitStatus.behind} behind
                        </span>
                      )}
                      {gitStatus.uncommitted > 0 && (
                        <span className="inline-flex items-center gap-1 rounded-full bg-yellow-500/15 px-2 py-0.5 text-[11px] font-medium text-yellow-600 dark:text-yellow-400">
                          ~ {gitStatus.uncommitted} uncommitted
                        </span>
                      )}
                      {gitStatus.untracked > 0 && (
                        <span className="inline-flex items-center gap-1 rounded-full bg-muted px-2 py-0.5 text-[11px] font-medium text-muted-foreground">
                          ? {gitStatus.untracked} untracked
                        </span>
                      )}
                      {gitStatus.uncommitted === 0 && gitStatus.untracked === 0 && gitStatus.ahead === 0 && gitStatus.behind === 0 && (
                        <span className="text-[11px] text-muted-foreground">Clean</span>
                      )}
                    </div>
                  </div>
                )}

                {/* Section 2 and 3: Commits + Branches side by side */}
                <div className="grid grid-cols-2 gap-4">
                  {/* Section 2: Recent Commits */}
                  <div className="flex flex-col gap-2">
                    <div className="flex items-center gap-2">
                      <GitCommitIcon className="h-3.5 w-3.5 text-muted-foreground/60" />
                      <span className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50">Recent Commits</span>
                    </div>
                    <div className="rounded-lg border border-border/50 overflow-hidden">
                      {gitCommits.length === 0 ? (
                        <div className="flex items-center justify-center py-6 text-xs text-muted-foreground">No commits</div>
                      ) : (
                        <div className="divide-y divide-border/30">
                          {gitCommits.map((commit) => (
                            <div key={commit.hash} className="flex items-start gap-3 px-3 py-2.5 hover:bg-muted/40 transition-colors">
                              <span className="font-mono text-[10px] text-muted-foreground/50 flex-shrink-0 pt-0.5 w-14 tabular-nums">{commit.short_hash}</span>
                              <div className="flex-1 min-w-0">
                                <p className="text-xs text-foreground leading-snug break-words">
                                  {renderCommitMessage(commit.message)}
                                </p>
                                <div className="flex items-center gap-2 mt-0.5">
                                  <span className="text-[10px] text-muted-foreground/50">{commit.author}</span>
                                  <span className="text-[10px] text-muted-foreground/30">&#183;</span>
                                  <span className="text-[10px] text-muted-foreground/50">{formatRelativeTime(commit.timestamp)}</span>
                                </div>
                              </div>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>

                  {/* Section 3: Branches */}
                  <div className="flex flex-col gap-2">
                    <div className="flex items-center gap-2">
                      <GitBranchIcon className="h-3.5 w-3.5 text-muted-foreground/60" />
                      <span className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50">Branches</span>
                    </div>
                    <div className="rounded-lg border border-border/50 overflow-hidden">
                      {gitBranches.length === 0 ? (
                        <div className="flex items-center justify-center py-6 text-xs text-muted-foreground">No branches</div>
                      ) : (
                        <div className="divide-y divide-border/30">
                          {gitBranches.map((branch) => (
                            <div key={branch.name} className="flex items-start gap-2.5 px-3 py-2.5 hover:bg-muted/40 transition-colors">
                              <Circle
                                className={cn(
                                  "h-2.5 w-2.5 flex-shrink-0 mt-1",
                                  branch.is_current ? "text-green-500 fill-green-500" : "text-muted-foreground/20 fill-muted-foreground/20"
                                )}
                              />
                              <div className="flex-1 min-w-0">
                                <div className="flex items-center gap-1.5 flex-wrap">
                                  <span className="font-mono text-xs text-foreground truncate max-w-[160px]">{branch.name}</span>
                                  {branch.issue_ref && (
                                    <span className="inline-flex items-center rounded-sm bg-violet-500/15 px-1 py-0.5 text-[10px] font-medium text-violet-600 dark:text-violet-400 flex-shrink-0">
                                      {branch.issue_ref}
                                    </span>
                                  )}
                                </div>
                                <p className="text-[10px] text-muted-foreground/50 truncate mt-0.5">{branch.last_commit_message}</p>
                              </div>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>
                </div>

                {/* Section 4: Worktrees */}
                <div className="flex flex-col gap-2">
                  <div className="flex items-center gap-2">
                    <GitFork className="h-3.5 w-3.5 text-muted-foreground/60" />
                    <span className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50">Worktrees</span>
                  </div>
                  <div className="rounded-lg border border-border/50 overflow-hidden">
                    {gitWorktrees.length === 0 ? (
                      <div className="flex items-center justify-center py-6 text-xs text-muted-foreground">No worktrees</div>
                    ) : (
                      <div className="divide-y divide-border/30">
                        {gitWorktrees.map((wt) => (
                          <div
                            key={wt.path}
                            className={cn(
                              "flex items-center gap-4 px-3 py-3 hover:bg-muted/40 transition-colors",
                              !wt.agent_id && !wt.is_main && "opacity-50"
                            )}
                          >
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center gap-2">
                                <span className="font-mono text-[11px] text-muted-foreground truncate">{wt.path}</span>
                                {wt.is_main && (
                                  <span className="inline-flex items-center rounded-sm bg-muted px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground flex-shrink-0">
                                    Main
                                  </span>
                                )}
                              </div>
                              <div className="flex items-center gap-1 mt-0.5">
                                <GitBranchIcon className="h-3 w-3 text-muted-foreground/40 flex-shrink-0" />
                                <span className="font-mono text-[10px] text-muted-foreground/60 truncate">{wt.branch}</span>
                              </div>
                            </div>
                            <div className="flex items-center gap-2 flex-shrink-0">
                              {wt.agent_name ? (
                                <>
                                  <span className="h-2 w-2 rounded-full bg-green-500 flex-shrink-0" />
                                  <span className="text-xs text-muted-foreground">{wt.agent_name}</span>
                                  {wt.task_identifier && (
                                    <span className="inline-flex items-center rounded-sm bg-blue-500/15 px-1 py-0.5 text-[10px] font-medium text-blue-600 dark:text-blue-400">
                                      {wt.task_identifier}
                                    </span>
                                  )}
                                </>
                              ) : (
                                <span className="text-[10px] text-muted-foreground/40">No agent</span>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              </>
            )}
          </div>
        ) : (
          /* Explorer tab */
          <div className="flex h-full overflow-hidden">
            {/* Tree panel */}
            <div className="flex flex-col w-80 flex-shrink-0 border-r border-border/50 overflow-hidden">
              <div className="overflow-y-auto flex-1 py-2 px-1">
                {/* Config files card */}
                {configFiles.length > 0 && (
                  <div className="mx-2 mb-3 rounded-lg border border-border/50 bg-muted/30 p-3">
                    <p className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">
                      Project Config
                    </p>
                    <div className="space-y-1">
                      {configFiles.map(cf => (
                        <button
                          key={cf}
                          onClick={() => handleFileClick(cf)}
                          className={cn(
                            "flex w-full items-center gap-2 rounded-md px-2 py-1 text-left text-xs transition-colors hover:bg-muted",
                            selectedFile === cf && "bg-muted text-foreground",
                          )}
                        >
                          {getFileIcon(cf, false)}
                          <span className="font-mono truncate text-muted-foreground">{cf}</span>
                        </button>
                      ))}
                    </div>
                  </div>
                )}

                {/* File tree */}
                {treeNodes.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
                    <FolderTree className="h-8 w-8 mb-2 opacity-30" />
                    <p className="text-xs">No files</p>
                  </div>
                ) : (
                  <div>
                    {treeNodes.map(node => (
                      <TreeNodeRow
                        key={node.path}
                        node={node}
                        depth={0}
                        heatMap={heatMap}
                        onFileClick={handleFileClick}
                      />
                    ))}
                  </div>
                )}
              </div>

              {/* Legend */}
              <div className="border-t border-border/50 px-3 py-2 flex items-center gap-3">
                <span className="text-[10px] text-muted-foreground/40 uppercase tracking-wider">Heat:</span>
                <span className="flex items-center gap-1 text-[10px] text-muted-foreground">
                  <span className="h-2 w-2 rounded-full bg-red-500" /> Bugs
                </span>
                <span className="flex items-center gap-1 text-[10px] text-muted-foreground">
                  <span className="h-2 w-2 rounded-full bg-blue-500" /> Active
                </span>
                <span className="flex items-center gap-1 text-[10px] text-muted-foreground">
                  <span className="h-2 w-2 rounded-full bg-muted-foreground/20" /> Untouched
                </span>
              </div>
            </div>

            {/* File content panel */}
            <div className="flex flex-1 flex-col overflow-hidden">
              {selectedFile ? (
                <>
                  <div className="flex items-center justify-between border-b border-border/50 px-4 py-2">
                    <span className="font-mono text-xs text-muted-foreground truncate">{selectedFile}</span>
                    <button
                      onClick={() => { setSelectedFile(null); setFileContent(null); }}
                      className="ml-2 text-muted-foreground hover:text-foreground transition-colors flex-shrink-0"
                    >
                      <X className="h-4 w-4" />
                    </button>
                  </div>
                  <div className="flex-1 overflow-y-auto px-6 py-4">
                    {fileContentLoading ? (
                      <div className="flex items-center justify-center py-12 text-muted-foreground text-sm">
                        Loading...
                      </div>
                    ) : fileContent !== null ? (
                      renderFileContent(selectedFile, fileContent)
                    ) : (
                      <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
                        <File className="h-8 w-8 mb-2 opacity-30" />
                        <p className="text-sm">Content not available</p>
                        <p className="text-xs mt-1">Only config files can be previewed in browser mode</p>
                      </div>
                    )}
                  </div>
                </>
              ) : (
                <div className="flex flex-1 flex-col items-center justify-center text-muted-foreground gap-2">
                  <FileText className="h-10 w-10 opacity-20" />
                  <p className="text-sm">Select a config file to preview</p>
                  <p className="text-xs opacity-60">CLAUDE.md, AGENTS.md, and .claude/ files are supported</p>
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
