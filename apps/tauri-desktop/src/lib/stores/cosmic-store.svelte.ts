import type { ZoomLevel } from "$lib/cosmic/cosmic-model";
import type { CosmicPlacement } from "$lib/cosmic/cosmic-taxonomy";

class CosmicStore {
  zoomLevel = $state<ZoomLevel>("cosmos");
  focusGalaxyId = $state<string | null>(null);
  focusStarId = $state<string | null>(null);
  focusPlanetId = $state<string | null>(null);
  placementCache = $state<Map<string, CosmicPlacement>>(new Map());

  zoomInGalaxy(galaxyId: string): void {
    this.focusGalaxyId = galaxyId;
    this.focusStarId = null;
    this.focusPlanetId = null;
    this.zoomLevel = "galaxy";
  }

  zoomInBody(starId: string, planetId?: string): void {
    this.focusStarId = starId;
    this.focusPlanetId = planetId ?? null;
    this.zoomLevel = "body";
  }

  zoomOut(): void {
    if (this.zoomLevel === "body") {
      this.zoomLevel = "galaxy";
      this.focusStarId = null;
      this.focusPlanetId = null;
      return;
    }
    if (this.zoomLevel === "galaxy") {
      this.zoomLevel = "cosmos";
      this.focusGalaxyId = null;
    }
  }

  resetZoom(): void {
    this.zoomLevel = "cosmos";
    this.focusGalaxyId = null;
    this.focusStarId = null;
    this.focusPlanetId = null;
  }

  breadcrumb(): string[] {
    const parts = ["Cosmos"];
    if (this.focusGalaxyId) parts.push(this.focusGalaxyId);
    if (this.focusStarId) parts.push(this.focusStarId.split("::").pop() ?? this.focusStarId);
    if (this.focusPlanetId) parts.push(this.focusPlanetId.split("::").pop() ?? this.focusPlanetId);
    return parts;
  }
}

export const cosmicStore = new CosmicStore();