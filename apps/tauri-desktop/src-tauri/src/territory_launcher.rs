//! Lancement Godot / export standalone depuis le desktop Tauri (Phase 25).

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use serde::Serialize;
use tauri::State;

/// État partagé des processus Territoire Graphique lancés par Tauri.
pub struct TerritoryLauncherState {
    sphere_child: Mutex<Option<Child>>,
    territory_child: Mutex<Option<Child>>,
    last_error: Mutex<Option<String>>,
}

impl Default for TerritoryLauncherState {
    fn default() -> Self {
        Self {
            sphere_child: Mutex::new(None),
            territory_child: Mutex::new(None),
            last_error: Mutex::new(None),
        }
    }
}

/// Résultat d'un lancement de fenêtre Godot.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct LaunchResult {
    pub ok: bool,
    pub mode: String,
    pub message: String,
    pub already_running: bool,
}

/// Statut des fenêtres Territoire lancées depuis Tauri.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct LaunchStatus {
    pub sphere_running: bool,
    pub territory_running: bool,
    pub last_error: Option<String>,
}

/// Ouvre la fenêtre Godot `SphereDedicated.tscn` (ou l'exécutable exporté).
#[tauri::command]
pub fn launch_sphere_window(state: State<'_, TerritoryLauncherState>) -> LaunchResult {
    launch_godot_scene(
        state,
        LaunchTarget::Sphere,
        "res://scenes/SphereDedicated.tscn",
        "OrchestrateurSphere.exe",
    )
}

/// Ouvre le territoire Godot complet `MainTerritory.tscn`.
#[tauri::command]
pub fn launch_territory_window(state: State<'_, TerritoryLauncherState>) -> LaunchResult {
    launch_godot_scene(
        state,
        LaunchTarget::Territory,
        "res://scenes/MainTerritory.tscn",
        "OrchestrateurTerritory.exe",
    )
}

/// Retourne si les processus Godot lancés par Tauri sont encore actifs.
#[tauri::command]
pub fn get_territory_launch_status(state: State<'_, TerritoryLauncherState>) -> LaunchStatus {
    LaunchStatus {
        sphere_running: child_running(&state.sphere_child),
        territory_running: child_running(&state.territory_child),
        last_error: state.last_error.lock().ok().and_then(|g| g.clone()),
    }
}

enum LaunchTarget {
    Sphere,
    Territory,
}

fn launch_godot_scene(
    state: State<'_, TerritoryLauncherState>,
    target: LaunchTarget,
    scene: &str,
    export_exe_name: &str,
) -> LaunchResult {
    let child_slot = match target {
        LaunchTarget::Sphere => &state.sphere_child,
        LaunchTarget::Territory => &state.territory_child,
    };
    let label = match target {
        LaunchTarget::Sphere => "sphere",
        LaunchTarget::Territory => "territory",
    };

    if let Ok(mut guard) = child_slot.lock() {
        if child_alive(guard.as_mut()) {
            return LaunchResult {
                ok: true,
                mode: label.into(),
                message: "Fenêtre déjà ouverte — focus manuel requis".into(),
                already_running: true,
            };
        }
        *guard = None;
    }

    let repo_root = match workspace_root() {
        Some(p) => p,
        None => {
            let msg = "Racine workspace introuvable (orchestrateur)".to_string();
            store_error(&state, &msg);
            return LaunchResult {
                ok: false,
                mode: label.into(),
                message: msg,
                already_running: false,
            };
        }
    };

    let token = std::env::var("ORCHESTRATEUR_DAEMON_TOKEN").unwrap_or_else(|_| "dev".into());

    if let Some(export_path) = find_export_exe(&repo_root, export_exe_name) {
        match spawn_export(&export_path, &token) {
            Ok(child) => {
                clear_error(&state);
                if let Ok(mut guard) = child_slot.lock() {
                    *guard = Some(child);
                }
                return LaunchResult {
                    ok: true,
                    mode: "export".into(),
                    message: format!("Export lancé : {}", export_path.display()),
                    already_running: false,
                };
            }
            Err(err) => store_error(&state, &err),
        }
    }

    let godot_project = repo_root.join("territoire-graphique/godot-project");
    if !godot_project.join("project.godot").is_file() {
        let msg = format!(
            "Projet Godot introuvable : {}",
            godot_project.display()
        );
        store_error(&state, &msg);
        return LaunchResult {
            ok: false,
            mode: label.into(),
            message: msg,
            already_running: false,
        };
    }

    let godot_exe = match find_godot_exe() {
        Some(p) => p,
        None => {
            let msg = "Godot 4.7 introuvable — installez Godot ou exportez la sphère (just export-sphere)"
                .to_string();
            store_error(&state, &msg);
            return LaunchResult {
                ok: false,
                mode: label.into(),
                message: msg,
                already_running: false,
            };
        }
    };

    match spawn_godot_dev(&godot_exe, &godot_project, scene, &token) {
        Ok(child) => {
            clear_error(&state);
            if let Ok(mut guard) = child_slot.lock() {
                *guard = Some(child);
            }
            LaunchResult {
                ok: true,
                mode: "godot_dev".into(),
                message: format!("Godot lancé : {scene}"),
                already_running: false,
            }
        }
        Err(err) => {
            store_error(&state, &err);
            LaunchResult {
                ok: false,
                mode: label.into(),
                message: err,
                already_running: false,
            }
        }
    }
}

/// Racine du dépôt orchestrateur (Godot + harness).
pub fn repo_root_for_harness() -> Option<PathBuf> {
    workspace_root()
}

fn workspace_root() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("ORCHESTRATEUR_WORKSPACE") {
        let p = PathBuf::from(dir);
        if p.join("territoire-graphique/godot-project/project.godot").is_file() {
            return Some(p);
        }
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let from_tauri = manifest
        .parent()?
        .parent()?
        .parent()?
        .to_path_buf();
    if from_tauri
        .join("territoire-graphique/godot-project/project.godot")
        .is_file()
    {
        return Some(from_tauri);
    }

    std::env::current_dir().ok().and_then(|cwd| {
        for ancestor in cwd.ancestors() {
            if ancestor
                .join("territoire-graphique/godot-project/project.godot")
                .is_file()
            {
                return Some(ancestor.to_path_buf());
            }
        }
        None
    })
}

fn find_export_exe(repo_root: &Path, exe_name: &str) -> Option<PathBuf> {
    let candidates = [
        repo_root.join("territoire-graphique/dist/sphere").join(exe_name),
        repo_root.join("territoire-graphique/dist/territory").join(exe_name),
        repo_root.join("dist/sphere").join(exe_name),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

fn find_godot_exe() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("GODOT_EXE") {
        let p = PathBuf::from(path);
        if p.is_file() {
            return Some(p);
        }
    }

    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(local).join("Godot/Godot_v4.7-stable_win64.exe");
        if p.is_file() {
            return Some(p);
        }
    }

    let program_files = PathBuf::from(r"C:\Program Files\Godot\Godot_v4.7-stable_win64.exe");
    if program_files.is_file() {
        return Some(program_files);
    }

    which_godot()
}

#[cfg(windows)]
fn which_godot() -> Option<PathBuf> {
    let output = Command::new("where").arg("godot").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let line = std::str::from_utf8(&output.stdout).ok()?.lines().next()?;
    let p = PathBuf::from(line.trim());
    if p.is_file() {
        Some(p)
    } else {
        None
    }
}

#[cfg(not(windows))]
fn which_godot() -> Option<PathBuf> {
    let output = Command::new("which").arg("godot").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let line = std::str::from_utf8(&output.stdout).ok()?.trim();
    let p = PathBuf::from(line);
    if p.is_file() {
        Some(p)
    } else {
        None
    }
}

fn spawn_godot_dev(
    godot_exe: &Path,
    project_dir: &Path,
    scene: &str,
    token: &str,
) -> Result<Child, String> {
    Command::new(godot_exe)
        .arg("--path")
        .arg(project_dir)
        .arg(scene)
        .env("ORCHESTRATEUR_DAEMON_TOKEN", token)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Échec lancement Godot : {e}"))
}

fn spawn_export(exe: &Path, token: &str) -> Result<Child, String> {
    Command::new(exe)
        .env("ORCHESTRATEUR_DAEMON_TOKEN", token)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Échec lancement export : {e}"))
}

fn child_alive(child: Option<&mut Child>) -> bool {
    match child {
        Some(c) => match c.try_wait() {
            Ok(None) => true,
            _ => false,
        },
        None => false,
    }
}

fn child_running(slot: &Mutex<Option<Child>>) -> bool {
    slot.lock()
        .ok()
        .map(|mut guard| child_alive(guard.as_mut()))
        .unwrap_or(false)
}

fn store_error(state: &TerritoryLauncherState, msg: &str) {
    if let Ok(mut guard) = state.last_error.lock() {
        *guard = Some(msg.to_string());
    }
}

fn clear_error(state: &TerritoryLauncherState) {
    if let Ok(mut guard) = state.last_error.lock() {
        *guard = None;
    }
}