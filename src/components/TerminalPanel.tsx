import { useState, useRef, useCallback, useEffect } from "react";
import { X, Minus, Maximize2, ChevronUp } from "lucide-react";
import { cn } from "@/lib/utils";
import { isTauri } from "@/tauri/mock-backend";
import * as api from "@/tauri/commands";

interface TerminalPanelProps {
  onClose: () => void;
  projectPath?: string | null;
}

export function TerminalPanel({ onClose, projectPath }: TerminalPanelProps) {
  const [height, setHeight] = useState(240);
  const [isMaximized, setIsMaximized] = useState(false);
  const [input, setInput] = useState("");
  const [cwd, setCwd] = useState<string | undefined>(projectPath ?? undefined);
  const [history, setHistory] = useState<{ type: "input" | "output" | "error"; text: string }[]>([
    { type: "output", text: "Terminal ready. Type commands below." },
    { type: "output", text: isTauri ? (projectPath ? `Working directory: ${projectPath}` : "Note: Set a project path in settings to start in project root.") : "Note: In browser mode, commands are simulated." },
  ]);
  const scrollRef = useRef<HTMLDivElement>(null);
  const dragRef = useRef<{ startY: number; startHeight: number } | null>(null);

  const prevHeight = useRef(height);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [history.length]);

  const handleDragStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    dragRef.current = { startY: e.clientY, startHeight: height };
    const handleMove = (ev: MouseEvent) => {
      if (!dragRef.current) return;
      const delta = dragRef.current.startY - ev.clientY;
      const newH = Math.max(120, Math.min(window.innerHeight - 100, dragRef.current.startHeight + delta));
      setHeight(newH);
    };
    const handleUp = () => {
      dragRef.current = null;
      window.removeEventListener("mousemove", handleMove);
      window.removeEventListener("mouseup", handleUp);
    };
    window.addEventListener("mousemove", handleMove);
    window.addEventListener("mouseup", handleUp);
  }, [height]);

  const toggleMaximize = () => {
    if (isMaximized) {
      setHeight(prevHeight.current);
      setIsMaximized(false);
    } else {
      prevHeight.current = height;
      setHeight(window.innerHeight - 100);
      setIsMaximized(true);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim()) return;
    const cmd = input.trim();
    setHistory(prev => [...prev, { type: "input", text: `$ ${cmd}` }]);
    setInput("");

    // Handle clear locally always
    if (cmd === "clear") {
      setHistory([]);
      return;
    }

    if (isTauri) {
      // Handle `cd` locally — update cwd state
      if (cmd === "cd" || cmd === "cd ~") {
        const home = cwd?.split("/").slice(0, 3).join("/") ?? "/home/user";
        setCwd(home);
        return;
      }
      if (cmd.startsWith("cd ")) {
        const target = cmd.slice(3).trim();
        let newCwd: string;
        if (target.startsWith("/")) {
          newCwd = target;
        } else {
          newCwd = (cwd ?? "") + "/" + target;
        }
        // Verify it exists by attempting to list it
        try {
          await api.listDirectories(newCwd);
          setCwd(newCwd);
        } catch {
          setHistory(prev => [...prev, { type: "error", text: `cd: ${target}: No such file or directory` }]);
        }
        return;
      }

      try {
        const output = await api.executeShellCommand(cmd, cwd);
        if (output.trim()) {
          setHistory(prev => [...prev, { type: "output", text: output.trimEnd() }]);
        }
      } catch (err) {
        setHistory(prev => [...prev, { type: "error", text: String(err) }]);
      }
    } else {
      // Browser mock fallback
      if (cmd === "help") {
        setHistory(prev => [...prev, { type: "output", text: "Available commands: help, clear, pwd, ls, echo, whoami" }]);
        return;
      }
      if (cmd === "pwd") {
        setHistory(prev => [...prev, { type: "output", text: cwd ?? "/home/user/project" }]);
        return;
      }
      if (cmd === "whoami") {
        setHistory(prev => [...prev, { type: "output", text: "kanban-user" }]);
        return;
      }
      if (cmd.startsWith("echo ")) {
        setHistory(prev => [...prev, { type: "output", text: cmd.slice(5) }]);
        return;
      }
      if (cmd === "ls") {
        setHistory(prev => [...prev, { type: "output", text: "src/  e2e/  docs/  package.json  tsconfig.json" }]);
        return;
      }
      setHistory(prev => [...prev, { type: "error", text: `command not found: ${cmd.split(" ")[0]}` }]);
    }
  };

  return (
    <div style={{ height: isMaximized ? "calc(100vh - 100px)" : height }} className="flex flex-col border-t border-border bg-[#1a1a2e]">
      {/* Drag handle */}
      <div
        onMouseDown={handleDragStart}
        className="h-1 cursor-ns-resize bg-border/30 hover:bg-primary/30 transition-colors shrink-0"
      />
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-1 bg-[#16162a] shrink-0">
        <div className="flex items-center gap-2">
          <ChevronUp className="h-3 w-3 text-muted-foreground/50" />
          <span className="text-[11px] font-medium text-muted-foreground">Terminal</span>
          {cwd && (
            <span className="text-[10px] text-muted-foreground/50 font-mono">{cwd}</span>
          )}
        </div>
        <div className="flex items-center gap-1">
          <button onClick={() => setHeight(120)} className="rounded p-0.5 hover:bg-muted/20 text-muted-foreground/50 hover:text-muted-foreground transition-colors">
            <Minus className="h-3 w-3" />
          </button>
          <button onClick={toggleMaximize} className="rounded p-0.5 hover:bg-muted/20 text-muted-foreground/50 hover:text-muted-foreground transition-colors">
            <Maximize2 className="h-3 w-3" />
          </button>
          <button onClick={onClose} className="rounded p-0.5 hover:bg-muted/20 text-muted-foreground/50 hover:text-muted-foreground transition-colors">
            <X className="h-3 w-3" />
          </button>
        </div>
      </div>
      {/* Output */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto px-3 py-2 font-mono text-[12px] leading-5">
        {history.map((entry, i) => (
          <div key={i} className={cn(
            entry.type === "input" ? "text-green-400" :
            entry.type === "error" ? "text-red-400" :
            "text-muted-foreground/80"
          )}>
            {entry.text}
          </div>
        ))}
      </div>
      {/* Input */}
      <form onSubmit={handleSubmit} className="flex items-center gap-2 px-3 py-1.5 border-t border-border/20 shrink-0">
        <span className="text-green-400 text-[12px] font-mono">$</span>
        <input
          value={input}
          onChange={e => setInput(e.target.value)}
          className="flex-1 bg-transparent text-[12px] font-mono text-foreground outline-none placeholder:text-muted-foreground/30"
          placeholder="Type a command..."
          autoFocus
        />
      </form>
    </div>
  );
}
