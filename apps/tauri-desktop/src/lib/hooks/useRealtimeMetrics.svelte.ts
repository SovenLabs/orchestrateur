import { connectionStore } from "$lib/stores/connection.svelte";

/** Métriques temps réel client + serveur. */
export function useRealtimeMetrics() {
  return {
    get clientEventRate() {
      return connectionStore.eventRate;
    },
    get wsLatencyMs() {
      return connectionStore.latencyMs;
    },
    get queuedMessages() {
      return connectionStore.queuedMessages;
    },
    get pendingRequests() {
      return connectionStore.pendingRequests;
    },
    get serverMetrics() {
      return connectionStore.serverMetrics;
    },
    get connectedClients() {
      return connectionStore.serverHealth?.connected_clients ?? 0;
    },
    refreshServerHealth: () => connectionStore.pollServerHealth(),
  };
}