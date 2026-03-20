import * as React from "react";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";

interface DialogOverlayProps extends React.HTMLAttributes<HTMLDivElement> {
  onClose?: () => void;
}

const DialogOverlay = React.forwardRef<HTMLDivElement, DialogOverlayProps>(
  ({ className, onClose, ...props }, ref) => {
    React.useEffect(() => {
      if (!onClose) return;
      const handler = (e: KeyboardEvent) => {
        if (e.key === "Escape") { e.stopPropagation(); onClose(); }
      };
      window.addEventListener("keydown", handler, true); // capture phase
      return () => window.removeEventListener("keydown", handler, true);
    }, [onClose]);
    return (
      <div
        ref={ref}
        className={cn(
          "fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm",
          className
        )}
        onClick={onClose}
        {...props}
      />
    );
  }
);
DialogOverlay.displayName = "DialogOverlay";

interface DialogContentProps extends React.HTMLAttributes<HTMLDivElement> {
  onClose?: () => void;
}

const DialogContent = React.forwardRef<HTMLDivElement, DialogContentProps>(
  ({ className, children, onClose, ...props }, ref) => (
    <div
      ref={ref}
      className={cn(
        "w-[520px] rounded-xl border border-border/50 bg-card p-6 shadow-2xl max-h-[85vh] overflow-y-auto animate-in fade-in-0 zoom-in-95 duration-150",
        className
      )}
      onClick={(e) => e.stopPropagation()}
      {...props}
    >
      {children}
    </div>
  )
);
DialogContent.displayName = "DialogContent";

interface DialogHeaderProps extends React.HTMLAttributes<HTMLDivElement> {
  onClose?: () => void;
}

function DialogHeader({ className, children, onClose, ...props }: DialogHeaderProps) {
  return (
    <div className={cn("flex items-center justify-between mb-5", className)} {...props}>
      <div className="flex-1">{children}</div>
      {onClose && (
        <button onClick={onClose} className="rounded-lg p-1.5 hover:bg-muted transition-colors">
          <X className="h-4 w-4 text-muted-foreground" />
        </button>
      )}
    </div>
  );
}

function DialogTitle({ className, ...props }: React.HTMLAttributes<HTMLHeadingElement>) {
  return (
    <h2
      className={cn("text-lg font-semibold leading-none tracking-tight", className)}
      {...props}
    />
  );
}

export { DialogOverlay, DialogContent, DialogHeader, DialogTitle };
