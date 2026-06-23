//! Versioning du protocole WebSocket territorial.

/// Version sémantique du protocole JSON client ↔ daemon.
///
/// Règles de compatibilité :
/// - **MAJOR** : rupture (champs obligatoires supprimés, renommage `type`)
/// - **MINOR** : extensions rétrocompatibles (nouveaux champs optionnels, événements)
/// - **PATCH** : clarifications doc, sans impact wire format
pub const PROTOCOL_VERSION: &str = "1.2.0";

/// Version minimale acceptée par le daemon (clients plus anciens reçoivent un avertissement).
pub const PROTOCOL_MIN_CLIENT: &str = "1.0.0";

/// Vérifie si la version client est supportée (comparaison major.minor simplifiée).
#[must_use]
pub fn is_client_version_supported(client: &str) -> bool {
    let (c_major, c_minor) = parse_prefix(client);
    let (min_major, min_minor) = parse_prefix(PROTOCOL_MIN_CLIENT);
    c_major > min_major || (c_major == min_major && c_minor >= min_minor)
}

fn parse_prefix(version: &str) -> (u32, u32) {
    let mut parts = version.split('.');
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_1_0_clients() {
        assert!(is_client_version_supported("1.0.0"));
        assert!(is_client_version_supported("1.1.0"));
        assert!(is_client_version_supported("1.2.0"));
    }

    #[test]
    fn rejects_0_x_clients() {
        assert!(!is_client_version_supported("0.9.0"));
    }
}