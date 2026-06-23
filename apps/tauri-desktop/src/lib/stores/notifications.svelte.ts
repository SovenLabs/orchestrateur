export type NotificationLevel = "info" | "warn" | "error";

export type AppNotification = {
  id: string;
  level: NotificationLevel;
  message: string;
  at: number;
};

class NotificationsStore {
  items = $state<AppNotification[]>([]);

  push(level: NotificationLevel, message: string): void {
    const item: AppNotification = {
      id: crypto.randomUUID(),
      level,
      message,
      at: Date.now(),
    };
    this.items = [item, ...this.items].slice(0, 24);
    if (level === "error") {
      window.setTimeout(() => this.dismiss(item.id), 12_000);
    } else {
      window.setTimeout(() => this.dismiss(item.id), 6_000);
    }
  }

  dismiss(id: string): void {
    this.items = this.items.filter((n) => n.id !== id);
  }

  clear(): void {
    this.items = [];
  }
}

export const notificationsStore = new NotificationsStore();