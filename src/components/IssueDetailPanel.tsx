import { useState, useEffect } from "react";
import { X, Copy, Trash2, Pencil, AlertCircle, SignalHigh, SignalMedium, SignalLow, Minus } from "lucide-react";
import { cn } from "@/lib/utils";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { Issue, IssueWithLabels, Status, Member, Label, ActivityLogEntry, Comment } from "@/types";
import * as api from "@/tauri/commands";

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
}

const priorities = [
  { value: "urgent", label: "Urgent", icon: AlertCircle, color: "text-red-500" },
  { value: "high", label: "High", icon: SignalHigh, color: "text-orange-500" },
  { value: "medium", label: "Medium", icon: SignalMedium, color: "text-yellow-500" },
  { value: "low", label: "Low", icon: SignalLow, color: "text-blue-500" },
  { value: "none", label: "None", icon: Minus, color: "text-muted-foreground" },
];

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
}: IssueDetailPanelProps) {
  const [issue, setIssue] = useState<IssueWithLabels | null>(null);
  const [editingTitle, setEditingTitle] = useState(false);
  const [title, setTitle] = useState("");
  const [editingDesc, setEditingDesc] = useState(false);
  const [desc, setDesc] = useState("");
  const [activity, setActivity] = useState<ActivityLogEntry[]>([]);
  const [subIssues, setSubIssues] = useState<Issue[]>([]);
  const [showPriorityMenu, setShowPriorityMenu] = useState(false);
  const [showStatusMenu, setShowStatusMenu] = useState(false);
  const [showAssigneeMenu, setShowAssigneeMenu] = useState(false);
  const [comments, setComments] = useState<Comment[]>([]);
  const [newComment, setNewComment] = useState("");
  const [estimateValue, setEstimateValue] = useState("");
  const [editingCommentId, setEditingCommentId] = useState<number | null>(null);
  const [editingCommentContent, setEditingCommentContent] = useState("");

  useEffect(() => {
    loadIssue();
  }, [issueId]);

  const loadIssue = async () => {
    try {
      const data = await api.getIssue(issueId);
      setIssue(data);
      setTitle(data.title);
      setDesc(data.description || "");
      setEstimateValue(data.estimate != null ? String(data.estimate) : "");
      const acts = await api.getActivityLog(issueId);
      setActivity(acts);
      const subs = await api.getSubIssues(issueId);
      setSubIssues(subs);
      const comms = await api.listComments(issueId);
      setComments(comms);
    } catch (e) {
      console.error("Failed to load issue", e);
    }
  };

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
    <div className="flex h-full w-[480px] flex-col border-l border-border bg-card">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <span className="text-sm text-muted-foreground">{issue.identifier}</span>
        <div className="flex items-center gap-1">
          <button onClick={() => onDuplicate(issueId)} className="rounded p-1 hover:bg-accent" title="Duplicate">
            <Copy className="h-4 w-4 text-muted-foreground" />
          </button>
          <button onClick={() => { onDelete(issueId); onClose(); }} className="rounded p-1 hover:bg-destructive/20" title="Delete">
            <Trash2 className="h-4 w-4 text-muted-foreground" />
          </button>
          <button onClick={onClose} className="rounded p-1 hover:bg-accent" title="Close (Esc)">
            <X className="h-4 w-4 text-muted-foreground" />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {/* Title */}
        <div className="px-4 pt-4">
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
              className="cursor-pointer text-lg font-semibold hover:text-primary"
            >
              {issue.title}
            </h2>
          )}
        </div>

        {/* Properties */}
        <div className="mt-4 space-y-3 px-4">
          {/* Status */}
          <div className="flex items-center gap-3 text-sm relative">
            <span className="w-20 text-muted-foreground">Status</span>
            <button
              onClick={() => setShowStatusMenu(!showStatusMenu)}
              className="flex items-center gap-1.5 rounded px-2 py-1 hover:bg-accent"
            >
              <span className="h-2 w-2 rounded-full" style={{ backgroundColor: currentStatus?.color || "#6b7280" }} />
              {currentStatus?.name || "Unknown"}
            </button>
            {showStatusMenu && (
              <div className="absolute left-20 top-8 z-50 rounded-md border border-border bg-popover p-1 shadow-lg">
                {statuses.map(s => (
                  <button
                    key={s.id}
                    onClick={async () => { await onUpdate(issueId, { status_id: s.id }); await loadIssue(); setShowStatusMenu(false); }}
                    className="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm hover:bg-accent"
                  >
                    <span className="h-2 w-2 rounded-full" style={{ backgroundColor: s.color || "#6b7280" }} />
                    {s.name}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Priority */}
          <div className="flex items-center gap-3 text-sm relative">
            <span className="w-20 text-muted-foreground">Priority</span>
            <button
              onClick={() => setShowPriorityMenu(!showPriorityMenu)}
              className="flex items-center gap-1.5 rounded px-2 py-1 hover:bg-accent"
            >
              <currentPriority.icon className={cn("h-3.5 w-3.5", currentPriority.color)} />
              {currentPriority.label}
            </button>
            {showPriorityMenu && (
              <div className="absolute left-20 top-8 z-50 rounded-md border border-border bg-popover p-1 shadow-lg">
                {priorities.map(p => (
                  <button
                    key={p.value}
                    onClick={async () => { await onUpdate(issueId, { priority: p.value }); await loadIssue(); setShowPriorityMenu(false); }}
                    className="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm hover:bg-accent"
                  >
                    <p.icon className={cn("h-3.5 w-3.5", p.color)} />
                    {p.label}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Assignee */}
          <div className="flex items-center gap-3 text-sm relative">
            <span className="w-20 text-muted-foreground">Assignee</span>
            <button
              onClick={() => setShowAssigneeMenu(!showAssigneeMenu)}
              className="flex items-center gap-1.5 rounded px-2 py-1 hover:bg-accent"
            >
              {currentMember ? (
                <>
                  <div
                    className="flex h-4 w-4 items-center justify-center rounded-full text-[9px] font-medium text-white"
                    style={{ backgroundColor: currentMember.avatar_color }}
                  >
                    {(currentMember.display_name || currentMember.name).charAt(0).toUpperCase()}
                  </div>
                  {currentMember.display_name || currentMember.name}
                </>
              ) : (
                <span className="text-muted-foreground">Unassigned</span>
              )}
            </button>
            {showAssigneeMenu && (
              <div className="absolute left-20 top-8 z-50 rounded-md border border-border bg-popover p-1 shadow-lg">
                <button
                  onClick={async () => { await onUpdate(issueId, { assignee_id: -1 }); await loadIssue(); setShowAssigneeMenu(false); }}
                  className="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm hover:bg-accent"
                >
                  Unassigned
                </button>
                {members.map(m => (
                  <button
                    key={m.id}
                    onClick={async () => { await onUpdate(issueId, { assignee_id: m.id }); await loadIssue(); setShowAssigneeMenu(false); }}
                    className="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm hover:bg-accent"
                  >
                    <div
                      className="flex h-4 w-4 items-center justify-center rounded-full text-[9px] font-medium text-white"
                      style={{ backgroundColor: m.avatar_color }}
                    >
                      {(m.display_name || m.name).charAt(0).toUpperCase()}
                    </div>
                    {m.display_name || m.name}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Labels */}
          <div className="flex items-start gap-3 text-sm">
            <span className="w-20 pt-1 text-muted-foreground">Labels</span>
            <div className="flex flex-wrap gap-1">
              {issue.labels.map(l => (
                <span key={l.id} className="rounded-full px-2 py-0.5 text-xs" style={{ backgroundColor: l.color + "20", color: l.color }}>
                  {l.name}
                </span>
              ))}
              {issue.labels.length === 0 && <span className="text-muted-foreground">None</span>}
            </div>
          </div>

          {/* Due date */}
          <div className="flex items-center gap-3 text-sm">
            <span className="w-20 text-muted-foreground">Due date</span>
            <input
              type="date"
              value={issue.due_date || ""}
              onChange={async (e) => { await onUpdate(issueId, { due_date: e.target.value || "" }); await loadIssue(); }}
              className="rounded bg-transparent px-2 py-1 text-sm outline-none hover:bg-accent"
            />
          </div>

          {/* Estimate */}
          <div className="flex items-center gap-3 text-sm">
            <span className="w-20 text-muted-foreground">Estimate</span>
            <input
              type="number"
              min="0"
              value={estimateValue}
              onChange={(e) => setEstimateValue(e.target.value)}
              onBlur={async () => {
                const parsed = estimateValue === "" ? -1 : parseFloat(estimateValue);
                const current = issue.estimate ?? -1;
                if (parsed !== current) {
                  await onUpdate(issueId, { estimate: parsed });
                  await loadIssue();
                }
              }}
              onKeyDown={async (e) => {
                if (e.key === "Enter") {
                  (e.target as HTMLElement).blur();
                }
                if (e.key === "Escape") {
                  setEstimateValue(issue.estimate != null ? String(issue.estimate) : "");
                }
              }}
              placeholder="Points"
              className="w-20 rounded bg-transparent px-2 py-1 text-sm outline-none hover:bg-accent"
            />
          </div>
        </div>

        {/* Description */}
        <div className="mt-6 border-t border-border px-4 pt-4">
          <h3 className="mb-2 text-xs font-medium text-muted-foreground">Description</h3>
          {editingDesc ? (
            <textarea
              autoFocus
              value={desc}
              onChange={(e) => setDesc(e.target.value)}
              onBlur={handleDescSave}
              rows={8}
              className="w-full rounded-md border border-border bg-background p-2 text-sm outline-none focus:border-primary"
              placeholder="Add a description... (Markdown supported)"
            />
          ) : (
            <div
              onClick={() => setEditingDesc(true)}
              className="prose prose-sm prose-invert max-w-none cursor-pointer rounded-md p-2 hover:bg-accent/30 min-h-[80px]"
            >
              {desc ? (
                <ReactMarkdown remarkPlugins={[remarkGfm]}>{desc}</ReactMarkdown>
              ) : (
                <p className="text-muted-foreground">Add a description...</p>
              )}
            </div>
          )}
        </div>

        {/* Sub-issues */}
        {subIssues.length > 0 && (
          <div className="mt-6 border-t border-border px-4 pt-4">
            <h3 className="mb-2 text-xs font-medium text-muted-foreground">
              Sub-issues ({subIssues.filter(s => {
                const st = statuses.find(st2 => st2.id === s.status_id);
                return st?.category === "completed" || st?.category === "discarded";
              }).length}/{subIssues.length})
            </h3>
            <div className="space-y-1">
              {subIssues.map(sub => {
                const subStatus = statuses.find(s => s.id === sub.status_id);
                return (
                  <button
                    key={sub.id}
                    onClick={() => onClickIssue(sub)}
                    className="flex w-full items-center gap-2 rounded px-2 py-1.5 text-sm hover:bg-accent"
                  >
                    <span className="h-2 w-2 rounded-full" style={{ backgroundColor: subStatus?.color || "#6b7280" }} />
                    <span className="text-muted-foreground">{sub.identifier}</span>
                    <span className="truncate">{sub.title}</span>
                  </button>
                );
              })}
            </div>
          </div>
        )}

        {/* Comments */}
        <div className="mt-6 border-t border-border px-4 pt-4">
          <h3 className="mb-3 text-xs font-medium text-muted-foreground">
            Comments ({comments.length})
          </h3>

          <div className="space-y-4">
            {comments.map((comment) => {
              const commentMember = members.find(m => m.id === comment.member_id);
              return (
                <div key={comment.id} className="group">
                  <div className="flex items-start gap-2">
                    {commentMember ? (
                      <div
                        className="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full text-[10px] font-medium text-white mt-0.5"
                        style={{ backgroundColor: commentMember.avatar_color }}
                      >
                        {(commentMember.display_name || commentMember.name).charAt(0).toUpperCase()}
                      </div>
                    ) : (
                      <div className="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full bg-muted text-[10px] font-medium text-muted-foreground mt-0.5">
                        S
                      </div>
                    )}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-xs font-medium">
                          {commentMember ? (commentMember.display_name || commentMember.name) : "System"}
                        </span>
                        <span className="text-[10px] text-muted-foreground">
                          {comment.created_at.slice(0, 16)}
                        </span>
                        <div className="ml-auto flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                          <button
                            onClick={() => { setEditingCommentId(comment.id); setEditingCommentContent(comment.content); }}
                            className="rounded p-0.5 hover:bg-accent"
                          >
                            <Pencil className="h-3 w-3 text-muted-foreground" />
                          </button>
                          <button
                            onClick={() => handleDeleteComment(comment.id)}
                            className="rounded p-0.5 hover:bg-destructive/20"
                          >
                            <Trash2 className="h-3 w-3 text-muted-foreground" />
                          </button>
                        </div>
                      </div>

                      {editingCommentId === comment.id ? (
                        <div className="mt-1">
                          <textarea
                            autoFocus
                            value={editingCommentContent}
                            onChange={(e) => setEditingCommentContent(e.target.value)}
                            rows={3}
                            className="w-full rounded-md border border-border bg-background p-2 text-sm outline-none focus:border-primary"
                          />
                          <div className="mt-1 flex gap-1">
                            <button
                              onClick={() => handleUpdateComment(comment.id)}
                              className="rounded-md bg-primary px-2 py-1 text-xs font-medium text-primary-foreground hover:bg-primary/90"
                            >
                              Save
                            </button>
                            <button
                              onClick={() => { setEditingCommentId(null); setEditingCommentContent(""); }}
                              className="rounded-md px-2 py-1 text-xs hover:bg-accent"
                            >
                              Cancel
                            </button>
                          </div>
                        </div>
                      ) : (
                        <div className="mt-1 prose prose-sm prose-invert max-w-none text-sm">
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
          <div className="mt-4">
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
              className="w-full rounded-md border border-border bg-background p-2 text-sm outline-none focus:border-primary"
            />
            <div className="mt-1 flex justify-end">
              <button
                onClick={handleAddComment}
                disabled={!newComment.trim()}
                className="rounded-md bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
              >
                Comment
              </button>
            </div>
          </div>
        </div>

        {/* Activity log */}
        {activity.length > 0 && (
          <div className="mt-6 border-t border-border px-4 pt-4 pb-4">
            <h3 className="mb-2 text-xs font-medium text-muted-foreground">Activity</h3>
            <div className="space-y-2">
              {activity.slice(0, 20).map(entry => (
                <div key={entry.id} className="flex items-start gap-2 text-xs text-muted-foreground">
                  <span className="shrink-0">{entry.timestamp.slice(0, 16)}</span>
                  <span>
                    Changed <span className="text-foreground">{entry.field_changed}</span>
                    {entry.old_value && <> from <span className="text-foreground">{entry.old_value}</span></>}
                    {entry.new_value && <> to <span className="text-foreground">{entry.new_value}</span></>}
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
