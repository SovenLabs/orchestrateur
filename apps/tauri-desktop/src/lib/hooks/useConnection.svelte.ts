import { connectionStore } from "$lib/stores/connection.svelte";

/** Accès réactif à l'état de connexion daemon (Svelte 5 runes). */
export function useConnection() {
  return {
    get status() {
      return connectionStore.status;
    },
    get isConnected() {
      return connectionStore.status === "connected";
    },
    get isDegraded() {
      return connectionStore.status !== "connected";
    },
    get version() {
      return connectionStore.version;
    },
    get protocolVersion() {
      return connectionStore.protocolVersion;
    },
    get latencyMs() {
      return connectionStore.latencyMs;
    },
    get eventRate() {
      return connectionStore.eventRate;
    },
    get agentActivity() {
      return connectionStore.agentActivity;
    },
    ping: () => connectionStore.ping(),
    healthCheck: () => connectionStore.healthCheck(),
    fetchMemories: () => connectionStore.fetchMemories(),
  };
}