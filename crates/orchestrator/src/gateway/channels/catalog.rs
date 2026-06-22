/// Descripteur statique d'un canal messaging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelDescriptor {
    /// Identifiant stable (`telegram`, `whatsapp`, …).
    pub id: &'static str,
    /// Nom affiché.
    pub display_name: &'static str,
    /// Variable d'environnement token / secret par défaut.
    pub default_token_env: &'static str,
    /// `true` si le canal a une implémentation dédiée (pas stub).
    pub dedicated: bool,
}

/// Catalogue Phase 10 — 18 canaux (≥ 15 requis).
pub const CHANNEL_DESCRIPTORS: &[ChannelDescriptor] = &[
    ChannelDescriptor {
        id: "webchat",
        display_name: "WebChat (WebSocket)",
        default_token_env: "",
        dedicated: true,
    },
    ChannelDescriptor {
        id: "webhook",
        display_name: "Webhook HTTP",
        default_token_env: "ORCHESTRATEUR_WEBHOOK_SECRET",
        dedicated: true,
    },
    ChannelDescriptor {
        id: "telegram",
        display_name: "Telegram Bot",
        default_token_env: "TELEGRAM_BOT_TOKEN",
        dedicated: true,
    },
    ChannelDescriptor {
        id: "discord",
        display_name: "Discord",
        default_token_env: "DISCORD_BOT_TOKEN",
        dedicated: true,
    },
    ChannelDescriptor {
        id: "slack",
        display_name: "Slack",
        default_token_env: "SLACK_BOT_TOKEN",
        dedicated: true,
    },
    ChannelDescriptor {
        id: "whatsapp",
        display_name: "WhatsApp Business",
        default_token_env: "WHATSAPP_TOKEN",
        dedicated: false,
    },
    ChannelDescriptor {
        id: "signal",
        display_name: "Signal",
        default_token_env: "SIGNAL_CLI_TOKEN",
        dedicated: false,
    },
    ChannelDescriptor {
        id: "matrix",
        display_name: "Matrix",
        default_token_env: "MATRIX_ACCESS_TOKEN",
        dedicated: false,
    },
    ChannelDescriptor {
        id: "teams",
        display_name: "Microsoft Teams",
        default_token_env: "TEAMS_BOT_TOKEN",
        dedicated: false,
    },
    ChannelDescriptor {
        id: "email",
        display_name: "Email (IMAP/SMTP)",
        default_token_env: "EMAIL_IMAP_PASSWORD",
        dedicated: false,
    },
    ChannelDescriptor {
        id: "irc",
        display_name: "IRC",
        default_token_env: "IRC_NICKSERV_PASSWORD",
        dedicated: false,
    },
    ChannelDescriptor {
        id: "google_chat",
        display_name: "Google Chat",
        default_token_env: "GOOGLE_CHAT_TOKEN",
        dedicated: false,
    },
    ChannelDescriptor {
        id: "line",
        display_name: "LINE",
        default_token_env: "LINE_CHANNEL_TOKEN",
        dedicated: false,
    },
    ChannelDescriptor {
        id: "mattermost",
        display_name: "Mattermost",
        default_token_env: "MATTERMOST_TOKEN",
        dedicated: false,
    },
    ChannelDescriptor {
        id: "rocketchat",
        display_name: "Rocket.Chat",
        default_token_env: "ROCKETCHAT_TOKEN",
        dedicated: false,
    },
    ChannelDescriptor {
        id: "bluesky",
        display_name: "Bluesky",
        default_token_env: "BLUESKY_APP_PASSWORD",
        dedicated: false,
    },
    ChannelDescriptor {
        id: "nostr",
        display_name: "Nostr",
        default_token_env: "NOSTR_PRIVATE_KEY",
        dedicated: false,
    },
    ChannelDescriptor {
        id: "twitch",
        display_name: "Twitch",
        default_token_env: "TWITCH_BOT_TOKEN",
        dedicated: false,
    },
];

/// Registre catalogue canaux.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChannelCatalog;

impl ChannelCatalog {
    /// Nouveau catalogue.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Tous les descripteurs.
    #[must_use]
    pub fn descriptors(&self) -> &'static [ChannelDescriptor] {
        CHANNEL_DESCRIPTORS
    }

    /// Recherche par identifiant.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&'static ChannelDescriptor> {
        CHANNEL_DESCRIPTORS.iter().find(|c| c.id == id)
    }

    /// Nombre de canaux enregistrés.
    #[must_use]
    pub fn count(&self) -> usize {
        CHANNEL_DESCRIPTORS.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_at_least_fifteen_channels() {
        assert!(ChannelCatalog::new().count() >= 15);
    }
}

/// Token env par défaut pour un canal du catalogue.
#[must_use]
pub fn default_token_env(channel_id: &str) -> &'static str {
    ChannelCatalog::new()
        .get(channel_id)
        .map(|d| d.default_token_env)
        .unwrap_or("")
}