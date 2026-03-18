import { useState, useRef, useEffect, useCallback } from "react";
import { cn } from "@/lib/utils";
import type { Member } from "@/types";
import * as api from "@/tauri/commands";

interface MentionInputProps {
  value: string;
  onChange: (value: string) => void;
  onKeyDown?: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
  placeholder?: string;
  rows?: number;
  className?: string;
  autoFocus?: boolean;
  members?: Member[];
}

export function MentionInput({
  value,
  onChange,
  onKeyDown,
  placeholder,
  rows = 3,
  className,
  autoFocus,
  members: propMembers,
}: MentionInputProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const [showDropdown, setShowDropdown] = useState(false);
  const [mentionQuery, setMentionQuery] = useState("");
  const [mentionStartPos, setMentionStartPos] = useState(0);
  const [suggestions, setSuggestions] = useState<Member[]>([]);
  const [selectedIdx, setSelectedIdx] = useState(0);

  // Fetch suggestions when query changes
  useEffect(() => {
    if (!showDropdown || !mentionQuery) {
      setSuggestions([]);
      return;
    }

    // If we have local members, filter locally
    if (propMembers && propMembers.length > 0) {
      const q = mentionQuery.toLowerCase();
      const filtered = propMembers.filter(
        (m) =>
          m.name.toLowerCase().includes(q) ||
          (m.display_name ?? "").toLowerCase().includes(q)
      ).slice(0, 10);
      setSuggestions(filtered);
      setSelectedIdx(0);
      return;
    }

    // Otherwise search via API
    const timer = setTimeout(() => {
      api.searchMembersForMention(mentionQuery)
        .then((results) => {
          setSuggestions(results);
          setSelectedIdx(0);
        })
        .catch(console.error);
    }, 100);

    return () => clearTimeout(timer);
  }, [mentionQuery, showDropdown, propMembers]);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      const newValue = e.target.value;
      onChange(newValue);

      const cursorPos = e.target.selectionStart;
      // Check if we should show mention dropdown
      // Look backwards from cursor for @
      const textBefore = newValue.slice(0, cursorPos);
      const atIdx = textBefore.lastIndexOf("@");

      if (atIdx >= 0) {
        // Check that @ is at start or preceded by whitespace
        const charBefore = atIdx > 0 ? textBefore[atIdx - 1] : " ";
        if (charBefore === " " || charBefore === "\n" || atIdx === 0) {
          const query = textBefore.slice(atIdx + 1);
          // Only show if query doesn't contain spaces (username-like)
          if (!query.includes(" ") && query.length <= 30) {
            setShowDropdown(true);
            setMentionQuery(query);
            setMentionStartPos(atIdx);
            return;
          }
        }
      }

      setShowDropdown(false);
      setMentionQuery("");
    },
    [onChange]
  );

  const insertMention = useCallback(
    (member: Member) => {
      const before = value.slice(0, mentionStartPos);
      const after = value.slice(
        mentionStartPos + mentionQuery.length + 1 // +1 for @
      );
      const newValue = `${before}@${member.name} ${after}`;
      onChange(newValue);
      setShowDropdown(false);
      setMentionQuery("");

      // Focus and set cursor
      setTimeout(() => {
        if (textareaRef.current) {
          textareaRef.current.focus();
          const newPos = mentionStartPos + member.name.length + 2; // @name + space
          textareaRef.current.setSelectionRange(newPos, newPos);
        }
      }, 0);
    },
    [value, onChange, mentionStartPos, mentionQuery]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (showDropdown && suggestions.length > 0) {
        if (e.key === "ArrowDown") {
          e.preventDefault();
          setSelectedIdx((i) => Math.min(i + 1, suggestions.length - 1));
          return;
        }
        if (e.key === "ArrowUp") {
          e.preventDefault();
          setSelectedIdx((i) => Math.max(i - 1, 0));
          return;
        }
        if (e.key === "Enter" || e.key === "Tab") {
          e.preventDefault();
          insertMention(suggestions[selectedIdx]);
          return;
        }
        if (e.key === "Escape") {
          e.preventDefault();
          setShowDropdown(false);
          return;
        }
      }

      onKeyDown?.(e);
    },
    [showDropdown, suggestions, selectedIdx, insertMention, onKeyDown]
  );

  return (
    <div className="relative">
      <textarea
        ref={textareaRef}
        autoFocus={autoFocus}
        value={value}
        onChange={handleChange}
        onKeyDown={handleKeyDown}
        rows={rows}
        placeholder={placeholder}
        className={cn(
          "w-full rounded-lg border border-border bg-background p-3 text-sm outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/20 placeholder:text-muted-foreground/40",
          className
        )}
      />

      {/* Mention autocomplete dropdown */}
      {showDropdown && suggestions.length > 0 && (
        <div
          ref={dropdownRef}
          className="absolute left-0 bottom-full mb-1 z-50 w-56 rounded-lg border border-border bg-popover p-1 shadow-lg animate-in fade-in-0 zoom-in-95 duration-100"
        >
          {suggestions.map((member, i) => (
            <button
              key={member.id}
              onMouseDown={(e) => {
                e.preventDefault(); // Prevent blur
                insertMention(member);
              }}
              className={cn(
                "flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[13px] hover:bg-muted transition-colors",
                i === selectedIdx && "bg-muted"
              )}
            >
              <div
                className="flex h-5 w-5 items-center justify-center rounded-full text-[9px] font-semibold text-white"
                style={{ backgroundColor: member.avatar_color }}
              >
                {(member.display_name || member.name).charAt(0).toUpperCase()}
              </div>
              <div className="flex flex-col items-start">
                <span className="font-medium">{member.display_name || member.name}</span>
                <span className="text-[11px] text-muted-foreground/50">@{member.name}</span>
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

/** Renders text with @mentions styled as inline badges */
export function MentionText({ text, members }: { text: string; members?: Member[] }) {
  // Parse @username patterns
  const parts: { type: "text" | "mention"; value: string; member?: Member }[] = [];
  const mentionRegex = /@([a-zA-Z0-9_-]+)/g;
  let lastIdx = 0;
  let match;

  while ((match = mentionRegex.exec(text)) !== null) {
    if (match.index > lastIdx) {
      parts.push({ type: "text", value: text.slice(lastIdx, match.index) });
    }
    const username = match[1];
    const member = members?.find(
      (m) => m.name.toLowerCase() === username.toLowerCase()
    );
    parts.push({ type: "mention", value: `@${username}`, member });
    lastIdx = match.index + match[0].length;
  }
  if (lastIdx < text.length) {
    parts.push({ type: "text", value: text.slice(lastIdx) });
  }

  return (
    <>
      {parts.map((part, i) => {
        if (part.type === "mention") {
          return (
            <span
              key={i}
              className={cn(
                "inline-flex items-center rounded px-1 py-0.5 text-xs font-medium",
                part.member
                  ? "bg-primary/10 text-primary"
                  : "bg-muted text-muted-foreground"
              )}
              title={part.member ? (part.member.display_name || part.member.name) : part.value}
            >
              {part.value}
            </span>
          );
        }
        return <span key={i}>{part.value}</span>;
      })}
    </>
  );
}
