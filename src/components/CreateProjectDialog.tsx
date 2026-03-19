import { useState, useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { DialogOverlay, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import * as api from "@/tauri/commands";

interface CreateProjectDialogProps {
  onClose: () => void;
  onCreate: (input: { name: string; description?: string; icon?: string; prefix: string; path: string }) => Promise<unknown>;
}

export function CreateProjectDialog({ onClose, onCreate }: CreateProjectDialogProps) {
  const [name, setName] = useState("");
  const [prefix, setPrefix] = useState("");
  const [description, setDescription] = useState("");
  const [icon, setIcon] = useState("📋");
  const [path, setPath] = useState("");
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (!path) {
      setSuggestions([]);
      return;
    }
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      api.listDirectories(path)
        .then(dirs => setSuggestions(dirs))
        .catch(() => setSuggestions([]));
    }, 300);
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [path]);

  const handleSubmit = async () => {
    if (!name.trim() || !prefix.trim() || !path.trim()) return;
    await onCreate({ name: name.trim(), prefix: prefix.trim().toUpperCase(), description: description || undefined, icon, path: path.trim() });
    onClose();
  };

  const handleNameChange = (val: string) => {
    setName(val);
    if (!prefix) {
      setPrefix(val.slice(0, 3).toUpperCase());
    }
  };

  const handleSelectSuggestion = (dir: string) => {
    setPath(dir);
    setSuggestions([]);
    setShowSuggestions(false);
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
            <label className="block text-sm text-muted-foreground mb-1">Project Path</label>
            <div className="relative">
              <Input
                value={path}
                onChange={(e) => { setPath(e.target.value); setShowSuggestions(true); }}
                onFocus={() => setShowSuggestions(true)}
                onBlur={() => setTimeout(() => setShowSuggestions(false), 150)}
                placeholder="/path/to/your/project"
              />
              {showSuggestions && suggestions.length > 0 && (
                <div className="absolute z-50 w-full mt-1 bg-popover border border-border rounded-md shadow-lg max-h-48 overflow-y-auto">
                  {suggestions.map((dir) => (
                    <button
                      key={dir}
                      className="w-full text-left px-3 py-1.5 text-sm hover:bg-accent hover:text-accent-foreground truncate"
                      onMouseDown={() => handleSelectSuggestion(dir)}
                    >
                      {dir}
                    </button>
                  ))}
                </div>
              )}
            </div>
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
          <Button onClick={handleSubmit} disabled={!name.trim() || !prefix.trim() || !path.trim()}>
            Create
          </Button>
        </div>
      </DialogContent>
    </DialogOverlay>
  );
}
