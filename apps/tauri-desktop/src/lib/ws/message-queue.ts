/** File d'attente des messages WS pendant déconnexion. */
export class OutboundMessageQueue {
  private queue: string[] = [];
  private readonly maxSize: number;

  constructor(maxSize = 64) {
    this.maxSize = maxSize;
  }

  enqueue(payload: string): void {
    if (this.queue.length >= this.maxSize) {
      this.queue.shift();
    }
    this.queue.push(payload);
  }

  drain(): string[] {
    const items = [...this.queue];
    this.queue = [];
    return items;
  }

  get size(): number {
    return this.queue.length;
  }
}