import {
  applyHarnessOnboard,
  getHarnessWorkspaceInfo,
  listHarnessChannels,
  probeHarnessServices,
  saveHarnessChannel,
  type ChannelRow,
  type HarnessServiceProbe,
  type OnboardRequest,
} from "$lib/tauri/harness";

const SETUP_DISMISSED_KEY = "orchestrateur_setup_complete";

class HarnessStore {
  setupOpen = $state(false);
  messagingOpen = $state(false);

  workspacePath = $state("");
  configExists = $state(false);

  bootstrapPercent = $state(0);
  bootstrapStatus = $state("En attente…");
  bootstrapping = $state(false);

  services = $state<HarnessServiceProbe | null>(null);
  channels = $state<ChannelRow[]>([]);
  selectedChannelId = $state<string | null>(null);
  loadingChannels = $state(false);
  saveMessage = $state<string | null>(null);

  selectedChannel = $derived(
    this.channels.find((c) => c.id === this.selectedChannelId) ?? null,
  );

  async init(): Promise<void> {
    const info = await getHarnessWorkspaceInfo();
    if (!info) return;
    this.workspacePath = info.path;
    this.configExists = info.config_exists;
    const dismissed = localStorage.getItem(SETUP_DISMISSED_KEY) === "1";
    this.setupOpen = !info.config_exists || !dismissed;
    await this.refreshServices();
    if (!this.setupOpen) {
      await this.refreshChannels();
    }
  }

  async refreshServices(): Promise<void> {
    this.services = await probeHarnessServices();
  }

  async refreshChannels(): Promise<void> {
    this.loadingChannels = true;
    try {
      this.channels = await listHarnessChannels();
      if (!this.selectedChannelId && this.channels.length > 0) {
        this.selectedChannelId = this.channels[0]?.id ?? null;
      }
    } finally {
      this.loadingChannels = false;
    }
  }

  openMessaging(): void {
    this.messagingOpen = true;
    void this.refreshChannels();
    void this.refreshServices();
  }

  closeMessaging(): void {
    this.messagingOpen = false;
  }

  openSetup(): void {
    this.setupOpen = true;
  }

  dismissSetup(): void {
    this.setupOpen = false;
    localStorage.setItem(SETUP_DISMISSED_KEY, "1");
  }

  async runBootstrap(): Promise<void> {
    this.bootstrapping = true;
    this.bootstrapPercent = 8;
    this.bootstrapStatus = "Vérification configuration…";
    await new Promise((r) => setTimeout(r, 400));

    for (let i = 0; i < 24; i++) {
      this.bootstrapPercent = Math.min(24 + i * 2, 55);
      this.bootstrapStatus = "Attente du daemon harness…";
      await this.refreshServices();
      if (this.services?.daemon === "alive") break;
      await new Promise((r) => setTimeout(r, 750));
    }

    for (let i = 0; i < 24; i++) {
      this.bootstrapPercent = Math.min(56 + i * 2, 95);
      this.bootstrapStatus = "Attente du gateway messaging…";
      await this.refreshServices();
      if (this.services?.gateway === "alive") break;
      await new Promise((r) => setTimeout(r, 750));
    }

    this.bootstrapPercent = 100;
    const ok =
      this.services?.daemon === "alive" && this.services?.gateway === "alive";
    this.bootstrapStatus = ok
      ? "Harness prêt"
      : "Services partiels — lancez orchestrateur harness run";
    this.bootstrapping = false;
  }

  async completeOnboard(req: OnboardRequest): Promise<void> {
    await applyHarnessOnboard(req);
    this.configExists = true;
    await this.runBootstrap();
    localStorage.setItem(SETUP_DISMISSED_KEY, "1");
    this.setupOpen = false;
    await this.refreshChannels();
  }

  async saveChannel(
    channelId: string,
    token: string,
    allowedIds: string,
    enabled: boolean,
  ): Promise<void> {
    this.saveMessage = null;
    try {
      await saveHarnessChannel({
        channel_id: channelId,
        token: token.trim() || undefined,
        allowed_ids: allowedIds.trim() || undefined,
        enabled,
      });
      await this.refreshChannels();
      this.saveMessage = "Enregistré";
    } catch (e) {
      this.saveMessage = e instanceof Error ? e.message : String(e);
    }
  }
}

export const harnessStore = new HarnessStore();