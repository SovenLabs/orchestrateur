//! Types partagés harness (options, rapports structurés).

use serde::{Deserialize, Serialize};

/// Options d'onboard workspace.
#[derive(Debug, Clone, Default)]
pub struct OnboardOptions {
    /// Profil sécurité TOML.
    pub profile: Option<String>,
    /// Provider LLM primaire.
    pub llm: Option<String>,
    /// Raccourci profil local souverain.
    pub local_only: bool,
    /// Installe la tâche planifiée daemon Windows.
    pub install_daemon: bool,
}

/// Options de reconfiguration harness.
#[derive(Debug, Clone, Default)]
pub struct ConfigureOptions {
    /// Profil sécurité.
    pub profile: Option<String>,
    /// Provider LLM.
    pub llm: Option<String>,
    /// Force profil local_only.
    pub local_only: bool,
}

/// Options smoke harness.
#[derive(Debug, Clone, Default)]
pub struct HarnessSmokeOptions {
    /// Ignore l'absence du gateway.
    pub skip_gateway: bool,
    /// Ignore l'étape chat agent.
    pub skip_chat: bool,
}

/// Résultat onboard (messages CLI).
#[derive(Debug, Clone)]
pub struct OnboardResult {
    /// Profil appliqué.
    pub profile: String,
    /// LLM configuré (si défini).
    pub llm: Option<String>,
    /// Token daemon généré ou réutilisé.
    pub daemon_token_generated: bool,
    /// Tâche planifiée installée.
    pub daemon_task_installed: bool,
    /// Chemin workspace.
    pub workspace_display: String,
}

/// État compact service (menus / badges).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceProbeState {
    /// État inconnu.
    Unknown,
    /// Service actif.
    Alive,
    /// Service arrêté.
    Down,
}

impl ServiceProbeState {
    /// Service répond.
    #[must_use]
    pub fn is_alive(self) -> bool {
        matches!(self, Self::Alive)
    }

    /// Depuis statut probe harness (`alive` / `down` / `skipped`).
    #[must_use]
    pub fn from_probe_status(status: &str) -> Self {
        match status {
            "alive" | "skipped" => Self::Alive,
            "down" => Self::Down,
            _ => Self::Unknown,
        }
    }

    /// Badge menu TUI (`[actif]` / `[arrêté]`).
    #[must_use]
    pub fn badge(self) -> &'static str {
        match self {
            Self::Alive => "[actif]",
            Self::Down => "[arrêté]",
            Self::Unknown => "[?]",
        }
    }
}

/// Statut HTTP d'un service (daemon / gateway).
#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatusDetail {
    /// Nom (`daemon`, `gateway`).
    pub name: String,
    /// Actif, arrêté, désactivé config, ou erreur HTTP.
    pub state: String,
    /// Version rapportée par /health.
    pub version: Option<String>,
    /// Port.
    pub port: Option<u16>,
    /// URL sonde.
    pub url: String,
    /// Détail erreur.
    pub detail: Option<String>,
}

/// Ligne statut canal gateway.
#[derive(Debug, Clone, Serialize)]
pub struct ChannelStatusRow {
    /// Identifiant canal.
    pub id: String,
    /// Activé dans TOML.
    pub enabled: bool,
    /// Nom affiché.
    pub display_name: String,
    /// Variable token.
    pub token_env: String,
    /// Token présent (`set` / `missing` / `n/a`).
    pub token_state: String,
    /// `live` ou `stub`.
    pub kind: String,
}

/// Résultat sonde providers.
#[derive(Debug, Clone)]
pub struct ProviderProbeResult {
    /// LLM joignable.
    pub llm_ok: bool,
    /// Embedding joignable.
    pub embedding_ok: bool,
    /// ID LLM configuré.
    pub llm_id: String,
    /// ID embedding configuré.
    pub embedding_id: String,
}

/// Statut d'une vérification doctor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    /// OK.
    Ok,
    /// Avertissement (non bloquant).
    Warn,
    /// Échec.
    Fail,
}

/// Une ligne du rapport doctor.
#[derive(Debug, Clone, Serialize)]
pub struct DoctorCheck {
    /// Libellé (`health bridge`, `daemon`, …).
    pub label: String,
    /// Statut.
    pub status: CheckStatus,
    /// Détail optionnel.
    pub detail: Option<String>,
}

/// Rapport doctor structuré (le CLI formate l'affichage).
#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    /// Vérifications.
    pub checks: Vec<DoctorCheck>,
    /// Canaux gateway activés.
    pub enabled_channels: Vec<String>,
    /// Nombre de problèmes bloquants.
    pub issue_count: usize,
}

impl DoctorReport {
    /// Le diagnostic est globalement OK.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.issue_count == 0
    }
}

/// Corps JSON `/health` daemon/gateway.
#[derive(Debug, Clone, Deserialize)]
pub struct HealthBody {
    /// Statut texte.
    pub status: String,
    /// Version binaire.
    pub version: String,
    /// Port d'écoute.
    pub port: u16,
}

/// Résultat arrêt daemon.
#[derive(Debug, Clone)]
pub struct DaemonStopResult {
    /// Processus arrêtés.
    pub stopped: bool,
}

/// Résultat install tâche planifiée.
#[derive(Debug, Clone)]
pub struct DaemonInstallResult {
    /// Tâche créée.
    pub installed: bool,
    /// Nom tâche Windows.
    pub task_name: String,
}