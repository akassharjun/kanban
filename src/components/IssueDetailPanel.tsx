import { useState, useEffect, useRef, useCallback } from "react";
import { X, Copy, Trash2, Pencil, AlertCircle, SignalHigh, SignalMedium, SignalLow, Minus, FileText, ChevronDown, GitBranch } from "lucide-react";
import { cn } from "@/lib/utils";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { Issue, IssueWithLabels, Status, Member, Label, ActivityLogEntry, Comment } from "@/types";
import * as api from "@/tauri/commands";
import { TaskContractDialog } from "./TaskContractDialog";

interface IssueDetailPanelProps {
  issueId: number;
  statuses: Status[];
  members: Member[];
  projectLabels: Label[];
  onClose: () => void;
  onUpdate: (id: number, input: Record<string, unknown>) => Promise<unknown>;
  onDelete: (id: number) => Promise<void>;
  onDuplicate: (id: number) => Promise<unknown>;
  onClickIssue: (issue: Issue) => void;
  onShowDependencies?: (issueId: number) => void;
}

const priorities = [
  { value: "urgent", label: "Urgent", icon: AlertCircle, color: "text-red-500" },
  { value: "high", label: "High", icon: SignalHigh, color: "text-orange-500" },
  { value: "medium", label: "Medium", icon: SignalMedium, color: "text-yellow-500" },
  { value: "low", label: "Low", icon: SignalLow, color: "text-blue-400" },
  { value: "none", label: "None", icon: Minus, color: "text-muted-foreground" },
];

/** Dropdown that auto-closes on outside click */
function Dropdown({ open, onClose, children, trigger }: {
  open: boolean;
  onClose: () => void;
  children: React.ReactNode;
  trigger: React.ReactNode;
}) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open, onClose]);

  return (
    <div ref={ref} className="relative">
      {trigger}
      {open && (
        <div className="absolute left-0 top-full mt-1 z-50 min-w-[160px] rounded-lg border border-border bg-popover p-1 shadow-lg animate-in fade-in-0 zoom-in-95 duration-100">
          {children}
        </div>
      )}
    </div>
  );
}

export function IssueDetailPanel({
  issueId,
  statuses,
  members,
  projectLabels: _projectLabels,
  onClose,
  onUpdate,
  onDelete,
  onDuplicate,
  onClickIssue,
  onShowDependencies,
}: IssueDetailPanelProps) {
  const [issue, setIssue] = useState<IssueWithLabels | null>(null);
  const [editingTitle, setEditingTitle] = useState(false);
  const [title, setTitle] = useState("");
  const [editingDesc, setEditingDesc] = useState(false);
  const [desc, setDesc] = useState("");
  const [activity, setActivity] = useState<ActivityLogEntry[]>([]);
  const [subIssues, setSubIssues] = useState<Issue[]>([]);
  const [openMenu, setOpenMenu] = useState<string | null>(null);
  const [comments, setComments] = useState<Comment[]>([]);
  const [newComment, setNewComment] = useState("");
  const [editingCommentId, setEditingCommentId] = useState<number | null>(null);
  const [editingCommentContent, setEditingCommentContent] = useState("");
  const [showTaskContractDialog, setShowTaskContractDialog] = useState(false);

  const loadIssue = useCallback(async () => {
    try {
      const data = await api.getIssue(issueId);
      setIssue(data);
      setTitle(data.title);
      setDesc(data.description || "");
      const [acts, subs, comms] = await Promise.all([
        api.getActivityLog(issueId),
        api.getSubIssues(issueId),
        api.listComments(issueId),
      ]);
      setActivity(acts);
      setSubIssues(subs);
      setComments(comms);
    } catch (e) {
      console.error("Failed to load issue", e);
    }
  }, [issueId]);

  useEffect(() => { loadIssue(); }, [loadIssue]);

  if (!issue) return null;

  const handleTitleSave = async () => {
    if (title !== issue.title) {
      await onUpdate(issueId, { title });
      await loadIssue();
    }
    setEditingTitle(false);
  };

  const handleDescSave = async () => {
    if (desc !== (issue.description || "")) {
      await onUpdate(issueId, { description: desc });
      await loadIssue();
    }
    setEditingDesc(false);
  };

  const handleAddComment = async () => {
    if (!newComment.trim()) return;
    try {
      await api.createComment({ issue_id: issueId, content: newComment.trim() });
      setNewComment("");
      await loadIssue();
    } catch (e) {
      console.error("Failed to add comment", e);
    }
  };

  const handleUpdateComment = async (commentId: number) => {
    if (!editingCommentContent.trim()) return;
    try {
      await api.updateComment(commentId, { content: editingCommentContent.trim() });
      setEditingCommentId(null);
      setEditingCommentContent("");
      await loadIssue();
    } catch (e) {
      console.error("Failed to update comment", e);
    }
  };

  const handleDeleteComment = async (commentId: number) => {
    try {
      await api.deleteComment(commentId);
      await loadIssue();
    } catch (e) {
      console.error("Failed to delete comment", e);
    }
  };

  const currentStatus = statuses.find(s => s.id === issue.status_id);
  const currentMember = members.find(m => m.id === issue.assignee_id);
  const currentPriority = priorities.find(p => p.value === issue.priority) || priorities[4];

  return (
    <div className="flex h-full w-[480px] flex-col border-l border-border/50 bg-card">
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-3">
        <span className="text-xs font-mono text-muted-foreground/60">{issue.identifier}</span>
        <div className="flex items-center gap-0.5">
          <button onClick={() => onDuplicate(issueId)} className="rounded-md p-1.5 hover:bg-muted transition-colors" title="Duplicate">
            <Copy className="h-3.5 w-3.5 text-muted-foreground" />
          </button>
          <button onClick={() => { onDelete(issueId); onClose(); }} className="rounded-md p-1.5 hover:bg-red-500/10 transition-colors" title="Delete">
            <Trash2 className="h-3.5 w-3.5 text-muted-foreground" />
          </button>
          <button onClick={onClose} className="rounded-md p-1.5 hover:bg-muted transition-colors" title="Close (Esc)">
            <X className="h-3.5 w-3.5 text-muted-foreground" />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {/* Title */}
        <div className="px-5">
          {editingTitle ? (
            <input
              autoFocus
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              onBlur={handleTitleSave}
              onKeyDown={(e) => { if (e.key === "Enter") handleTitleSave(); if (e.key === "Escape") { setTitle(issue.title); setEditingTitle(false); } }}
              className="w-full bg-transparent text-lg font-semibold outline-none"
            />
          ) : (
            <h2
              onClick={() => setEditingTitle(true)}
              className="cursor-pointer text-lg font-semibold leading-snug hover:text-primary transition-colors"
            >
              {issue.title}
            </h2>
          )}
        </div>

        {/* Properties */}
        <div className="mt-5 space-y-1 px-5">
          {/* Status */}
          <div className="flex items-center py-1.5 text-sm">
            <span className="w-24 text-[13px] text-muted-foreground/70">Status</span>
            <Dropdown
              open={openMenu === "status"}
              onClose={() => setOpenMenu(null)}
              trigger={
                <button
                  onClick={() => setOpenMenu(openMenu === "status" ? null : "status")}
                  className="flex items-center gap-2 rounded-md px-2 py-1 text-[13px] hover:bg-muted transition-colors"
                >
                  <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: currentStatus?.color || "#6b7280" }} />
                  {currentStatus?.name || "Unknown"}
                  <ChevronDown className="h-3 w-3 text-muted-foreground/40" />
                </button>
              }
            >
              {statuses.map(s => (
                <button
                  key={s.id}
                  onClick={async () => { await onUpdate(issueId, { status_id: s.id }); await loadIssue(); setOpenMenu(null); }}
                  className={cn(
                    "flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[13px] hover:bg-muted transition-colors",
                    s.id === issue.status_id && "bg-muted font-medium"
                  )}
                >
                  <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: s.color || "#6b7280" }} />
                  {s.name}
                </button>
              ))}
            </Dropdown>
          </div>

          {/* Priority */}
          <div className="flex items-center py-1.5 text-sm">
            <span className="w-24 text-[13px] text-muted-foreground/70">Priority</span>
            <Dropdown
              open={openMenu === "priority"}
              onClose={() => setOpenMenu(null)}
              trigger={
                <button
                  onClick={() => setOpenMenu(openMenu === "priority" ? null : "priority")}
                  className="flex items-center gap-2 rounded-md px-2 py-1 text-[13px] hover:bg-muted transition-colors"
                >
                  <currentPriority.icon className={cn("h-3.5 w-3.5", currentPriority.color)} />
                  {currentPriority.label}
                  <ChevronDown className="h-3 w-3 text-muted-foreground/40" />
                </button>
              }
            >
              {priorities.map(p => (
                <button
                  key={p.value}
                  onClick={async () => { await onUpdate(issueId, { priority: p.value }); await loadIssue(); setOpenMenu(null); }}
                  className={cn(
                    "flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[13px] hover:bg-muted transition-colors",
                    p.value === issue.priority && "bg-muted font-medium"
                  )}
                >
                  <p.icon className={cn("h-3.5 w-3.5", p.color)} />
                  {p.label}
                </button>
              ))}
            </Dropdown>
          </div>

          {/* Assignee */}
          <div className="flex items-center py-1.5 text-sm">
            <span className="w-24 text-[13px] text-muted-foreground/70">Assignee</span>
            <Dropdown
              open={openMenu === "assignee"}
              onClose={() => setOpenMenu(null)}
              trigger={
                <button
                  onClick={() => setOpenMenu(openMenu === "assignee" ? null : "assignee")}
                  className="flex items-center gap-2 rounded-md px-2 py-1 text-[13px] hover:bg-muted transition-colors"
                >
                  {currentMember ? (
                    <>
                      <div
                        className="flex h-5 w-5 items-center justify-center rounded-full text-[9px] font-semibold text-white"
                        style={{ backgroundColor: currentMember.avatar_color }}
                      >
                        {(currentMember.display_name || currentMember.name).charAt(0).toUpperCase()}
                      </div>
                      {currentMember.display_name || currentMember.name}
                    </>
                  ) : (
                    <span className="text-muted-foreground/60">Unassigned</span>
                  )}
                  <ChevronDown className="h-3 w-3 text-muted-foreground/40" />
                </button>
              }
            >
              <button
                onClick={async () => { await onUpdate(issueId, { assignee_id: -1 }); await loadIssue(); setOpenMenu(null); }}
                className="flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[13px] hover:bg-muted transition-colors"
              >
                <div className="h-5 w-5 rounded-full border border-dashed border-muted-foreground/30" />
                Unassigned
              </button>
              {members.map(m => (
                <button
                  key={m.id}
                  onClick={async () => { await onUpdate(issueId, { assignee_id: m.id }); await loadIssue(); setOpenMenu(null); }}
                  className={cn(
                    "flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[13px] hover:bg-muted transition-colors",
                    m.id === issue.assignee_id && "bg-muted font-medium"
                  )}
                >
                  <div
                    className="flex h-5 w-5 items-center justify-center rounded-full text-[9px] font-semibold text-white"
                    style={{ backgroundColor: m.avatar_color }}
                  >
                    {(m.display_name || m.name).charAt(0).toUpperCase()}
                  </div>
                  {m.display_name || m.name}
                </button>
              ))}
            </Dropdown>
          </div>

          {/* Labels */}
          <div className="flex items-start py-1.5 text-sm">
            <span className="w-24 pt-0.5 text-[13px] text-muted-foreground/70">Labels</span>
            <div className="flex flex-wrap gap-1">
              {issue.labels.map(l => (
                <span key={l.id} className="rounded-md px-2 py-0.5 text-[11px] font-medium" style={{ backgroundColor: l.color + "18", color: l.color }}>
                  {l.name}
                </span>
              ))}
              {issue.labels.length === 0 && <span className="text-[13px] text-muted-foreground/40">None</span>}
            </div>
          </div>

          {/* Due date */}
          <div className="flex items-center py-1.5 text-sm">
            <span className="w-24 text-[13px] text-muted-foreground/70">Due date</span>
            <input
              type="date"
              value={issue.due_date || ""}
              onChange={async (e) => { await onUpdate(issueId, { due_date: e.target.value || "" }); await loadIssue(); }}
              className="rounded-md bg-transparent px-2 py-1 text-[13px] outline-none hover:bg-muted transition-colors cursor-pointer"
            />
          </div>

          {/* Estimate */}
          <div className="flex items-center py-1.5 text-sm">
            <span className="w-24 text-[13px] text-muted-foreground/70">Estimate</span>
            <input
              type="number"
              min="0"
              value={issue.estimate ?? ""}
              onChange={async (e) => {
                const val = e.target.value === "" ? -1 : parseFloat(e.target.value);
                await onUpdate(issueId, { estimate: val });
                await loadIssue();
              }}
              placeholder="Points"
              className="w-20 rounded-md bg-transparent px-2 py-1 text-[13px] outline-none hover:bg-muted transition-colors"
            />
          </div>
        </div>

        {/* Description */}
        <div className="mt-6 border-t border-border/50 px-5 pt-4">
          <h3 className="mb-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">Description</h3>
          {editingDesc ? (
            <textarea
              autoFocus
              value={desc}
              onChange={(e) => setDesc(e.target.value)}
              onBlur={handleDescSave}
              rows={8}
              className="w-full rounded-lg border border-border bg-background p-3 text-sm outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/20"
              placeholder="Add a description... (Markdown supported)"
            />
          ) : (
            <div
              onClick={() => setEditingDesc(true)}
              className="prose prose-sm dark:prose-invert max-w-none cursor-pointer rounded-lg p-2.5 hover:bg-muted/50 min-h-[60px] transition-colors"
            >
              {desc ? (
                <ReactMarkdown remarkPlugins={[remarkGfm]}>{desc}</ReactMarkdown>
              ) : (
                <p className="text-muted-foreground/40 text-sm">Click to add a description...</p>
              )}
            </div>
          )}
        </div>

        {/* Task Contract & Dependencies */}
        <div className="mt-4 px-5 flex gap-2">
          <button
            onClick={() => setShowTaskContractDialog(true)}
            className="flex items-center gap-1.5 rounded-lg border border-border/50 px-3 py-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
          >
            <FileText className="h-3.5 w-3.5" />
            Create Task Contract
          </button>
          {onShowDependencies && (
            <button
              onClick={() => onShowDependencies(issueId)}
              className="flex items-center gap-1.5 rounded-lg border border-border/50 px-3 py-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
            >
              <GitBranch className="h-3.5 w-3.5" />
              Show Dependencies
            </button>
          )}
        </div>

        {/* Sub-issues */}
        {subIssues.length > 0 && (
          <div className="mt-6 border-t border-border/50 px-5 pt-4">
            <h3 className="mb-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">
              Sub-issues ({subIssues.filter(s => {
                const st = statuses.find(st2 => st2.id === s.status_id);
                return st?.category === "completed" || st?.category === "discarded";
              }).length}/{subIssues.length})
            </h3>
            <div className="space-y-0.5">
              {subIssues.map(sub => {
                const subStatus = statuses.find(s => s.id === sub.status_id);
                return (
                  <button
                    key={sub.id}
                    onClick={() => onClickIssue(sub)}
                    className="flex w-full items-center gap-2 rounded-lg px-2.5 py-2 text-sm hover:bg-muted transition-colors"
                  >
                    <span className="h-2 w-2 rounded-full flex-shrink-0" style={{ backgroundColor: subStatus?.color || "#6b7280" }} />
                    <span className="text-muted-foreground/60 font-mono text-xs">{sub.identifier}</span>
                    <span className="truncate">{sub.title}</span>
                  </button>
                );
              })}
            </div>
          </div>
        )}

        {/* Comments */}
        <div className="mt-6 border-t border-border/50 px-5 pt-4">
          <h3 className="mb-3 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">
            Comments ({comments.length})
          </h3>

          <div className="space-y-4">
            {comments.map((comment) => {
              const commentMember = members.find(m => m.id === comment.member_id);
              return (
                <div key={comment.id} className="group">
                  <div className="flex items-start gap-2.5">
                    {commentMember ? (
                      <div
                        className="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full text-[10px] font-semibold text-white mt-0.5"
                        style={{ backgroundColor: commentMember.avatar_color }}
                      >
                        {(commentMember.display_name || commentMember.name).charAt(0).toUpperCase()}
                      </div>
                    ) : (
                      <div className="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-muted text-[10px] font-medium text-muted-foreground mt-0.5">
                        S
                      </div>
                    )}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-[13px] font-medium">
                          {commentMember ? (commentMember.display_name || commentMember.name) : "System"}
                        </span>
                        <span className="text-[11px] text-muted-foreground/40">
                          {comment.created_at.slice(0, 16).replace("T", " ")}
                        </span>
                        <div className="ml-auto flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                          <button
                            onClick={() => { setEditingCommentId(comment.id); setEditingCommentContent(comment.content); }}
                            className="rounded-md p-1 hover:bg-muted"
                          >
                            <Pencil className="h-3 w-3 text-muted-foreground/50" />
                          </button>
                          <button
                            onClick={() => handleDeleteComment(comment.id)}
                            className="rounded-md p-1 hover:bg-red-500/10"
                          >
                            <Trash2 className="h-3 w-3 text-muted-foreground/50" />
                          </button>
                        </div>
                      </div>

                      {editingCommentId === comment.id ? (
                        <div className="mt-1.5">
                          <textarea
                            autoFocus
                            value={editingCommentContent}
                            onChange={(e) => setEditingCommentContent(e.target.value)}
                            rows={3}
                            className="w-full rounded-lg border border-border bg-background p-2.5 text-sm outline-none focus:border-primary/50"
                          />
                          <div className="mt-1.5 flex gap-1.5">
                            <button
                              onClick={() => handleUpdateComment(comment.id)}
                              className="rounded-lg bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground hover:bg-primary/90"
                            >
                              Save
                            </button>
                            <button
                              onClick={() => { setEditingCommentId(null); setEditingCommentContent(""); }}
                              className="rounded-lg px-3 py-1.5 text-xs hover:bg-muted"
                            >
                              Cancel
                            </button>
                          </div>
                        </div>
                      ) : (
                        <div className="mt-1 prose prose-sm dark:prose-invert max-w-none text-[13px] leading-relaxed">
                          <ReactMarkdown remarkPlugins={[remarkGfm]}>{comment.content}</ReactMarkdown>
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>

          {/* Add comment */}
          <div className="mt-4 pb-4">
            <textarea
              value={newComment}
              onChange={(e) => setNewComment(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
                  e.preventDefault();
                  handleAddComment();
                }
              }}
              rows={3}
              placeholder="Leave a comment... (Markdown supported, Cmd+Enter to submit)"
              className="w-full rounded-lg border border-border bg-background p-3 text-sm outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/20 placeholder:text-muted-foreground/40"
            />
            <div className="mt-2 flex justify-end">
              <button
                onClick={handleAddComment}
                disabled={!newComment.trim()}
                className="rounded-lg bg-primary px-4 py-2 text-xs font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-40 transition-colors"
              >
                Comment
              </button>
            </div>
          </div>
        </div>

        {/* Activity log */}
        {activity.length > 0 && (
          <div className="border-t border-border/50 px-5 pt-4 pb-4">
            <h3 className="mb-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">Activity</h3>
            <div className="space-y-2">
              {activity.slice(0, 20).map(entry => (
                <div key={entry.id} className="flex items-start gap-2 text-xs text-muted-foreground/60">
                  <span className="shrink-0 font-mono">{entry.timestamp.slice(0, 16)}</span>
                  <span>
                    Changed <span className="text-foreground/80">{entry.field_changed}</span>
                    {entry.old_value && <> from <span className="text-foreground/80">{entry.old_value}</span></>}
                    {entry.new_value && <> to <span className="text-foreground/80">{entry.new_value}</span></>}
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {showTaskContractDialog && (
        <TaskContractDialog
          projectId={issue.project_id}
          statusId={issue.status_id}
          defaultTitle={issue.title}
          onClose={() => setShowTaskContractDialog(false)}
          onCreated={() => loadIssue()}
        />
      )}
    </div>
  );
}
