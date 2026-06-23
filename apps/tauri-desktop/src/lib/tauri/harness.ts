import { invoke } from "@tauri-apps/api/core";

export type HarnessWorkspaceInfo = {
  path: string;
  config_exists: boolean;
};

export type HarnessServiceProbe = {
  daemon: string;
  gateway: string;
  daemon_url: string;
  gateway_url: string;
};

export type ChannelRow = {
  id: string;
  display_name: string;
  enabled: boolean;
  token_env: string;
  token_set: boolean;
  dedicated: boolean;
  badges: string[];
  setup_url: string;
  setup_hint: string;
};

export type OnboardRequest = {
  profile: string;
  llm: string;
  install_daemon: boolean;
};

export type SaveChannelRequest = {
  channel_id: string;
  token?: string;
  allowed_ids?: string;
  enabled: boolean;
};

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function getHarnessWorkspaceInfo(): Promise<HarnessWorkspaceInfo | null> {
  if (!isTauriRuntime()) return null;
  return invoke<HarnessWorkspaceInfo>("harness_workspace_info");
}

export async function probeHarnessServices(): Promise<HarnessServiceProbe | null> {
  if (!isTauriRuntime()) return null;
  return invoke<HarnessServiceProbe>("harness_probe_services");
}

export async function listHarnessChannels(): Promise<ChannelRow[]> {
  if (!isTauriRuntime()) return [];
  return invoke<ChannelRow[]>("harness_list_channels");
}

export async function saveHarnessChannel(req: SaveChannelRequest): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("harness_save_channel", { req });
}

export async function applyHarnessOnboard(req: OnboardRequest): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("harness_apply_onboard", { req });
}