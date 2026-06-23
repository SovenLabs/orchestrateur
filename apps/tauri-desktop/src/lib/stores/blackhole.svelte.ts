import {
  blackholeStateFromScroll,
  computeNuanceDepth,
  isThinking,
  type BlackholeState,
} from "$lib/cosmic/nuance";
import type { OrbitalNodeKind } from "$lib/cosmic/orbital-nodes";

export type OrbitalHit = {
  id: string;
  x: number;
  y: number;
  r: number;
  kind: OrbitalNodeKind;
  label: string;
};
import { connectionStore } from "$lib/stores/connection.svelte";

class BlackholeStore {
  state = $state<BlackholeState>("expanded");
  scrollAnchor = $state<{ top: number; height: number; client: number } | null>(null);
  feedBurstNonce = $state(0);
  orbitalHits = $state<OrbitalHit[]>([]);

  nuanceDepth = $derived(
    computeNuanceDepth(connectionStore.chatMessages, connectionStore.eventLog, {
      total: connectionStore.memoryTotal,
      linkTotal:
        connectionStore.memories.length > 0
          ? connectionStore.memories.reduce((sum, m) => sum + m.backlink_count, 0)
          : connectionStore.memoryTotal * 4,
    }),
  );

  thinking = $derived(isThinking(connectionStore.chatPending, connectionStore.agentActivity));

  updateFromScroll(scrollTop: number, scrollHeight: number, clientHeight: number): void {
    const next = blackholeStateFromScroll(scrollTop, scrollHeight, clientHeight, this.state);
    this.scrollAnchor = next.anchor;
    this.state = next.state;
  }

  expand(): void {
    this.state = "expanded";
  }

  dock(): void {
    this.state = "docked";
  }

  triggerFeedBurst(): void {
    this.feedBurstNonce += 1;
  }

  setOrbitalHits(hits: OrbitalHit[]): void {
    this.orbitalHits = hits;
  }
}

export const blackholeStore = new BlackholeStore();