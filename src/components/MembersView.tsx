import { useState } from "react";
import { Plus, Pencil, Trash2, X } from "lucide-react";
import type { Member } from "@/types";

interface MembersViewProps {
  members: Member[];
  onCreate: (input: { name: string; display_name?: string; email?: string; avatar_color?: string }) => Promise<unknown>;
  onUpdate: (id: number, input: { name?: string; display_name?: string; email?: string; avatar_color?: string }) => Promise<unknown>;
  onDelete: (id: number) => Promise<void>;
}

const avatarColors = ["#6366f1", "#ec4899", "#f59e0b", "#10b981", "#3b82f6", "#8b5cf6", "#ef4444", "#14b8a6", "#f97316", "#84cc16"];

export function MembersView({ members, onCreate, onUpdate, onDelete }: MembersViewProps) {
  const [showAdd, setShowAdd] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [name, setName] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");
  const [color, setColor] = useState(avatarColors[0]);

  const handleAdd = async () => {
    if (!name.trim()) return;
    await onCreate({ name: name.trim(), display_name: displayName || undefined, email: email || undefined, avatar_color: color });
    setName(""); setDisplayName(""); setEmail(""); setShowAdd(false);
  };

  const startEdit = (member: Member) => {
    setEditingId(member.id);
    setName(member.name);
    setDisplayName(member.display_name || "");
    setEmail(member.email || "");
    setColor(member.avatar_color);
  };

  const handleSaveEdit = async () => {
    if (!editingId || !name.trim()) return;
    await onUpdate(editingId, { name: name.trim(), display_name: displayName || undefined, email: email || undefined, avatar_color: color });
    setEditingId(null); setName(""); setDisplayName(""); setEmail("");
  };

  return (
    <div className="flex-1 overflow-auto p-6">
      <div className="max-w-2xl">
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-xl font-semibold">Team Members</h1>
          <button
            onClick={() => { setShowAdd(true); setEditingId(null); setName(""); setDisplayName(""); setEmail(""); }}
            className="flex items-center gap-1 rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90"
          >
            <Plus className="h-4 w-4" /> Add Member
          </button>
        </div>

        {(showAdd || editingId !== null) && (
          <div className="mb-6 rounded-lg border border-border bg-card p-4 space-y-3">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium">{editingId ? "Edit Member" : "New Member"}</h3>
              <button onClick={() => { setShowAdd(false); setEditingId(null); }} className="rounded p-1 hover:bg-accent">
                <X className="h-4 w-4" />
              </button>
            </div>
            <input value={name} onChange={e => setName(e.target.value)} placeholder="Name" className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary" />
            <input value={displayName} onChange={e => setDisplayName(e.target.value)} placeholder="Display Name (optional)" className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary" />
            <input value={email} onChange={e => setEmail(e.target.value)} placeholder="Email (optional)" className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary" />
            <div className="flex gap-1">
              {avatarColors.map(c => (
                <button key={c} onClick={() => setColor(c)} className={`h-6 w-6 rounded-full border-2 ${color === c ? "border-white" : "border-transparent"}`} style={{ backgroundColor: c }} />
              ))}
            </div>
            <div className="flex justify-end gap-2">
              <button onClick={() => { setShowAdd(false); setEditingId(null); }} className="rounded-md px-3 py-1.5 text-sm hover:bg-accent">Cancel</button>
              <button onClick={editingId ? handleSaveEdit : handleAdd} className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90">
                {editingId ? "Save" : "Add"}
              </button>
            </div>
          </div>
        )}

        <div className="space-y-2">
          {members.map(member => (
            <div key={member.id} className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
              <div className="flex items-center gap-3">
                <div
                  className="flex h-8 w-8 items-center justify-center rounded-full text-sm font-medium text-white"
                  style={{ backgroundColor: member.avatar_color }}
                >
                  {(member.display_name || member.name).charAt(0).toUpperCase()}
                </div>
                <div>
                  <div className="text-sm font-medium">{member.display_name || member.name}</div>
                  {member.email && <div className="text-xs text-muted-foreground">{member.email}</div>}
                </div>
              </div>
              <div className="flex items-center gap-1">
                <button onClick={() => startEdit(member)} className="rounded p-1.5 hover:bg-accent"><Pencil className="h-3.5 w-3.5 text-muted-foreground" /></button>
                <button onClick={() => onDelete(member.id)} className="rounded p-1.5 hover:bg-destructive/20"><Trash2 className="h-3.5 w-3.5 text-muted-foreground" /></button>
              </div>
            </div>
          ))}
          {members.length === 0 && (
            <div className="py-12 text-center text-muted-foreground">No team members yet. Add one to get started.</div>
          )}
        </div>
      </div>
    </div>
  );
}
