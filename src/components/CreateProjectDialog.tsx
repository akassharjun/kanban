import { useState } from "react";
import { X } from "lucide-react";

interface CreateProjectDialogProps {
  onClose: () => void;
  onCreate: (input: { name: string; description?: string; icon?: string; prefix: string }) => Promise<unknown>;
}

export function CreateProjectDialog({ onClose, onCreate }: CreateProjectDialogProps) {
  const [name, setName] = useState("");
  const [prefix, setPrefix] = useState("");
  const [description, setDescription] = useState("");
  const [icon, setIcon] = useState("📋");

  const handleSubmit = async () => {
    if (!name.trim() || !prefix.trim()) return;
    await onCreate({ name: name.trim(), prefix: prefix.trim().toUpperCase(), description: description || undefined, icon });
    onClose();
  };

  const handleNameChange = (val: string) => {
    setName(val);
    if (!prefix) {
      setPrefix(val.slice(0, 3).toUpperCase());
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={onClose}>
      <div className="w-[420px] rounded-lg border border-border bg-card p-6 shadow-xl" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold">New Project</h2>
          <button onClick={onClose} className="rounded p-1 hover:bg-accent"><X className="h-4 w-4" /></button>
        </div>

        <div className="space-y-4">
          <div>
            <label className="block text-sm text-muted-foreground mb-1">Name</label>
            <input
              autoFocus
              value={name}
              onChange={(e) => handleNameChange(e.target.value)}
              className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
              placeholder="My Project"
            />
          </div>
          <div>
            <label className="block text-sm text-muted-foreground mb-1">Prefix (3 letters, used in issue IDs like PRJ-42)</label>
            <input
              value={prefix}
              onChange={(e) => setPrefix(e.target.value.toUpperCase().slice(0, 5))}
              className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
              placeholder="PRJ"
              maxLength={5}
            />
          </div>
          <div>
            <label className="block text-sm text-muted-foreground mb-1">Description (optional)</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
              className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
              placeholder="What is this project about?"
            />
          </div>
          <div>
            <label className="block text-sm text-muted-foreground mb-1">Icon</label>
            <input
              value={icon}
              onChange={(e) => setIcon(e.target.value)}
              className="w-20 rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary text-center"
            />
          </div>
        </div>

        <div className="mt-6 flex justify-end gap-2">
          <button onClick={onClose} className="rounded-md px-4 py-2 text-sm hover:bg-accent">Cancel</button>
          <button
            onClick={handleSubmit}
            disabled={!name.trim() || !prefix.trim()}
            className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
          >
            Create
          </button>
        </div>
      </div>
    </div>
  );
}
