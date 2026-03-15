import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { DialogOverlay, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";

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
    <DialogOverlay onClose={onClose}>
      <DialogContent className="w-[420px]">
        <DialogHeader onClose={onClose}>
          <DialogTitle>New Project</DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          <div>
            <label className="block text-sm text-muted-foreground mb-1">Name</label>
            <Input
              autoFocus
              value={name}
              onChange={(e) => handleNameChange(e.target.value)}
              placeholder="My Project"
            />
          </div>
          <div>
            <label className="block text-sm text-muted-foreground mb-1">Prefix (3 letters, used in issue IDs like PRJ-42)</label>
            <Input
              value={prefix}
              onChange={(e) => setPrefix(e.target.value.toUpperCase().slice(0, 5))}
              placeholder="PRJ"
              maxLength={5}
            />
          </div>
          <div>
            <label className="block text-sm text-muted-foreground mb-1">Description (optional)</label>
            <Textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
              placeholder="What is this project about?"
            />
          </div>
          <div>
            <label className="block text-sm text-muted-foreground mb-1">Icon</label>
            <Input
              value={icon}
              onChange={(e) => setIcon(e.target.value)}
              className="w-20 text-center"
            />
          </div>
        </div>

        <div className="mt-6 flex justify-end gap-2">
          <Button variant="ghost" onClick={onClose}>Cancel</Button>
          <Button onClick={handleSubmit} disabled={!name.trim() || !prefix.trim()}>
            Create
          </Button>
        </div>
      </DialogContent>
    </DialogOverlay>
  );
}
