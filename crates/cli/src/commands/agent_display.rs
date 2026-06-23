//! Formatage tabulaire des listes agents (CLI).

use orchestrator::{AgentIdentity, AgentManager, AgentStatusSnapshot};

/// Affiche la liste compacte des agents.
pub async fn print_agent_list(manager: &AgentManager) -> Result<(), anyhow::Error> {
    let agents = manager.list().await.map_err(|e| anyhow::anyhow!("{e}"))?;
    if agents.is_empty() {
        println!("Aucun agent persistant.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = agents
        .iter()
        .map(|agent| {
            vec![
                agent.id().to_string(),
                agent.name().to_string(),
                agent.role().to_string(),
                agent.status().label().to_string(),
                agent.model().to_string(),
            ]
        })
        .collect();
    print_table(
        &["ID", "NOM", "RÔLE", "STATUT", "MODÈLE"],
        &rows,
    );
    Ok(())
}

/// Affiche le statut opérationnel (inbox non lus, heartbeat).
pub fn print_agent_status(snapshots: &[AgentStatusSnapshot]) {
    if snapshots.is_empty() {
        println!("Aucun agent persistant.");
        return;
    }

    let rows: Vec<Vec<String>> = snapshots
        .iter()
        .map(|snap| {
            let hb = snap
                .agent
                .config
                .last_heartbeat
                .as_deref()
                .unwrap_or("—");
            vec![
                snap.agent.id().to_string(),
                snap.agent.status().label().to_string(),
                snap.unread_inbox.to_string(),
                snap.agent.role().to_string(),
                snap.agent.model().to_string(),
                hb.to_string(),
            ]
        })
        .collect();
    print_table(
        &["ID", "STATUT", "INBOX", "RÔLE", "MODÈLE", "HEARTBEAT"],
        &rows,
    );
}

fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (idx, cell) in row.iter().enumerate() {
            widths[idx] = widths[idx].max(cell.len());
        }
    }

    let header_line: String = headers
        .iter()
        .enumerate()
        .map(|(idx, h)| format!("{:<width$}", h, width = widths[idx]))
        .collect::<Vec<_>>()
        .join("  ");
    println!("{header_line}");

    let rule: String = widths
        .iter()
        .map(|w| "-".repeat(*w))
        .collect::<Vec<_>>()
        .join("  ");
    println!("{rule}");

    for row in rows {
        let line: String = row
            .iter()
            .enumerate()
            .map(|(idx, cell)| format!("{:<width$}", cell, width = widths[idx]))
            .collect::<Vec<_>>()
            .join("  ");
        println!("{line}");
    }
}