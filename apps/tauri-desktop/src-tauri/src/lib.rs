mod territory_launcher;

use orchestrator_core::init as init_core;
use tauri::Manager;
use territory_launcher::{
    get_territory_launch_status, launch_sphere_window, launch_territory_window,
    TerritoryLauncherState,
};

/// Démarre l'application desktop Orchestrateur v2.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    init_core();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(TerritoryLauncherState::default())
        .invoke_handler(tauri::generate_handler![
            launch_sphere_window,
            launch_territory_window,
            get_territory_launch_status,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").expect("fenêtre main");
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();

            #[cfg(debug_assertions)]
            if std::env::var("ORCHESTRATEUR_DEVTOOLS")
                .ok()
                .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            {
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erreur exécution Tauri");
}