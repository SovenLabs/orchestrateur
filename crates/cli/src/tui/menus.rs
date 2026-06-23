//! Définition des menus et hints (plan harness P0–P6).

/// Identifiant de sous-menu (pile de navigation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubMenuId {
    SetupRoot,
    Health,
    Daemon,
    Gateway,
    GatewayChannels,
    Cortex,
    Drafts,
    Skills,
    SkillsHub,
    SettingsRoot,
    SettingsProviders,
    Maintenance,
    Watcher,
}

/// Action exécutée depuis un item de menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Back,
    Quit,
    SubMenu(SubMenuId),
    OpenSettings,
    Run(HarnessAction),
}

/// Action harness mappée vers `orchestrator::harness` / bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarnessAction {
    Doctor,
    DaemonStatus,
    DaemonInstall,
    DaemonStop,
    GatewayStatus,
    HarnessSmoke,
    HarnessRun,
    Health,
    Search,
    ListMemories,
    Assimilate,
    Chat,
    Graph,
    DraftList,
    ImportMd,
    Reindex,
    ChannelsList,
    ChannelsStatus,
    SkillsList,
    SkillRun,
    SkillsHubList,
    SkillsHubPath,
    SkillsHubMarketplace,
    SkillsHubSync,
    SkillsHubVerify,
    ProvidersList,
    ProvidersTest,
    ProviderSetOllama,
    ProviderSetXai,
    ConfigureLocalOnly,
    DaemonInstallSettings,
    ShowConfigPath,
    Update,
    Uninstall,
    Audit,
    WatcherStatus,
    WatcherStart,
    WatcherStop,
    McpInfo,
}

/// Entrée de menu avec hint entre parenthèses.
#[derive(Debug, Clone, Copy)]
pub struct MenuItem {
    pub label: &'static str,
    pub hint: &'static str,
    pub action: MenuAction,
}

pub const BACK: MenuItem = MenuItem {
    label: "← Retour",
    hint: "revenir au menu précédent",
    action: MenuAction::Back,
};

pub const QUIT: MenuItem = MenuItem {
    label: "Quitter",
    hint: "fermer le menu",
    action: MenuAction::Quit,
};

/// Items pour un sous-menu donné.
pub fn items_for(id: SubMenuId) -> &'static [MenuItem] {
    match id {
        SubMenuId::SetupRoot => SETUP_ROOT,
        SubMenuId::Health => HEALTH,
        SubMenuId::Daemon => DAEMON,
        SubMenuId::Gateway => GATEWAY,
        SubMenuId::GatewayChannels => GATEWAY_CHANNELS,
        SubMenuId::Cortex => CORTEX,
        SubMenuId::Drafts => DRAFTS,
        SubMenuId::Skills => SKILLS,
        SubMenuId::SkillsHub => SKILLS_HUB,
        SubMenuId::SettingsRoot => SETTINGS_ROOT,
        SubMenuId::SettingsProviders => SETTINGS_PROVIDERS,
        SubMenuId::Maintenance => MAINTENANCE,
        SubMenuId::Watcher => WATCHER,
    }
}

pub fn title_for(id: SubMenuId) -> &'static str {
    match id {
        SubMenuId::SetupRoot => "setup — Centre de commande harness",
        SubMenuId::Health => "Démarrage & santé",
        SubMenuId::Daemon => "Daemon",
        SubMenuId::Gateway => "Gateway",
        SubMenuId::GatewayChannels => "Canaux messaging",
        SubMenuId::Cortex => "Cortex & mémoire",
        SubMenuId::Drafts => "Brouillons",
        SubMenuId::Skills => "Skills & extensions",
        SubMenuId::SkillsHub => "Hub skills",
        SubMenuId::SettingsRoot => "settings — Configuration",
        SubMenuId::SettingsProviders => "Profil & providers",
        SubMenuId::Maintenance => "Maintenance système",
        SubMenuId::Watcher => "Watcher sessions",
    }
}

static SETUP_ROOT: &[MenuItem] = &[
    MenuItem {
        label: "Démarrage & santé",
        hint: "quotidien : doctor, daemon, gateway, harness",
        action: MenuAction::SubMenu(SubMenuId::Health),
    },
    MenuItem {
        label: "Cortex & mémoire",
        hint: "chercher, assimiler, chat, graphe, brouillons",
        action: MenuAction::SubMenu(SubMenuId::Cortex),
    },
    MenuItem {
        label: "Gateway & canaux",
        hint: "statut, démarrage, Telegram/Discord/Slack",
        action: MenuAction::SubMenu(SubMenuId::Gateway),
    },
    MenuItem {
        label: "Skills & extensions",
        hint: "liste, exécution, hub marketplace",
        action: MenuAction::SubMenu(SubMenuId::Skills),
    },
    MenuItem {
        label: "Configuration",
        hint: "ouvre le menu settings",
        action: MenuAction::OpenSettings,
    },
    MenuItem {
        label: "Maintenance système",
        hint: "mise à jour, audit, watcher, désinstall",
        action: MenuAction::SubMenu(SubMenuId::Maintenance),
    },
    QUIT,
];

static HEALTH: &[MenuItem] = &[
    BACK,
    MenuItem {
        label: "Doctor",
        hint: "vérifie Cortex, daemon, gateway et tokens",
        action: MenuAction::Run(HarnessAction::Doctor),
    },
    MenuItem {
        label: "Daemon",
        hint: "service WS pour desktop et Godot",
        action: MenuAction::SubMenu(SubMenuId::Daemon),
    },
    MenuItem {
        label: "Gateway",
        hint: "WebSocket et webhooks messaging",
        action: MenuAction::SubMenu(SubMenuId::Gateway),
    },
    MenuItem {
        label: "Harness smoke",
        hint: "test rapide sans LLM obligatoire",
        action: MenuAction::Run(HarnessAction::HarnessSmoke),
    },
    MenuItem {
        label: "Harness run",
        hint: "démarre daemon+gateway, attend Ctrl+C",
        action: MenuAction::Run(HarnessAction::HarnessRun),
    },
];

static DAEMON: &[MenuItem] = &[
    BACK,
    MenuItem {
        label: "Statut",
        hint: "tâche Windows + sonde HTTP /health",
        action: MenuAction::Run(HarnessAction::DaemonStatus),
    },
    MenuItem {
        label: "Arrêter",
        hint: "stoppe processus et tâche planifiée",
        action: MenuAction::Run(HarnessAction::DaemonStop),
    },
    MenuItem {
        label: "Installer au démarrage",
        hint: "crée la tâche Windows à la connexion",
        action: MenuAction::Run(HarnessAction::DaemonInstall),
    },
];

static GATEWAY: &[MenuItem] = &[
    BACK,
    MenuItem {
        label: "Statut gateway",
        hint: "sonde HTTP /health du gateway",
        action: MenuAction::Run(HarnessAction::GatewayStatus),
    },
    MenuItem {
        label: "Canaux messaging",
        hint: "configurer Telegram, Discord, Slack (guide credentials)",
        action: MenuAction::SubMenu(SubMenuId::GatewayChannels),
    },
];

// Menu canaux dynamique : voir `tui/mod.rs::run_gateway_channels_menu`.
static GATEWAY_CHANNELS: &[MenuItem] = &[BACK];

static CORTEX: &[MenuItem] = &[
    BACK,
    MenuItem {
        label: "Rechercher",
        hint: "recherche sémantique dans LanceDB",
        action: MenuAction::Run(HarnessAction::Search),
    },
    MenuItem {
        label: "Lister les mémoires",
        hint: "affiche les notes Markdown indexées",
        action: MenuAction::Run(HarnessAction::ListMemories),
    },
    MenuItem {
        label: "Assimiler une idée",
        hint: "crée une mémoire via le LLM configuré",
        action: MenuAction::Run(HarnessAction::Assimilate),
    },
    MenuItem {
        label: "Chat libre",
        hint: "conversation directe avec le provider",
        action: MenuAction::Run(HarnessAction::Chat),
    },
    MenuItem {
        label: "Graphe",
        hint: "stats nœuds, arêtes et hubs",
        action: MenuAction::Run(HarnessAction::Graph),
    },
    MenuItem {
        label: "Brouillons",
        hint: "file de relecture avant publication",
        action: MenuAction::SubMenu(SubMenuId::Drafts),
    },
    MenuItem {
        label: "Import Markdown",
        hint: "ingère un dossier de fichiers .md",
        action: MenuAction::Run(HarnessAction::ImportMd),
    },
    MenuItem {
        label: "Ré-indexer",
        hint: "recalcule les embeddings de toutes les mémoires",
        action: MenuAction::Run(HarnessAction::Reindex),
    },
];

static DRAFTS: &[MenuItem] = &[
    BACK,
    MenuItem {
        label: "Lister les brouillons",
        hint: "drafts en attente de publication",
        action: MenuAction::Run(HarnessAction::DraftList),
    },
];

static SKILLS: &[MenuItem] = &[
    BACK,
    MenuItem {
        label: "Lister skills",
        hint: "skills natives et plugins découverts",
        action: MenuAction::Run(HarnessAction::SkillsList),
    },
    MenuItem {
        label: "Exécuter une skill",
        hint: "lance une skill par son identifiant",
        action: MenuAction::Run(HarnessAction::SkillRun),
    },
    MenuItem {
        label: "Hub",
        hint: "marketplace, sync et vérification BLAKE3",
        action: MenuAction::SubMenu(SubMenuId::SkillsHub),
    },
];

static SKILLS_HUB: &[MenuItem] = &[
    BACK,
    MenuItem {
        label: "Chemin hub",
        hint: "répertoire plugins configuré",
        action: MenuAction::Run(HarnessAction::SkillsHubPath),
    },
    MenuItem {
        label: "Lister hub",
        hint: "skills installées localement",
        action: MenuAction::Run(HarnessAction::SkillsHubList),
    },
    MenuItem {
        label: "Marketplace",
        hint: "catalogue distant des skills",
        action: MenuAction::Run(HarnessAction::SkillsHubMarketplace),
    },
    MenuItem {
        label: "Synchroniser",
        hint: "installe les skills du catalogue",
        action: MenuAction::Run(HarnessAction::SkillsHubSync),
    },
    MenuItem {
        label: "Vérifier empreintes",
        hint: "contrôle BLAKE3 des manifestes",
        action: MenuAction::Run(HarnessAction::SkillsHubVerify),
    },
];

static SETTINGS_ROOT: &[MenuItem] = &[
    MenuItem {
        label: "← Retour au hub setup",
        hint: "revenir au menu précédent",
        action: MenuAction::Back,
    },
    MenuItem {
        label: "Profil & providers",
        hint: "sécurité, LLM primaire, test joignabilité",
        action: MenuAction::SubMenu(SubMenuId::SettingsProviders),
    },
    MenuItem {
        label: "Profil local_only",
        hint: "zéro egress cloud, ollama par défaut",
        action: MenuAction::Run(HarnessAction::ConfigureLocalOnly),
    },
    MenuItem {
        label: "Daemon & tokens",
        hint: "tâche planifiée et ORCHESTRATEUR_DAEMON_TOKEN",
        action: MenuAction::Run(HarnessAction::DaemonInstallSettings),
    },
    MenuItem {
        label: "Workspace & config",
        hint: "chemin et fichier orchestrator.toml",
        action: MenuAction::Run(HarnessAction::ShowConfigPath),
    },
    QUIT,
];

static SETTINGS_PROVIDERS: &[MenuItem] = &[
    BACK,
    MenuItem {
        label: "Lister providers",
        hint: "catalogue LLM et embedding",
        action: MenuAction::Run(HarnessAction::ProvidersList),
    },
    MenuItem {
        label: "Tester providers",
        hint: "sonde joignabilité LLM / embedding",
        action: MenuAction::Run(HarnessAction::ProvidersTest),
    },
    MenuItem {
        label: "LLM primaire : ollama",
        hint: "définit ollama dans orchestrator.toml",
        action: MenuAction::Run(HarnessAction::ProviderSetOllama),
    },
    MenuItem {
        label: "LLM primaire : xai",
        hint: "définit xAI dans orchestrator.toml",
        action: MenuAction::Run(HarnessAction::ProviderSetXai),
    },
];

static MAINTENANCE: &[MenuItem] = &[
    BACK,
    MenuItem {
        label: "Mettre à jour",
        hint: "télécharge et installe la dernière version",
        action: MenuAction::Run(HarnessAction::Update),
    },
    MenuItem {
        label: "Santé bridge",
        hint: "ping interne facade / health check",
        action: MenuAction::Run(HarnessAction::Health),
    },
    MenuItem {
        label: "Audit",
        hint: "journal BLAKE3 des dernières opérations",
        action: MenuAction::Run(HarnessAction::Audit),
    },
    MenuItem {
        label: "Watcher",
        hint: "surveille les sessions Markdown",
        action: MenuAction::SubMenu(SubMenuId::Watcher),
    },
    MenuItem {
        label: "MCP serve",
        hint: "expose Cortex aux IDE (Cursor, Claude)",
        action: MenuAction::Run(HarnessAction::McpInfo),
    },
    MenuItem {
        label: "Désinstaller",
        hint: "arrêt sécurité + retrait PATH et binaires",
        action: MenuAction::Run(HarnessAction::Uninstall),
    },
];

static WATCHER: &[MenuItem] = &[
    BACK,
    MenuItem {
        label: "Statut watcher",
        hint: "état du surveillant de sessions",
        action: MenuAction::Run(HarnessAction::WatcherStatus),
    },
    MenuItem {
        label: "Démarrer watcher",
        hint: "active la surveillance Markdown",
        action: MenuAction::Run(HarnessAction::WatcherStart),
    },
    MenuItem {
        label: "Arrêter watcher",
        hint: "stoppe le surveillant global",
        action: MenuAction::Run(HarnessAction::WatcherStop),
    },
];