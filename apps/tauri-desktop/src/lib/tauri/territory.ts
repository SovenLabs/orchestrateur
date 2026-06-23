import { invoke } from "@tauri-apps/api/core";

export type LaunchResult = {
  ok: boolean;
  mode: string;
  message: string;
  already_running: boolean;
};

export type LaunchStatus = {
  sphere_running: boolean;
  territory_running: boolean;
  last_error: string | null;
};

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** Lance la fenêtre Godot SphereDedicated (export ou Godot dev). */
export async function launchSphereWindow(): Promise<LaunchResult> {
  if (!isTauriRuntime()) {
    return {
      ok: false,
      mode: "browser",
      message: "Disponible uniquement dans l'app Tauri (npm run tauri dev)",
      already_running: false,
    };
  }
  return invoke<LaunchResult>("launch_sphere_window");
}

/** Lance le territoire Godot complet MainTerritory.tscn. */
export async function launchTerritoryWindow(): Promise<LaunchResult> {
  if (!isTauriRuntime()) {
    return {
      ok: false,
      mode: "browser",
      message: "Disponible uniquement dans l'app Tauri",
      already_running: false,
    };
  }
  return invoke<LaunchResult>("launch_territory_window");
}

/** Statut des processus Godot spawnés par Tauri. */
export async function getTerritoryLaunchStatus(): Promise<LaunchStatus> {
  if (!isTauriRuntime()) {
    return { sphere_running: false, territory_running: false, last_error: null };
  }
  return invoke<LaunchStatus>("get_territory_launch_status");
}