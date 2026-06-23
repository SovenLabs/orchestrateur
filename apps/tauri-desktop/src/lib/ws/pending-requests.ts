export type PendingRequest = {
  resolve: (response: Record<string, unknown>) => void;
  reject: (error: Error) => void;
  timeoutId: ReturnType<typeof setTimeout>;
};

/** Registre des requêtes `execute` en attente de `result`. */
export class PendingRequestRegistry {
  private pending = new Map<string, PendingRequest>();

  register(
    requestId: string,
    resolve: (response: Record<string, unknown>) => void,
    reject: (error: Error) => void,
    timeoutMs = 30_000,
  ): void {
    const timeoutId = setTimeout(() => {
      this.reject(requestId, new Error(`timeout execute ${requestId}`));
    }, timeoutMs);
    this.pending.set(requestId, { resolve, reject, timeoutId });
  }

  resolve(requestId: string, response: Record<string, unknown>): boolean {
    const entry = this.pending.get(requestId);
    if (!entry) return false;
    clearTimeout(entry.timeoutId);
    entry.resolve(response);
    this.pending.delete(requestId);
    return true;
  }

  reject(requestId: string, error: Error): boolean {
    const entry = this.pending.get(requestId);
    if (!entry) return false;
    clearTimeout(entry.timeoutId);
    entry.reject(error);
    this.pending.delete(requestId);
    return true;
  }

  rejectAll(reason: string): void {
    for (const [id, entry] of this.pending) {
      clearTimeout(entry.timeoutId);
      entry.reject(new Error(reason));
      this.pending.delete(id);
    }
  }

  get size(): number {
    return this.pending.size;
  }
}