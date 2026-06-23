/** Calcule le délai de reconnexion (backoff exponentiel plafonné). */
export function computeReconnectDelay(
  attempt: number,
  baseMs: number,
  maxMs: number,
): number {
  const exp = baseMs * 2 ** Math.max(0, attempt);
  return Math.min(exp, maxMs);
}