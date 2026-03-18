import ReactDiffViewer, { DiffMethod } from "react-diff-viewer-continued";
import type { ExecutionLog } from "@/types";

interface DiffPreviewProps {
  logs: ExecutionLog[];
}

export function DiffPreview({ logs }: DiffPreviewProps) {
  const edits = logs.filter((l) => l.entry_type === "file_edit" && l.metadata);

  if (edits.length === 0) {
    return (
      <div className="p-4 text-sm text-muted-foreground">
        No file changes in this attempt.
      </div>
    );
  }

  return (
    <div className="divide-y divide-border">
      {edits.map((edit) => {
        let meta: { file?: string; old_content?: string; new_content?: string } = {};
        try {
          meta = JSON.parse(edit.metadata || "{}");
        } catch {
          /* ignore */
        }

        return (
          <div key={edit.id} className="p-3">
            <div className="text-xs font-mono text-muted-foreground mb-2">
              {meta.file || "Unknown file"}
            </div>
            <div className="rounded overflow-hidden text-xs">
              <ReactDiffViewer
                oldValue={meta.old_content || ""}
                newValue={meta.new_content || ""}
                splitView={false}
                useDarkTheme
                compareMethod={DiffMethod.LINES}
                styles={{
                  variables: {
                    dark: {
                      diffViewerBackground: "#18181b",
                      addedBackground: "#052e16",
                      removedBackground: "#450a0a",
                      addedColor: "#4ade80",
                      removedColor: "#f87171",
                      wordAddedBackground: "#065f46",
                      wordRemovedBackground: "#7f1d1d",
                    },
                  },
                }}
              />
            </div>
          </div>
        );
      })}
    </div>
  );
}
