import { useState, useEffect } from "react";
import { X, Check, Trash2 } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Notification } from "@/types";
import * as api from "@/tauri/commands";

interface NotificationsPanelProps {
  onClose: () => void;
}

export function NotificationsPanel({ onClose }: NotificationsPanelProps) {
  const [notifications, setNotifications] = useState<Notification[]>([]);

  useEffect(() => { loadNotifications(); }, []);

  const loadNotifications = async () => {
    const data = await api.listNotifications();
    setNotifications(data);
  };

  const handleMarkRead = async (id: number) => {
    await api.markNotificationRead(id);
    await loadNotifications();
  };

  const handleMarkAllRead = async () => {
    await api.markAllNotificationsRead();
    await loadNotifications();
  };

  const handleClear = async () => {
    await api.clearNotifications();
    setNotifications([]);
  };

  return (
    <div className="fixed right-0 top-0 z-50 flex h-full w-80 flex-col border-l border-border bg-card shadow-xl">
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <h2 className="text-sm font-semibold">Notifications</h2>
        <div className="flex items-center gap-1">
          <button onClick={handleMarkAllRead} className="rounded p-1 hover:bg-accent" title="Mark all read">
            <Check className="h-4 w-4 text-muted-foreground" />
          </button>
          <button onClick={handleClear} className="rounded p-1 hover:bg-accent" title="Clear all">
            <Trash2 className="h-4 w-4 text-muted-foreground" />
          </button>
          <button onClick={onClose} className="rounded p-1 hover:bg-accent">
            <X className="h-4 w-4 text-muted-foreground" />
          </button>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto">
        {notifications.length === 0 ? (
          <div className="py-12 text-center text-sm text-muted-foreground">No notifications</div>
        ) : (
          notifications.map(n => (
            <div
              key={n.id}
              onClick={() => !n.read && handleMarkRead(n.id)}
              className={cn("cursor-pointer border-b border-border/50 px-4 py-3 text-sm hover:bg-accent/30", !n.read && "bg-accent/10")}
            >
              <p className={cn(n.read && "text-muted-foreground")}>{n.message}</p>
              <p className="mt-1 text-xs text-muted-foreground">{n.created_at.slice(0, 16)}</p>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
