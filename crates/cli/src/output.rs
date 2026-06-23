//! Affichage des réponses bridge CLI.

use anyhow::Result;
use orchestrator::Response;

/// Affiche une [`Response`] bridge sur stdout.
pub fn print_response(response: Response) -> Result<()> {
    match response {
        Response::Health {
            status,
            version,
            llm_available,
            embedding_available,
        } => {
            println!(
                "status={status} version={version} llm={llm_available} embedding={embedding_available}"
            );
        }
        Response::MemoryList { items, total } => {
            if items.is_empty() {
                println!("Aucune mémoire (total={total}).");
                return Ok(());
            }
            println!("# total={total}");
            for item in items {
                let tags = item.tags.join(", ");
                println!("{} | {} | tags=[{tags}]", item.id, item.title);
            }
        }
        Response::MemoryDetail { memory } => {
            println!("# {}", memory.title);
            println!("id: {}", memory.id);
            if !memory.tags.is_empty() {
                let tags: Vec<_> = memory.tags.iter().map(|t| t.as_str()).collect();
                println!("tags: {}", tags.join(", "));
            }
            println!("---");
            println!("{}", memory.content);
        }
        Response::SearchResults { items } => {
            if items.is_empty() {
                println!("Aucun résultat.");
                return Ok(());
            }
            for hit in items {
                let preview: String = hit
                    .snippet
                    .as_deref()
                    .unwrap_or("")
                    .chars()
                    .take(120)
                    .collect();
                println!("{:.3} | {} | {}", hit.score, hit.memory_id, preview);
            }
        }
        Response::Assimilated { memory_id, title } => {
            println!("Assimilé : {title} ({memory_id})");
        }
        Response::GraphSummary {
            node_count,
            edge_count,
            hubs,
        } => {
            println!("Nœuds : {node_count}");
            println!("Arêtes : {edge_count}");
            for hub in hubs {
                println!(
                    "  hub ({}) : {} [{}]",
                    hub.inbound_links, hub.title, hub.memory_id
                );
            }
        }
        Response::AuditLog {
            entries,
            chain_intact,
        } => {
            let status = if chain_intact { "intacte" } else { "ROMPUE" };
            println!("Chaîne d'audit : {status}");
            for entry in entries {
                println!(
                    "{} | {} | {} | {}",
                    entry.timestamp, entry.event_type, entry.details, entry.hash
                );
            }
        }
        Response::Error(err) => {
            anyhow::bail!("[{}] {}", err.kind, err.message);
        }
        Response::Success { message } => {
            println!("{message}");
        }
        Response::ChatReply {
            reply,
            tools_invoked,
            auto_assimilated,
            auto_executed_skills,
        } => {
            println!("{reply}");
            if !tools_invoked.is_empty() {
                println!("# outils: {}", tools_invoked.join(", "));
            }
            if !auto_executed_skills.is_empty() {
                println!(
                    "# skills auto-exécutées: {}",
                    auto_executed_skills.join(", ")
                );
            }
            if let Some(summary) = auto_assimilated {
                println!("# auto-assimilé: {summary}");
            }
        }
        Response::SkillList { skills } => {
            if skills.is_empty() {
                println!("Aucune skill enregistrée.");
                return Ok(());
            }
            for skill in skills {
                let version = skill
                    .version
                    .as_deref()
                    .map(|v| format!(" v{v}"))
                    .unwrap_or_default();
                println!(
                    "{} [{}{}] — {}",
                    skill.name, skill.source, version, skill.description
                );
            }
        }
        Response::SkillResult { message } => {
            println!("{message}");
        }
        Response::MarketplaceList {
            version,
            catalog_hash,
            entries,
        } => {
            let hash = catalog_hash.as_deref().unwrap_or("(non signé)");
            println!("# Marketplace v{version} hash={hash} ({} entrées)", entries.len());
            for entry in entries {
                let state = if entry.enabled { "on" } else { "off" };
                println!(
                    "{:<16} {:<6} {} — {}",
                    entry.id, state, entry.version, entry.description
                );
            }
        }
        Response::HubIntegrityReport { report } => {
            println!(
                "Hub intégrité : {} valide(s), {} invalide(s)",
                report.valid_count,
                report.invalid.len()
            );
            for (path, message) in &report.invalid {
                println!("  ! {path}: {message}");
            }
        }
        Response::Event(_) => {}
        Response::WatcherStatus { status } => {
            println!(
                "watcher enabled={} running={} pending={} processed={}",
                status.enabled,
                status.running,
                status.drafts_pending,
                status.sessions_processed
            );
            for dir in &status.watch_dirs {
                println!("  watch: {dir}");
            }
            if let Some(err) = &status.last_error {
                println!("  erreur: {err}");
            }
        }
        Response::DraftList { items, total } => {
            println!("# brouillons total={total}");
            for item in items {
                println!(
                    "{} | {} | {:?} | tags=[{}]",
                    item.id,
                    item.title,
                    item.kind,
                    item.tags.join(", ")
                );
            }
        }
        Response::DraftPublished {
            draft_id,
            memory_id,
            title,
        } => {
            println!("Brouillon publié : {title} (draft={draft_id}, memory={memory_id})");
        }
        Response::DraftDiscarded { id } => {
            println!("Brouillon supprimé : {id}");
        }
        Response::DraftDetail { draft } => {
            println!("# brouillon {}", draft.id);
            println!("status: {:?}", draft.status);
            println!("kind: {:?}", draft.draft.kind);
            println!("title: {}", draft.draft.title);
            if !draft.draft.tags.is_empty() {
                println!("tags: {}", draft.draft.tags.join(", "));
            }
            println!("---");
            println!("{}", draft.draft.content);
        }
    }
    Ok(())
}