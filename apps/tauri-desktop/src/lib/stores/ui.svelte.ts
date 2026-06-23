import {
  getTerritoryLaunchStatus,
  launchSphereWindow,
  launchTerritoryWindow,
  type LaunchResult,
} from "$lib/tauri/territory";
import { navigationStore } from "$lib/stores/navigation.svelte";

export type SphereLaunchState = "idle" | "launching" | "running" | "error";

class UiStore {
  commandPaletteOpen = $state(false);
  sphereLaunchState = $state<SphereLaunchState>("idle");
  sphereLaunchMessage = $state<string | null>(null);
  territoryLaunchState = $state<SphereLaunchState>("idle");
  territoryLaunchMessage = $state<string | null>(null);

  /** @deprecated Phase 27 — utilise EscMenu via navigationStore */
  openCommandPalette(): void {
    this.commandPaletteOpen = false;
    navigationStore.openEscMenu();
  }

  closeCommandPalette(): void {
    this.commandPaletteOpen = false;
    navigationStore.closeEscMenu();
  }

  toggleCommandPalette(): void {
    navigationStore.toggleEscMenu();
  }

  async openSphere(): Promise<LaunchResult> {
    this.sphereLaunchState = "launching";
    this.sphereLaunchMessage = null;
    const result = await launchSphereWindow();
    if (result.ok) {
      this.sphereLaunchState = "running";
      this.sphereLaunchMessage = result.message;
    } else {
      this.sphereLaunchState = "error";
      this.sphereLaunchMessage = result.message;
    }
    return result;
  }

  async openTerritory(): Promise<LaunchResult> {
    this.territoryLaunchState = "launching";
    this.territoryLaunchMessage = null;
    const result = await launchTerritoryWindow();
    if (result.ok) {
      this.territoryLaunchState = "running";
      this.territoryLaunchMessage = result.message;
    } else {
      this.territoryLaunchState = "error";
      this.territoryLaunchMessage = result.message;
    }
    return result;
  }

  async refreshLaunchStatus(): Promise<void> {
    const status = await getTerritoryLaunchStatus();
    this.sphereLaunchState = status.sphere_running ? "running" : "idle";
    this.territoryLaunchState = status.territory_running ? "running" : "idle";
    if (status.last_error && this.sphereLaunchState === "idle") {
      this.sphereLaunchMessage = status.last_error;
    }
  }
}

export const uiStore = new UiStore();