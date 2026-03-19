import { useState, useEffect, useRef, useCallback } from "react";
import { X, Copy, Trash2, Pencil, AlertCircle, SignalHigh, SignalMedium, SignalLow, Minus, FileText, ChevronDown, History, MessageSquare, Activity, Star, GitBranch, GitPullRequest, GitCommitHorizontal, ExternalLink, CheckCircle2, XCircle, Clock, Loader2, Code, Link2, Unlink, ArrowRightLeft, Lightbulb, Zap, ChevronRight, Check, Plus } from "lucide-react";
import { cn } from "@/lib/utils";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { Issue, IssueWithLabels, Status, Member, Label, ActivityLogEntry, Comment, Epic, MilestoneWithProgress, GitLink, IssueFileLink, HandoffNote, TaskLearning } from "@/types";
import * as api from "@/tauri/commands";
import { useToast } from "./Toast";
import { TaskContractDialog } from "./TaskContractDialog";
import { IssueHistoryPanel } from "./IssueHistoryPanel";
import { MentionInput, MentionText } from "./MentionInput";
import { ReviewToolbar } from "./ReviewToolbar";
import { AttemptTabs } from "./AttemptTabs";
// DiffPreview and useExecutionLogs available for future per-attempt diff view
// import { DiffPreview } from "./DiffPreview";
// import { useExecutionLogs } from "@/hooks/use-execution-logs";

interface IssueDetailPanelProps {
  issueId: number;
  projectId?: number | null;
  statuses: Status[];
  members: Member[];
  projectLabels: Label[];
  epics?: Epic[];
  milestones?: MilestoneWithProgress[];
  onClose: () => void;
  onUpdate: (id: number, input: Record<string, unknown>) => Promise<unknown>;
  onDelete: (id: number) => Promise<void>;
  onDuplicate: (id: number) => Promise<unknown>;
  onClickIssue: (issue: Issue) => void;
  isStarred?: boolean;
  onToggleStar?: (issueId: number) => void;
  onRecordView?: (issueId: number) => void;
  onShowDependencies?: (issueId: number) => void;
  onCreateSubIssue?: (parentId: number) => void;
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
  projectId,
  statuses,
  members,
  projectLabels,
  epics,
  milestones,
  onClose,
  onUpdate,
  onDelete,
  onDuplicate,
  onClickIssue,
  isStarred,
  onToggleStar,
  onRecordView,
  onShowDependencies,
  onCreateSubIssue,
}: IssueDetailPanelProps) {
  const { showToast } = useToast();
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
  const [commentError, setCommentError] = useState<string | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [showLabelPicker, setShowLabelPicker] = useState(false);
  const [activityLimit, setActivityLimit] = useState(50);
  const isSavingRef = useRef(false);
  const [activeTab, setActiveTab] = useState<"comments" | "history" | "activity">("comments");
  const [gitLinks, setGitLinks] = useState<GitLink[]>([]);
  const [issueCommits, setIssueCommits] = useState<import("@/types").GitCommit[]>([]);
  const [issueBranches, setIssueBranches] = useState<import("@/types").GitBranch[]>([]);
  const [showGitLinkForm, setShowGitLinkForm] = useState<"branch" | "pr" | "commit" | null>(null);
  const [gitLinkRefName, setGitLinkRefName] = useState("");
  const [gitLinkUrl, setGitLinkUrl] = useState("");
  const [creatingBranch, setCreatingBranch] = useState(false);
  const [decomposeTasks, setDecomposeTasks] = useState<{ title: string; description: string | null }[] | null>(null);
  const [decomposing, setDecomposing] = useState(false);
  const [fileLinks, setFileLinks] = useState<IssueFileLink[]>([]);
  const [newFilePath, setNewFilePath] = useState("");
  const [newLinkType, setNewLinkType] = useState<"related" | "cause" | "fix">("related");
  const [showAddFile, setShowAddFile] = useState(false);
  const [handoffNotes, setHandoffNotes] = useState<HandoffNote[]>([]);
  const [taskLearnings, setTaskLearnings] = useState<TaskLearning[]>([]);
  const [taskContract, setTaskContract] = useState<import("@/types").FullTaskContract | null>(null);
  const [activeAttempt, setActiveAttempt] = useState(1);
  const [showWsjf, setShowWsjf] = useState(false);
  const [wsjfBv, setWsjfBv] = useState(5);
  const [wsjfTc, setWsjfTc] = useState(5);
  const [wsjfRr, setWsjfRr] = useState(5);
  const [wsjfSize, setWsjfSize] = useState(5);
  const [customFields, setCustomFields] = useState<import("@/types").CustomField[]>([]);
  const [customValues, setCustomValues] = useState<import("@/types").CustomFieldValue[]>([]);

  const loadIssue = useCallback(async () => {
    // Skip refresh while a save is in-flight to prevent overwriting user edits
    if (isSavingRef.current) return;
    try {
      const data = await api.getIssue(issueId);
      setIssue(data);
      setTitle(data.title);
      setDesc(data.description || "");
      if (data.business_value != null) setWsjfBv(data.business_value);
      if (data.time_criticality != null) setWsjfTc(data.time_criticality);
      if (data.risk_reduction != null) setWsjfRr(data.risk_reduction);
      if (data.job_size != null) setWsjfSize(data.job_size);
      const [acts, subs, comms, links, flinks] = await Promise.all([
        api.getActivityLog(issueId),
        api.getSubIssues(issueId),
        api.listComments(issueId),
        api.listGitLinks(issueId).catch(() => [] as GitLink[]),
        api.listFileLinks(issueId).catch(() => [] as IssueFileLink[]),
      ]);
      setActivity(acts);
      setSubIssues(subs);
      setComments(comms);
      setGitLinks(links);
      setFileLinks(flinks);
      // Load handoff notes and learnings for this issue's identifier
      try {
        const [notes, learnings] = await Promise.all([
          api.listHandoffNotes(data.identifier),
          api.getLearningsForTask(data.identifier),
        ]);
        setHandoffNotes(notes);
        setTaskLearnings(learnings);
      } catch {
        // Non-critical - don't block issue load
      }
      // Load per-issue code context (commits + branches referencing this issue)
      try {
        const pid = projectId ?? data.project_id;
        const [iCommits, iBranches] = await Promise.all([
          api.getIssueCommits(pid, data.identifier),
          api.getIssueBranches(pid, data.identifier),
        ]);
        setIssueCommits(iCommits);
        setIssueBranches(iBranches);
      } catch {
        // Non-critical
      }
      // Load task contract if exists
      try {
        const contract = await api.getTaskContract(data.identifier);
        setTaskContract(contract);
      } catch {
        setTaskContract(null);
      }
      // Load custom fields
      try {
        const pid = projectId ?? data.project_id;
        const [fields, values] = await Promise.all([
          api.listCustomFields(pid),
          api.getIssueCustomValues(issueId),
        ]);
        setCustomFields(fields);
        setCustomValues(values);
      } catch {
        // Non-critical
      }
    } catch (e) {
      console.error("Failed to load issue", e);
    }
  }, [issueId, projectId]);

  useEffect(() => { loadIssue(); }, [loadIssue]);

  // Record view when opening
  useEffect(() => {
    if (issueId && onRecordView) {
      onRecordView(issueId);
    }
  }, [issueId]); // eslint-disable-line react-hooks/exhaustive-deps

  // Reset delete confirmation when issue changes or panel reopens
  // NOTE: must be declared before the early return to follow Rules of Hooks
  useEffect(() => {
    setShowDeleteConfirm(false);
    setShowLabelPicker(false);
  }, [issueId]);

  if (!issue) return null;

  const handleTitleSave = async () => {
    if (title !== issue.title) {
      isSavingRef.current = true;
      try {
        await onUpdate(issueId, { title });
        await loadIssue();
      } catch (e) {
        console.error("Failed to save title:", e);
        showToast("Failed to save title");
      } finally {
        isSavingRef.current = false;
      }
    }
    setEditingTitle(false);
  };

  const handleDescSave = async () => {
    if (desc !== (issue.description || "")) {
      isSavingRef.current = true;
      try {
        await onUpdate(issueId, { description: desc });
        await loadIssue();
      } catch (e) {
        console.error("Failed to save description:", e);
        showToast("Failed to save description");
      } finally {
        isSavingRef.current = false;
      }
    }
    setEditingDesc(false);
  };

  const handleAddComment = async () => {
    if (!newComment.trim()) return;
    setCommentError(null);
    try {
      await api.createComment({ issue_id: issueId, content: newComment.trim() });
      setNewComment("");
      showToast("Comment added");
      await loadIssue();
    } catch (e) {
      console.error("Failed to add comment", e);
      setCommentError("Failed to add comment. Please try again.");
    }
  };

  const handleUpdateComment = async (commentId: number) => {
    if (!editingCommentContent.trim()) return;
    setCommentError(null);
    try {
      await api.updateComment(commentId, { content: editingCommentContent.trim() });
      setEditingCommentId(null);
      setEditingCommentContent("");
      await loadIssue();
    } catch (e) {
      console.error("Failed to update comment", e);
      setCommentError("Failed to update comment. Please try again.");
    }
  };

  const handleDeleteComment = async (commentId: number) => {
    setCommentError(null);
    try {
      await api.deleteComment(commentId);
      await loadIssue();
    } catch (e) {
      console.error("Failed to delete comment", e);
      setCommentError("Failed to delete comment. Please try again.");
    }
  };

  const handleAddFileLink = async () => {
    if (!newFilePath.trim()) return;
    try {
      await api.linkFileToIssue({ issue_id: issueId, file_path: newFilePath.trim(), link_type: newLinkType });
      setNewFilePath("");
      setShowAddFile(false);
      await loadIssue();
    } catch (e) {
      console.error("Failed to link file", e);
    }
  };

  const handleUnlinkFile = async (filePath: string) => {
    try {
      await api.unlinkFileFromIssue(issueId, filePath);
      await loadIssue();
    } catch (e) {
      console.error("Failed to unlink file", e);
    }
  };

  const currentStatus = statuses.find(s => s.id === issue.status_id);
  const currentMember = members.find(m => m.id === issue.assignee_id);
  const currentPriority = priorities.find(p => p.value === issue.priority) || priorities[4];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm" onMouseDown={(e) => { if (e.target === e.currentTarget) onClose(); }}>
    <div className="flex w-full max-w-[560px] max-h-[85vh] flex-col rounded-xl border border-border/50 bg-card shadow-2xl">
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-3">
        <span className="text-xs font-mono text-muted-foreground/60">{issue.identifier}</span>
        <div className="flex items-center gap-0.5">
          {onToggleStar && (
            <button
              onClick={() => onToggleStar(issueId)}
              className={cn(
                "rounded-md p-1.5 transition-colors",
                isStarred ? "text-yellow-500 hover:bg-yellow-500/10" : "hover:bg-muted"
              )}
              title={isStarred ? "Unstar" : "Star"}
            >
              <Star className={cn("h-3.5 w-3.5", isStarred ? "fill-current" : "text-muted-foreground")} />
            </button>
          )}
          <button onClick={() => onDuplicate(issueId)} className="rounded-md p-1.5 hover:bg-muted transition-colors" title="Duplicate">
            <Copy className="h-3.5 w-3.5 text-muted-foreground" />
          </button>
          {showDeleteConfirm ? (
            <button onClick={() => { onDelete(issueId); onClose(); }} className="rounded-md px-2 py-1 text-xs font-medium text-red-500 hover:bg-red-500/10 transition-colors" title="Confirm Delete">
              Confirm?
            </button>
          ) : (
            <button onClick={() => setShowDeleteConfirm(true)} className="rounded-md p-1.5 hover:bg-red-500/10 transition-colors" title="Delete">
              <Trash2 className="h-3.5 w-3.5 text-muted-foreground" />
            </button>
          )}
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
          <div className="flex items-start py-1.5 text-sm relative">
            <span className="w-24 pt-0.5 text-[13px] text-muted-foreground/70">Labels</span>
            <div className="flex flex-wrap gap-1 items-center">
              {issue.labels.map(l => (
                <span key={l.id} className="rounded-md px-2 py-0.5 text-[11px] font-medium" style={{ backgroundColor: l.color + "18", color: l.color }}>
                  {l.name}
                </span>
              ))}
              <button
                onClick={() => setShowLabelPicker(!showLabelPicker)}
                className="rounded-md p-0.5 hover:bg-muted text-muted-foreground/50 hover:text-muted-foreground transition-colors"
              >
                <Plus className="h-3.5 w-3.5" />
              </button>
            </div>
            {showLabelPicker && (
              <div className="absolute left-24 top-full mt-1 z-50 min-w-[200px] rounded-lg border border-border bg-popover p-1.5 shadow-lg">
                {projectLabels.length === 0 && (
                  <p className="px-2 py-1.5 text-xs text-muted-foreground">No labels available</p>
                )}
                {projectLabels.map(l => {
                  const isSelected = issue.labels.some(il => il.id === l.id);
                  return (
                    <button
                      key={l.id}
                      onClick={async () => {
                        const newLabelIds = isSelected
                          ? issue.labels.filter(il => il.id !== l.id).map(il => il.id)
                          : [...issue.labels.map(il => il.id), l.id];
                        await api.setIssueLabels(issueId, newLabelIds);
                        await loadIssue();
                      }}
                      className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-xs hover:bg-muted transition-colors"
                    >
                      <span className="h-2.5 w-2.5 rounded-full flex-shrink-0" style={{ backgroundColor: l.color }} />
                      <span className="flex-1 text-left">{l.name}</span>
                      {isSelected && <Check className="h-3 w-3 text-primary" />}
                    </button>
                  );
                })}
              </div>
            )}
          </div>

          {/* Epic */}
          {epics && epics.length > 0 && (
            <div className="flex items-center py-1.5 text-sm">
              <span className="w-24 text-[13px] text-muted-foreground/70">Epic</span>
              <Dropdown
                open={openMenu === "epic"}
                onClose={() => setOpenMenu(null)}
                trigger={
                  <button
                    onClick={() => setOpenMenu(openMenu === "epic" ? null : "epic")}
                    className="flex items-center gap-2 rounded-md px-2 py-1 text-[13px] hover:bg-muted transition-colors"
                  >
                    {issue.epic_id ? (
                      <>
                        <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: epics.find(e => e.id === issue.epic_id)?.color || "#6366f1" }} />
                        {epics.find(e => e.id === issue.epic_id)?.title || "Unknown"}
                      </>
                    ) : (
                      <span className="text-muted-foreground/60">None</span>
                    )}
                    <ChevronDown className="h-3 w-3 text-muted-foreground/40" />
                  </button>
                }
              >
                <button
                  onClick={async () => { await onUpdate(issueId, { epic_id: -1 }); await loadIssue(); setOpenMenu(null); }}
                  className="flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[13px] hover:bg-muted transition-colors"
                >
                  None
                </button>
                {epics.filter(e => e.status === "active").map(e => (
                  <button
                    key={e.id}
                    onClick={async () => { await onUpdate(issueId, { epic_id: e.id }); await loadIssue(); setOpenMenu(null); }}
                    className={cn(
                      "flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[13px] hover:bg-muted transition-colors",
                      e.id === issue.epic_id && "bg-muted font-medium"
                    )}
                  >
                    <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: e.color }} />
                    {e.title}
                  </button>
                ))}
              </Dropdown>
            </div>
          )}

          {/* Milestone */}
          {milestones && milestones.length > 0 && (
            <div className="flex items-center py-1.5 text-sm">
              <span className="w-24 text-[13px] text-muted-foreground/70">Milestone</span>
              <Dropdown
                open={openMenu === "milestone"}
                onClose={() => setOpenMenu(null)}
                trigger={
                  <button
                    onClick={() => setOpenMenu(openMenu === "milestone" ? null : "milestone")}
                    className="flex items-center gap-2 rounded-md px-2 py-1 text-[13px] hover:bg-muted transition-colors"
                  >
                    {issue.milestone_id ? (
                      milestones.find(m => m.id === issue.milestone_id)?.title || "Unknown"
                    ) : (
                      <span className="text-muted-foreground/60">None</span>
                    )}
                    <ChevronDown className="h-3 w-3 text-muted-foreground/40" />
                  </button>
                }
              >
                <button
                  onClick={async () => { await onUpdate(issueId, { milestone_id: -1 }); await loadIssue(); setOpenMenu(null); }}
                  className="flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[13px] hover:bg-muted transition-colors"
                >
                  None
                </button>
                {milestones.filter(m => m.status === "open").map(m => (
                  <button
                    key={m.id}
                    onClick={async () => { await onUpdate(issueId, { milestone_id: m.id }); await loadIssue(); setOpenMenu(null); }}
                    className={cn(
                      "flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[13px] hover:bg-muted transition-colors",
                      m.id === issue.milestone_id && "bg-muted font-medium"
                    )}
                  >
                    {m.title}
                    {m.due_date && <span className="text-[10px] text-muted-foreground/50 ml-auto">{m.due_date}</span>}
                  </button>
                ))}
              </Dropdown>
            </div>
          )}

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

        </div>

        {/* Custom Fields */}
        {customFields.length > 0 && (
          <div className="mt-4 border-t border-border/50 px-5 pt-3">
            <h3 className="mb-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">Custom Fields</h3>
            <div className="space-y-0.5">
              {customFields.map(field => {
                const cv = customValues.find(v => v.field_id === field.id);
                const currentValue = cv?.value ?? "";
                const handleChange = async (val: string | null) => {
                  await api.setIssueCustomValue(issueId, field.id, val);
                  const updated = await api.getIssueCustomValues(issueId);
                  setCustomValues(updated);
                };
                if (field.field_type === "select") {
                  let options: string[] = [];
                  try { options = field.options ? JSON.parse(field.options) : []; } catch { options = []; }
                  return (
                    <div key={field.id} className="flex items-center py-1.5 text-sm">
                      <span className="w-24 text-[13px] text-muted-foreground/70">{field.name}</span>
                      <select
                        value={currentValue}
                        onChange={async (e) => handleChange(e.target.value || null)}
                        className="rounded-md bg-transparent px-2 py-1 text-[13px] outline-none hover:bg-muted transition-colors cursor-pointer"
                      >
                        <option value="">— None —</option>
                        {options.map(opt => <option key={opt} value={opt}>{opt}</option>)}
                      </select>
                    </div>
                  );
                }
                if (field.field_type === "number") {
                  return (
                    <div key={field.id} className="flex items-center py-1.5 text-sm">
                      <span className="w-24 text-[13px] text-muted-foreground/70">{field.name}</span>
                      <input
                        type="number"
                        value={currentValue}
                        onChange={async (e) => handleChange(e.target.value || null)}
                        placeholder="—"
                        className="w-20 rounded-md bg-transparent px-2 py-1 text-[13px] outline-none hover:bg-muted transition-colors"
                      />
                    </div>
                  );
                }
                // text (default)
                return (
                  <div key={field.id} className="flex items-center py-1.5 text-sm">
                    <span className="w-24 text-[13px] text-muted-foreground/70">{field.name}</span>
                    <input
                      type="text"
                      value={currentValue}
                      onChange={(e) => setCustomValues(prev => {
                        const next = prev.filter(v => v.field_id !== field.id);
                        if (e.target.value) next.push({ id: cv?.id ?? 0, issue_id: issueId, field_id: field.id, value: e.target.value });
                        return next;
                      })}
                      onBlur={async (e) => handleChange(e.target.value || null)}
                      placeholder="—"
                      className="flex-1 rounded-md bg-transparent px-2 py-1 text-[13px] outline-none hover:bg-muted focus:bg-muted transition-colors"
                    />
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* WSJF Scoring */}
        <div className="mt-4 border-t border-border/50 px-5 pt-3">
          <button
            onClick={() => setShowWsjf(!showWsjf)}
            className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50 hover:text-muted-foreground transition-colors"
          >
            <ChevronRight className={cn("h-3 w-3 transition-transform", showWsjf && "rotate-90")} />
            WSJF Score
            {issue.wsjf_score != null && (
              <span className="ml-1 rounded-md bg-primary/10 px-1.5 py-0.5 text-[10px] font-bold text-primary normal-case tracking-normal">
                {issue.wsjf_score.toFixed(1)}
              </span>
            )}
          </button>
          {showWsjf && (
            <div className="mt-3 space-y-3">
              {(
                [
                  ["Business Value", wsjfBv, setWsjfBv, "How much value to users/business"],
                  ["Time Criticality", wsjfTc, setWsjfTc, "How urgent based on deadlines"],
                  ["Risk Reduction", wsjfRr, setWsjfRr, "Risk/opportunity if done"],
                  ["Job Size", wsjfSize, setWsjfSize, "Effort required (higher = bigger)"],
                ] as [string, number, (v: number) => void, string][]
              ).map(([label, value, setter, tooltip]) => (
                <div key={label} className="flex items-center gap-3">
                  <span className="w-28 text-[12px] text-muted-foreground/70" title={tooltip}>{label}</span>
                  <input
                    type="range"
                    min={1}
                    max={10}
                    value={value}
                    onChange={(e) => setter(Number(e.target.value))}
                    className="h-1.5 flex-1 appearance-none rounded-full bg-muted accent-primary"
                  />
                  <span className="w-6 text-center text-[12px] font-medium">{value}</span>
                </div>
              ))}
              <div className="flex items-center justify-between pt-1">
                <div className="text-[12px] text-muted-foreground/70">
                  Score: <span className="font-bold text-foreground">{wsjfSize > 0 ? ((wsjfBv + wsjfTc + wsjfRr) / wsjfSize).toFixed(2) : "0.00"}</span>
                  <span className="ml-1 text-[10px] text-muted-foreground/50">= ({wsjfBv}+{wsjfTc}+{wsjfRr}) / {wsjfSize}</span>
                </div>
              </div>
              <div className="flex items-center gap-2 pt-1">
                <button
                  onClick={async () => {
                    try {
                      await api.setWsjfScores({ issue_id: issueId, business_value: wsjfBv, time_criticality: wsjfTc, risk_reduction: wsjfRr, job_size: wsjfSize });
                      await loadIssue();
                    } catch (e) { console.error("Failed to set WSJF", e); }
                  }}
                  className="rounded-md bg-primary/10 px-3 py-1.5 text-[11px] font-medium text-primary hover:bg-primary/20 transition-colors"
                >
                  Save Score
                </button>
                <button
                  onClick={async () => {
                    try {
                      const result = await api.autoScoreIssue(issueId);
                      setWsjfBv(result.business_value);
                      setWsjfTc(result.time_criticality);
                      setWsjfRr(result.risk_reduction);
                      setWsjfSize(result.job_size);
                      await loadIssue();
                    } catch (e) { console.error("Failed to auto-score", e); }
                  }}
                  className="flex items-center gap-1 rounded-md border border-border/50 px-3 py-1.5 text-[11px] text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
                >
                  <Zap className="h-3 w-3" />
                  Auto Score
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Description */}
        <div className="mt-4 border-t border-border/50 px-5 pt-4">
          <h3 className="mb-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">Description</h3>
          {editingDesc ? (
            <div onBlur={handleDescSave}>
              <MentionInput
                autoFocus
                value={desc}
                onChange={setDesc}
                rows={8}
                placeholder="Add a description... (@ to mention, Markdown supported)"
                members={members}
              />
            </div>
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

        {/* Git / GitHub */}
        <div className="mt-6 border-t border-border/50 px-5 pt-4">
          <h3 className="mb-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">Git & GitHub</h3>

          {/* Existing git links */}
          {gitLinks.length > 0 && (
            <div className="space-y-1.5 mb-3">
              {gitLinks.map(link => (
                <div key={link.id} className="flex items-center gap-2 rounded-lg border border-border/50 px-3 py-2 text-xs">
                  {link.link_type === "branch" && <GitBranch className="h-3.5 w-3.5 text-muted-foreground flex-shrink-0" />}
                  {link.link_type === "pr" && <GitPullRequest className="h-3.5 w-3.5 text-muted-foreground flex-shrink-0" />}

                  <span className="font-mono truncate flex-1">
                    {link.link_type === "pr" && link.pr_number ? `#${link.pr_number} ` : ""}
                    {link.ref_name}
                  </span>

                  {/* PR state badges */}
                  {link.link_type === "pr" && link.pr_state && (
                    <span className={cn(
                      "rounded-full px-1.5 py-0.5 text-[10px] font-medium",
                      link.pr_merged ? "bg-purple-500/10 text-purple-500" :
                      link.pr_state === "open" ? "bg-green-500/10 text-green-500" :
                      "bg-red-500/10 text-red-500"
                    )}>
                      {link.pr_merged ? "merged" : link.pr_state}
                    </span>
                  )}

                  {/* Review status */}
                  {link.review_status && link.review_status !== "none" && (
                    <span className={cn(
                      "flex items-center gap-0.5 text-[10px]",
                      link.review_status === "approved" ? "text-green-500" :
                      link.review_status === "changes_requested" ? "text-orange-500" :
                      "text-yellow-500"
                    )}>
                      {link.review_status === "approved" && <CheckCircle2 className="h-3 w-3" />}
                      {link.review_status === "changes_requested" && <XCircle className="h-3 w-3" />}
                      {link.review_status === "pending" && <Clock className="h-3 w-3" />}
                    </span>
                  )}

                  {/* CI status */}
                  {link.ci_status && (
                    <span className={cn(
                      "h-2 w-2 rounded-full flex-shrink-0",
                      link.ci_status === "success" ? "bg-green-500" :
                      link.ci_status === "failure" ? "bg-red-500" :
                      "bg-yellow-500"
                    )} title={`CI: ${link.ci_status}`} />
                  )}

                  {link.url && (
                    <a href={link.url} target="_blank" rel="noopener noreferrer" className="text-muted-foreground hover:text-foreground">
                      <ExternalLink className="h-3 w-3" />
                    </a>
                  )}
                </div>
              ))}
            </div>
          )}

          {/* Create branch button */}
          {projectId && issue && (
            <button
              onClick={async () => {
                setCreatingBranch(true);
                try {
                  await api.createBranchForIssue(projectId, issue.identifier);
                  await loadIssue();
                } catch (e) {
                  console.error("Failed to create branch", e);
                } finally {
                  setCreatingBranch(false);
                }
              }}
              disabled={creatingBranch}
              className="flex items-center gap-1.5 rounded-lg border border-border/50 px-3 py-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors disabled:opacity-50"
            >
              {creatingBranch ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <GitBranch className="h-3.5 w-3.5" />}
              Create Branch
            </button>
          )}
        </div>

        {/* Review Toolbar for tasks in validating state */}
        {taskContract && taskContract.task_state === "validating" && issue && (
          <div className="mt-4 px-5">
            <ReviewToolbar
              issueIdentifier={issue.identifier}
              onActionComplete={loadIssue}
            />
          </div>
        )}

        {/* Attempt Tabs for tasks with multiple attempts */}
        {taskContract && taskContract.attempt_count > 1 && issue && (
          <div className="mt-4 px-5">
            <AttemptTabs
              attempts={Array.from({ length: taskContract.attempt_count }, (_, i) => ({
                number: i + 1,
                confidence: i + 1 === taskContract.attempt_count ? taskContract.confidence : null,
                agentName: taskContract.claimed_by || "unknown",
              }))}
              activeAttempt={activeAttempt}
              onSelect={setActiveAttempt}
            />
          </div>
        )}

        {/* Task Contract, Decompose & Dependencies */}
        <div className="mt-4 px-5 flex gap-2">
          <button
            onClick={() => setShowTaskContractDialog(true)}
            className="flex items-center gap-1.5 rounded-lg border border-border/50 px-3 py-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
          >
            <FileText className="h-3.5 w-3.5" />
            Create Task Contract
          </button>
          <button
            onClick={async () => {
              try {
                setDecomposing(true);
                const tasks = await api.decomposeIssue(issueId);
                setDecomposeTasks(tasks);
              } catch (e) {
                console.error("Decompose failed", e);
              } finally {
                setDecomposing(false);
              }
            }}
            disabled={decomposing}
            className="flex items-center gap-1.5 rounded-lg border border-border/50 px-3 py-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
          >
            <GitBranch className="h-3.5 w-3.5" />
            {decomposing ? "Analyzing..." : "Decompose"}
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

        {/* Git Links */}
        <div className="mt-6 border-t border-border/50 px-5 pt-4">
          <h3 className="mb-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">
            Git ({gitLinks.length})
          </h3>
          {gitLinks.length > 0 && (
            <div className="space-y-1.5 mb-3">
              {gitLinks.map((link) => {
                const Icon = link.link_type === "branch" ? GitBranch : link.link_type === "pr" ? GitPullRequest : GitCommitHorizontal;
                const linkStatus = link.pr_merged ? "merged" : link.pr_state === "closed" ? "closed" : "open";
                const statusColor = linkStatus === "merged" ? "text-purple-500 bg-purple-500/10" : linkStatus === "closed" ? "text-red-500 bg-red-500/10" : "text-green-500 bg-green-500/10";
                return (
                  <div key={link.id} className="group flex items-center gap-2 rounded-lg px-2.5 py-2 hover:bg-muted transition-colors">
                    <Icon className="h-3.5 w-3.5 text-muted-foreground flex-shrink-0" />
                    <span className="text-[13px] font-mono truncate flex-1">{link.ref_name}</span>
                    <span className={cn("text-[10px] font-medium rounded-full px-2 py-0.5", statusColor)}>
                      {linkStatus}
                    </span>
                    {link.url && (
                      <a href={link.url} target="_blank" rel="noopener noreferrer" onClick={(e) => e.stopPropagation()} className="text-muted-foreground/50 hover:text-foreground">
                        <ExternalLink className="h-3 w-3" />
                      </a>
                    )}
                    <button
                      onClick={async () => { await api.deleteGitLink(link.id); await loadIssue(); }}
                      className="opacity-0 group-hover:opacity-100 rounded-md p-1 hover:bg-red-500/10 transition-opacity"
                    >
                      <Trash2 className="h-3 w-3 text-muted-foreground/50" />
                    </button>
                  </div>
                );
              })}
            </div>
          )}

          {showGitLinkForm ? (
            <div className="rounded-lg border border-border bg-background p-3 space-y-2">
              <div className="text-[11px] font-medium text-muted-foreground/70 capitalize">
                {showGitLinkForm === "pr" ? "Pull Request" : showGitLinkForm}
              </div>
              <input
                autoFocus
                value={gitLinkRefName}
                onChange={(e) => setGitLinkRefName(e.target.value)}
                placeholder={showGitLinkForm === "branch" ? "Branch name..." : showGitLinkForm === "pr" ? "PR number (e.g. #42)..." : "Commit SHA..."}
                className="w-full rounded-md border border-border bg-transparent px-2.5 py-1.5 text-sm outline-none focus:border-primary/50"
              />
              <input
                value={gitLinkUrl}
                onChange={(e) => setGitLinkUrl(e.target.value)}
                placeholder="URL (optional)..."
                className="w-full rounded-md border border-border bg-transparent px-2.5 py-1.5 text-sm outline-none focus:border-primary/50"
              />
              <div className="flex gap-1.5">
                <button
                  onClick={async () => {
                    if (!gitLinkRefName.trim()) return;
                    await api.createGitLink({
                      issue_id: issueId,
                      link_type: showGitLinkForm,
                      ref_name: gitLinkRefName.trim(),
                      url: gitLinkUrl.trim() || undefined,
                    });
                    setShowGitLinkForm(null);
                    setGitLinkRefName("");
                    setGitLinkUrl("");
                    await loadIssue();
                  }}
                  className="rounded-lg bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground hover:bg-primary/90"
                >
                  Link
                </button>
                <button
                  onClick={() => { setShowGitLinkForm(null); setGitLinkRefName(""); setGitLinkUrl(""); }}
                  className="rounded-lg px-3 py-1.5 text-xs hover:bg-muted"
                >
                  Cancel
                </button>
              </div>
            </div>
          ) : (
            <div className="flex gap-1.5">
              <button
                onClick={() => setShowGitLinkForm("branch")}
                className="flex items-center gap-1 rounded-lg border border-border/50 px-2.5 py-1.5 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
              >
                <GitBranch className="h-3 w-3" />
                Branch
              </button>
              <button
                onClick={() => setShowGitLinkForm("pr")}
                className="flex items-center gap-1 rounded-lg border border-border/50 px-2.5 py-1.5 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
              >
                <GitPullRequest className="h-3 w-3" />
                PR
              </button>
              <button
                onClick={() => setShowGitLinkForm("commit")}
                className="flex items-center gap-1 rounded-lg border border-border/50 px-2.5 py-1.5 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
              >
                <GitCommitHorizontal className="h-3 w-3" />
                Commit
              </button>
            </div>
          )}
        </div>

        {/* Code Context — commits and branches that reference this issue */}
        {(issueCommits.length > 0 || issueBranches.length > 0) && (
          <div className="mt-6 border-t border-border/50 px-5 pt-4">
            <h3 className="mb-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">
              Code Context
            </h3>
            {issueBranches.length > 0 && (
              <div className="mb-3 space-y-0.5">
                {issueBranches.map(branch => (
                  <div key={branch.name} className="flex items-center gap-2 rounded-lg px-2.5 py-1.5 hover:bg-muted transition-colors">
                    <GitBranch className="h-3.5 w-3.5 text-muted-foreground/60 flex-shrink-0" />
                    <span className="font-mono text-xs truncate flex-1 text-foreground/80">{branch.name}</span>
                    <span className="text-[11px] text-muted-foreground/40 truncate max-w-[160px]">{branch.last_commit_message}</span>
                  </div>
                ))}
              </div>
            )}
            {issueCommits.length > 0 && (
              <div className="space-y-0.5">
                {issueCommits.map(commit => (
                  <div key={commit.hash} className="flex items-center gap-2 rounded-lg px-2.5 py-1.5 hover:bg-muted transition-colors">
                    <GitCommitHorizontal className="h-3.5 w-3.5 text-muted-foreground/60 flex-shrink-0" />
                    <span className="font-mono text-[11px] text-muted-foreground/50 flex-shrink-0">{commit.short_hash}</span>
                    <span className="text-xs truncate flex-1 text-foreground/80">{commit.message}</span>
                    <span className="text-[11px] text-muted-foreground/40 flex-shrink-0">{commit.author}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Decompose preview */}
        {decomposeTasks && (
          <div className="mt-3 mx-5 rounded-lg border border-border/50 bg-muted/30 p-3">
            <h4 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">
              Decomposition Preview ({decomposeTasks.length} sub-tasks)
            </h4>
            <div className="space-y-1.5 mb-3">
              {decomposeTasks.map((t, i) => (
                <div key={t.title} className="text-xs text-foreground/80 flex items-start gap-1.5">
                  <span className="text-muted-foreground/50 font-mono">{i + 1}.</span>
                  <span>{t.title}</span>
                </div>
              ))}
            </div>
            <div className="flex gap-2">
              <button
                onClick={async () => {
                  try {
                    await api.applyDecomposition(issueId);
                    setDecomposeTasks(null);
                    await loadIssue();
                  } catch (e) {
                    console.error("Apply decomposition failed", e);
                  }
                }}
                className="rounded-md bg-primary px-2.5 py-1 text-xs text-primary-foreground hover:bg-primary/90 transition-colors"
              >
                Create Sub-issues
              </button>
              <button
                onClick={() => setDecomposeTasks(null)}
                className="rounded-md px-2.5 py-1 text-xs text-muted-foreground hover:bg-muted transition-colors"
              >
                Cancel
              </button>
            </div>
          </div>
        )}

        {/* Sub-issues */}
        <div className="mt-6 border-t border-border/50 px-5 pt-4">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">
              Sub-issues{subIssues.length > 0 && ` (${subIssues.filter(s => {
                const st = statuses.find(st2 => st2.id === s.status_id);
                return st?.category === "completed" || st?.category === "discarded";
              }).length}/${subIssues.length})`}
            </h3>
            {onCreateSubIssue && (
              <button
                onClick={() => onCreateSubIssue(issueId)}
                className="flex items-center gap-1 rounded-md px-2 py-1 text-[11px] text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
                title="Create sub-issue"
              >
                <Plus className="h-3 w-3" />
                Add sub-issue
              </button>
            )}
          </div>
          {subIssues.length > 0 && (
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
          )}
        </div>

        {/* Linked Files */}
        <div className="mt-6 border-t border-border/50 px-5 pt-4">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">
              Code ({fileLinks.length})
            </h3>
            <button
              onClick={() => setShowAddFile(!showAddFile)}
              className="rounded-md p-1 hover:bg-muted transition-colors"
              title="Link a file"
            >
              <Link2 className="h-3.5 w-3.5 text-muted-foreground/50" />
            </button>
          </div>

          {showAddFile && (
            <div className="mb-3 flex flex-col gap-1.5">
              <input
                autoFocus
                type="text"
                value={newFilePath}
                onChange={(e) => setNewFilePath(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleAddFileLink();
                  if (e.key === "Escape") setShowAddFile(false);
                }}
                placeholder="src/components/Example.tsx"
                className="w-full rounded-lg border border-border bg-background p-2 text-sm outline-none focus:border-primary/50 font-mono text-xs"
              />
              <div className="flex items-center gap-1.5">
                <select
                  value={newLinkType}
                  onChange={(e) => setNewLinkType(e.target.value as "related" | "cause" | "fix")}
                  className="rounded-lg border border-border bg-background px-2 py-1.5 text-xs outline-none"
                >
                  <option value="related">Related</option>
                  <option value="cause">Cause</option>
                  <option value="fix">Fix</option>
                </select>
                <button
                  onClick={handleAddFileLink}
                  disabled={!newFilePath.trim()}
                  className="rounded-lg bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-40"
                >
                  Link
                </button>
                <button
                  onClick={() => setShowAddFile(false)}
                  className="rounded-lg px-3 py-1.5 text-xs hover:bg-muted"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}

          {fileLinks.length > 0 ? (
            <div className="space-y-0.5">
              {fileLinks.map(link => {
                const typeColor = link.link_type === "cause" ? "text-red-400" : link.link_type === "fix" ? "text-green-400" : "text-blue-400";
                return (
                  <div key={link.id} className="group flex items-center gap-2 rounded-lg px-2.5 py-1.5 text-sm hover:bg-muted transition-colors">
                    <Code className="h-3.5 w-3.5 text-muted-foreground/50 flex-shrink-0" />
                    <span className="font-mono text-xs truncate flex-1">{link.file_path}</span>
                    <span className={cn("text-[10px] font-medium uppercase", typeColor)}>{link.link_type}</span>
                    <button
                      onClick={() => handleUnlinkFile(link.file_path)}
                      className="rounded-md p-0.5 opacity-0 group-hover:opacity-100 hover:bg-red-500/10 transition-opacity"
                    >
                      <Unlink className="h-3 w-3 text-muted-foreground/50" />
                    </button>
                  </div>
                );
              })}
            </div>
          ) : !showAddFile ? (
            <p className="text-xs text-muted-foreground/40">No files linked</p>
          ) : null}
        </div>

        {/* Handoff Notes */}
        {handoffNotes.length > 0 && (
          <div className="border-t border-border/50 px-5 pt-4 pb-4">
            <h3 className="mb-3 flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">
              <ArrowRightLeft className="h-3.5 w-3.5" />
              Handoff Notes
            </h3>
            <div className="space-y-3">
              {handoffNotes.map(note => {
                const typeColors: Record<string, string> = {
                  completion: "bg-green-500/10 text-green-600 dark:text-green-400",
                  review_request: "bg-purple-500/10 text-purple-600 dark:text-purple-400",
                  escalation: "bg-red-500/10 text-red-600 dark:text-red-400",
                  context: "bg-blue-500/10 text-blue-600 dark:text-blue-400",
                  warning: "bg-amber-500/10 text-amber-600 dark:text-amber-400",
                  suggestion: "bg-cyan-500/10 text-cyan-600 dark:text-cyan-400",
                };
                return (
                  <div key={note.id} className="rounded-lg border border-border/50 bg-muted/20 p-3">
                    <div className="flex items-center gap-2 mb-1.5">
                      <span className={cn("rounded-full px-2 py-0.5 text-[10px] font-medium", typeColors[note.note_type] || "bg-muted text-muted-foreground")}>
                        {note.note_type.replace("_", " ")}
                      </span>
                      <span className="text-[11px] text-muted-foreground/60">
                        {note.from_agent_id} {note.to_agent_id ? `\u2192 ${note.to_agent_id}` : "\u2192 any"}
                      </span>
                      <span className="ml-auto text-[10px] text-muted-foreground/40">
                        {note.created_at.slice(0, 16).replace("T", " ")}
                      </span>
                    </div>
                    <p className="text-sm text-foreground/90">{note.summary}</p>
                    {note.details && (
                      <p className="mt-1 text-xs text-muted-foreground/70">{note.details}</p>
                    )}
                    {note.files_changed.length > 0 && (
                      <div className="mt-2 flex flex-wrap gap-1">
                        {note.files_changed.map((f) => (
                          <span key={f} className="inline-flex items-center gap-0.5 rounded bg-muted px-1.5 py-0.5 text-[10px] font-mono text-muted-foreground">
                            <FileText className="h-2.5 w-2.5" />{f}
                          </span>
                        ))}
                      </div>
                    )}
                    {note.risks.length > 0 && (
                      <div className="mt-2 flex flex-wrap gap-1">
                        {note.risks.map((r) => (
                          <span key={r} className="inline-flex items-center gap-0.5 rounded bg-red-500/10 px-1.5 py-0.5 text-[10px] text-red-600 dark:text-red-400">
                            <AlertCircle className="h-2.5 w-2.5" />{r}
                          </span>
                        ))}
                      </div>
                    )}
                    {note.test_results && (
                      <div className="mt-2 flex gap-3 text-[10px]">
                        {note.test_results.passed != null && <span className="text-green-600 dark:text-green-400">{note.test_results.passed} passed</span>}
                        {note.test_results.failed != null && <span className="text-red-600 dark:text-red-400">{note.test_results.failed} failed</span>}
                        {note.test_results.skipped != null && <span className="text-muted-foreground/60">{note.test_results.skipped} skipped</span>}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* Learnings */}
        {taskLearnings.length > 0 && (
          <div className="border-t border-border/50 px-5 pt-4 pb-4">
            <h3 className="mb-3 flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">
              <Lightbulb className="h-3.5 w-3.5" />
              Learnings
            </h3>
            <div className="space-y-3">
              {taskLearnings.map(learning => {
                const outcomeColors: Record<string, string> = {
                  success: "bg-green-500/10 text-green-600 dark:text-green-400",
                  failure: "bg-red-500/10 text-red-600 dark:text-red-400",
                  partial: "bg-amber-500/10 text-amber-600 dark:text-amber-400",
                };
                return (
                  <div key={learning.id} className="rounded-lg border border-border/50 bg-muted/20 p-3">
                    <div className="flex items-center gap-2 mb-1.5">
                      <span className={cn("rounded-full px-2 py-0.5 text-[10px] font-medium", outcomeColors[learning.outcome] || "bg-muted text-muted-foreground")}>
                        {learning.outcome}
                      </span>
                      <span className="text-[11px] text-muted-foreground/60">by {learning.agent_id}</span>
                      <span className="ml-auto text-[10px] text-muted-foreground/40">
                        {learning.created_at.slice(0, 16).replace("T", " ")}
                      </span>
                    </div>
                    <p className="text-sm text-foreground/90">{learning.approach_summary}</p>
                    {learning.key_insight && (
                      <p className="mt-1.5 flex items-start gap-1 text-xs text-primary/80">
                        <Lightbulb className="h-3 w-3 mt-0.5 shrink-0" />
                        {learning.key_insight}
                      </p>
                    )}
                    {learning.pitfalls.length > 0 && (
                      <div className="mt-2">
                        <span className="text-[10px] font-medium text-red-500/80">Pitfalls:</span>
                        <ul className="mt-0.5 space-y-0.5">
                          {learning.pitfalls.map((p) => (
                            <li key={p} className="text-[11px] text-muted-foreground/70 pl-2">&bull; {p}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {learning.effective_patterns.length > 0 && (
                      <div className="mt-2">
                        <span className="text-[10px] font-medium text-green-500/80">What worked:</span>
                        <ul className="mt-0.5 space-y-0.5">
                          {learning.effective_patterns.map((p) => (
                            <li key={p} className="text-[11px] text-muted-foreground/70 pl-2">&bull; {p}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {learning.tags.length > 0 && (
                      <div className="mt-2 flex flex-wrap gap-1">
                        {learning.tags.map((tag) => (
                          <span key={tag} className="rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
                            {tag}
                          </span>
                        ))}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* Tabs: Comments / History / Activity */}
        <div className="mt-6 border-t border-border/50">
          <div className="flex items-center gap-0 px-5 border-b border-border/50">
            <button
              onClick={() => setActiveTab("comments")}
              className={cn(
                "flex items-center gap-1.5 px-3 py-2.5 text-xs font-medium border-b-2 -mb-px transition-colors",
                activeTab === "comments"
                  ? "border-primary text-primary"
                  : "border-transparent text-muted-foreground/60 hover:text-muted-foreground"
              )}
            >
              <MessageSquare className="h-3 w-3" />
              Comments ({comments.length})
            </button>
            <button
              onClick={() => setActiveTab("history")}
              className={cn(
                "flex items-center gap-1.5 px-3 py-2.5 text-xs font-medium border-b-2 -mb-px transition-colors",
                activeTab === "history"
                  ? "border-primary text-primary"
                  : "border-transparent text-muted-foreground/60 hover:text-muted-foreground"
              )}
            >
              <History className="h-3 w-3" />
              History
            </button>
            <button
              onClick={() => setActiveTab("activity")}
              className={cn(
                "flex items-center gap-1.5 px-3 py-2.5 text-xs font-medium border-b-2 -mb-px transition-colors",
                activeTab === "activity"
                  ? "border-primary text-primary"
                  : "border-transparent text-muted-foreground/60 hover:text-muted-foreground"
              )}
            >
              <Activity className="h-3 w-3" />
              Activity
            </button>
          </div>

          {/* Comments tab */}
          {activeTab === "comments" && (
            <div className="px-5 pt-4">
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
                              <MentionInput
                                autoFocus
                                value={editingCommentContent}
                                onChange={setEditingCommentContent}
                                rows={3}
                                members={members}
                                className="border-border bg-background"
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
                              <MentionText text={comment.content} members={members} />
                            </div>
                          )}
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>

          {commentError && (
            <div className="mt-2 rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-xs text-red-500">
              {commentError}
            </div>
          )}

              {/* Add comment */}
              <div className="mt-4 pb-4">
                <MentionInput
                  value={newComment}
                  onChange={setNewComment}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
                      e.preventDefault();
                      handleAddComment();
                    }
                  }}
                  rows={3}
                  placeholder="Leave a comment... (@ to mention, Cmd+Enter to submit)"
                  members={members}
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
          )}

          {/* History tab */}
          {activeTab === "history" && (
            <div className="px-5 pt-4 pb-4">
              <IssueHistoryPanel
                issueId={issueId}
                statuses={statuses}
                members={members}
                createdAt={issue.created_at}
              />
            </div>
          )}

          {/* Activity tab */}
          {activeTab === "activity" && (
            <div className="px-5 pt-4 pb-4">
              {activity.length === 0 ? (
                <div className="py-4 text-center text-xs text-muted-foreground/40">No activity yet</div>
              ) : (
                <div className="space-y-2">
                  {activity.slice(0, activityLimit).map(entry => (
                    <div key={entry.id} className="flex items-start gap-2 text-xs text-muted-foreground/60">
                      <span className="shrink-0 font-mono">{entry.timestamp.slice(0, 16)}</span>
                      <span>
                        Changed <span className="text-foreground/80">{entry.field_changed}</span>
                        {entry.old_value && <> from <span className="text-foreground/80">{typeof entry.old_value === "object" ? JSON.stringify(entry.old_value) : entry.old_value}</span></>}
                        {entry.new_value && <> to <span className="text-foreground/80">{typeof entry.new_value === "object" ? JSON.stringify(entry.new_value) : entry.new_value}</span></>}
                      </span>
                    </div>
                  ))}
                  {activity.length > activityLimit && (
                    <button
                      onClick={() => setActivityLimit(prev => prev + 50)}
                      className="mt-2 text-xs text-primary hover:text-primary/80 transition-colors"
                    >
                      Load more activity ({activity.length - activityLimit} remaining)
                    </button>
                  )}
                </div>
              )}
            </div>
          )}
        </div>
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
    </div>
  );
}
