use std::collections::{HashMap, HashSet, VecDeque};

use thiserror::Error;

use super::hub::SkillHubDescriptor;
use super::metadata::SkillMetadata;

/// Erreur de résolution de dépendances entre skills.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DependencyError {
    /// Skill requise introuvable.
    #[error("dépendance manquante: {skill} requiert {missing}")]
    Missing {
        /// Skill demandeuse.
        skill: String,
        /// Dépendance absente.
        missing: String,
    },
    /// Cycle de dépendances.
    #[error("cycle de dépendances: {0}")]
    Cycle(String),
}

/// Résout l'ordre de chargement topologique des skills hub.
pub fn resolve_load_order(
    descriptors: &[SkillHubDescriptor],
    metadata_by_id: &HashMap<String, SkillMetadata>,
) -> Result<Vec<String>, DependencyError> {
    let enabled_ids: HashSet<&str> = descriptors
        .iter()
        .filter(|d| d.enabled)
        .map(|d| d.id.as_str())
        .collect();

    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();

    for descriptor in descriptors.iter().filter(|d| d.enabled) {
        in_degree.entry(descriptor.id.clone()).or_insert(0);
        graph.entry(descriptor.id.clone()).or_default();
        if let Some(meta) = metadata_by_id.get(&descriptor.id) {
            for dep in &meta.dependencies {
                if !enabled_ids.contains(dep.as_str()) {
                    return Err(DependencyError::Missing {
                        skill: descriptor.id.clone(),
                        missing: dep.clone(),
                    });
                }
                graph
                    .entry(dep.clone())
                    .or_default()
                    .push(descriptor.id.clone());
                *in_degree.entry(descriptor.id.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|(_, deg)| **deg == 0)
        .map(|(id, _)| id.clone())
        .collect();
    queue.make_contiguous().sort();

    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node.clone());
        if let Some(children) = graph.get(&node) {
            let mut next: Vec<String> = Vec::new();
            for child in children {
                let entry = in_degree.get_mut(child).expect("in_degree");
                *entry = entry.saturating_sub(1);
                if *entry == 0 {
                    next.push(child.clone());
                }
            }
            next.sort();
            for child in next {
                queue.push_back(child);
            }
        }
    }

    if order.len() != in_degree.len() {
        return Err(DependencyError::Cycle(
            "impossible de résoudre l'ordre de chargement".into(),
        ));
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    fn descriptor(id: &str) -> SkillHubDescriptor {
        SkillHubDescriptor {
            id: id.into(),
            name: id.into(),
            description: id.into(),
            version: "0.1.0".into(),
            kind: "subprocess".into(),
            origin: "filesystem".into(),
            path: PathBuf::from("skill.toml"),
            enabled: true,
        }
    }

    #[test]
    fn topological_order_respects_dependencies() {
        let descriptors = vec![descriptor("a"), descriptor("b"), descriptor("c")];
        let mut meta = HashMap::new();
        let mut meta_a = SkillMetadata::minimal("a", "a");
        meta_a.dependencies = vec!["b".into()];
        meta.insert("a".into(), meta_a);
        meta.insert("b".into(), SkillMetadata::minimal("b", "b"));
        meta.insert("c".into(), SkillMetadata::minimal("c", "c"));

        let order = resolve_load_order(&descriptors, &meta).unwrap();
        let pos = |id: &str| order.iter().position(|v| v == id).unwrap();
        assert!(pos("b") < pos("a"));
    }

}