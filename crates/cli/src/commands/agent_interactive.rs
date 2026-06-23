//! Assistant guidé de création d'agent (`orch agent create --interactive`).

use anyhow::{bail, Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use orchestrator::{AgentIdentity, AgentManager};

const ROLES: &[(&str, &str)] = &[
    ("assistant", "Assistant généraliste"),
    ("analyste", "Analyse et synthèse"),
    ("coordinateur", "Coordination inter-agents"),
    ("archiviste", "Mémoire et documentation"),
    ("exécutant", "Exécution de tâches"),
    ("custom", "Saisir un rôle personnalisé"),
];

/// Paramètres collectés par le wizard interactif.
pub struct CreateWizardInput {
    /// Identifiant stable (nom de dossier).
    pub id: String,
    /// Nom affiché.
    pub name: String,
    /// Rôle fonctionnel.
    pub role: String,
    /// Modèle LLM (optionnel).
    pub model: Option<String>,
}

/// Lance le wizard et crée l'agent.
pub async fn run_create_wizard(manager: &AgentManager) -> Result<()> {
    let input = collect_create_wizard_input(manager).await?;
    let agent = manager
        .create_agent(&input.id, &input.name, &input.role, input.model.as_deref())
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!(
        "Agent créé : {} ({}) — dossier {}",
        agent.id(),
        agent.name(),
        agent.root.display()
    );
    Ok(())
}

async fn collect_create_wizard_input(manager: &AgentManager) -> Result<CreateWizardInput> {
    let theme = ColorfulTheme::default();
    let default_model = manager.deps().config.xai.model.clone();

    println!("Création d'un agent persistant — assistant guidé\n");

    let id = prompt_agent_id(manager, &theme).await?;

    let name: String = Input::with_theme(&theme)
        .with_prompt("Nom affiché")
        .default(id.clone())
        .interact_text()
        .context("saisie nom")?;

    let role_index = Select::with_theme(&theme)
        .with_prompt("Rôle")
        .items(
            &ROLES
                .iter()
                .map(|(_, label)| *label)
                .collect::<Vec<_>>(),
        )
        .default(0)
        .interact()
        .context("sélection rôle")?;

    let role = if ROLES[role_index].0 == "custom" {
        Input::with_theme(&theme)
            .with_prompt("Rôle personnalisé")
            .interact_text()
            .context("saisie rôle")?
    } else {
        ROLES[role_index].0.to_string()
    };

    let use_default_model = Confirm::with_theme(&theme)
        .with_prompt(format!("Utiliser le modèle par défaut ({default_model}) ?"))
        .default(true)
        .interact()
        .context("confirmation modèle")?;

    let model = if use_default_model {
        None
    } else {
        Some(
            Input::with_theme(&theme)
                .with_prompt("Modèle LLM")
                .default(default_model)
                .interact_text()
                .context("saisie modèle")?,
        )
    };

    Ok(CreateWizardInput {
        id,
        name,
        role,
        model,
    })
}

async fn prompt_agent_id(manager: &AgentManager, theme: &ColorfulTheme) -> Result<String> {
    loop {
        let raw: String = Input::with_theme(theme)
            .with_prompt("Identifiant (dossier, ex. researcher)")
            .interact_text()
            .context("saisie identifiant")?;
        match validate_agent_id(manager, raw.trim()).await {
            Ok(id) => return Ok(id),
            Err(msg) => println!("{msg} — réessayez."),
        }
    }
}

async fn validate_agent_id(manager: &AgentManager, raw: &str) -> Result<String, String> {
    let id = raw.trim();
    if id.is_empty() {
        return Err("l'identifiant ne peut pas être vide".into());
    }
    if id.contains(['/', '\\', ' ']) {
        return Err("utilisez un identifiant sans espaces ni séparateurs".into());
    }
    if manager.get(id).await.is_ok() {
        return Err(format!("l'agent `{id}` existe déjà"));
    }
    Ok(id.to_string())
}

/// Demande confirmation avant suppression définitive.
pub fn confirm_kill(id: &str) -> Result<bool> {
    let theme = ColorfulTheme::default();
    Confirm::with_theme(&theme)
        .with_prompt(format!(
            "Supprimer définitivement l'agent `{id}` (dossier + registre) ?"
        ))
        .default(false)
        .interact()
        .context("confirmation suppression")
}

/// Valide un identifiant fourni en ligne de commande.
pub fn ensure_id_present(id: Option<String>, interactive: bool) -> Result<String> {
    match id {
        Some(value) if !value.trim().is_empty() => Ok(value),
        _ if interactive => bail!("mode interactif : ne pas passer d'identifiant positionnel"),
        _ => bail!("identifiant requis (ou utilisez --interactive)"),
    }
}