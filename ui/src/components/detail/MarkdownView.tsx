import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { open } from "@tauri-apps/plugin-shell";

/**
 * Render markdown (GFM) with links routed to the system browser via the Tauri
 * shell plugin instead of navigating inside the WebView.
 */
export function MarkdownView({ source }: { source: string }) {
  return (
    <div className="prose-sm max-w-none">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          a: ({ href, children }) => (
            <a
              href={href}
              onClick={(e) => {
                if (href) {
                  e.preventDefault();
                  void open(href);
                }
              }}
              className="underline decoration-dotted underline-offset-2"
            >
              {children}
            </a>
          ),
        }}
      >
        {source}
      </ReactMarkdown>
    </div>
  );
}
