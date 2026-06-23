import type { PanelId } from "$lib/types/ui";

class NavigationStore {
  activePanel = $state<PanelId>("chat");
  leftDrawerOpen = $state(false);
  insightsPanelOpen = $state(false);
  escMenuOpen = $state(false);

  navigate(panel: PanelId): void {
    this.activePanel = panel;
  }

  toggleLeftDrawer(): void {
    this.leftDrawerOpen = !this.leftDrawerOpen;
    if (this.leftDrawerOpen) this.insightsPanelOpen = false;
  }

  toggleInsightsPanel(): void {
    this.insightsPanelOpen = !this.insightsPanelOpen;
    if (this.insightsPanelOpen) this.leftDrawerOpen = false;
  }

  openEscMenu(): void {
    this.escMenuOpen = true;
  }

  closeEscMenu(): void {
    this.escMenuOpen = false;
  }

  toggleEscMenu(): void {
    this.escMenuOpen = !this.escMenuOpen;
  }

  closeAllOverlays(): void {
    this.leftDrawerOpen = false;
    this.insightsPanelOpen = false;
    this.escMenuOpen = false;
  }
}

export const navigationStore = new NavigationStore();