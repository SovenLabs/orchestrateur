//! # Orchestrateur — L'Esprit
//!
//! Couche application : facade, use cases, Skill Registry.
//! Ne dépend que du squelette [`cortex`] et de ses ports.
//!
//! Garde-fous : Rust stable uniquement — pas de `#![feature(...)]`.

#![cfg_attr(not(feature = "plugins-native"), forbid(unsafe_code))]
#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

/// Contrat client visuel ↔ orchestrateur (`Command`, `Response`, `OrchestratorHandle`).
pub mod bridge;
/// Configuration applicative (`OrchestratorConfig`).
pub mod config;
/// Port MCP (Phase 9).
pub mod mcp;
/// Registre typé des providers (Phase 9).
pub mod providers;
/// Injection des ports Cortex (`AppDependencies`).
pub mod deps;
/// Erreurs de la couche application.
pub mod error;
/// Publication des événements de domaine.
pub mod events;
/// Facade publique stable (`OrchestratorFacade`).
pub mod facade;
/// Sondes harness partagées CLI / desktop.
pub mod harness;
/// Sondes de disponibilité providers (health bridge).
pub mod health;
/// Ports LLM — génération de [`MemoryDraft`] et chat.
pub mod llm;
/// Brouillon structuré issu des providers IA (`MemoryDraft`).
pub mod memory_draft;
/// Brouillons persistés et port [`DraftRepository`].
pub mod draft;
/// Dédup et prompts d'extraction d'insights.
pub mod memory;
/// Watcher sessions Markdown → brouillons insight.
pub mod watcher;
/// Défense en profondeur — validation adversariale des sorties LLM.
pub mod security;
/// Squelette Skills (trait, registre, noop).
pub mod skills;
/// Daemon WebSocket local pour clients visuels (feature `websocket-server`).
#[cfg(feature = "websocket-server")]
pub mod daemon;
/// Boucle agent Phase 7.
pub mod agent;
/// Protocole B212 (enfant Orchestrateur — Phase 3).
pub mod b212;
/// Messagerie inter-agents persistants (Phase 2).
pub mod communication;
/// Cycle de vie heartbeat des agents persistants (Phase 2).
pub mod heartbeat;
/// Gestion centralisée des agents persistants (Phase 2).
pub mod manager;
/// Worker tick agents persistants (Phase 2b).
pub mod worker;
/// Entités et dossier standard des agents persistants (Phase 2).
pub mod persistent;
/// Registre central des agents persistants (Phase 2).
pub mod registry;
/// Registre d'outils agent Phase 7.
pub mod tools;
/// Gateway WebSocket + canaux (feature `gateway`).
#[cfg(feature = "gateway")]
pub mod gateway;
/// Use cases applicatifs testables en mémoire.
pub mod use_cases;

/// Mocks in-memory des ports pour tests isolés.
pub mod testing;

pub use bridge::{
    audit_from_response, domain_event_action, execute_command, format_assimilate_user_prompt,
    format_health_status, graph_from_response, graph_status_message, health_from_response,
    spawn_orchestrator_bridge, AppError, AuditUpdate, BridgeError, BridgeSearchHit, BridgeUiAction,
    ChannelHandle, Command, FanoutEventPublisher, BridgeSkillContext, GraphUpdate, HealthUpdate,
    HubIntegritySummary, HubSummary, MarketplaceEntrySummary, MemoryDetailView, MemorySummary,
    OrchestratorHandle, OrchestratorThread, Response, SkillSummary, DraftSummary, WatcherStatus,
};
pub use config::{
    AgentSettingsConfig, AgentsConfig, AuditConfig, B212Config, BehavioralConfig, ConfigError,
    DaemonConfig,
    WatcherConfig,
    GatewayChannelConfig, GatewayConfig, IntegrityConfig, McpConfig, McpServerConfig, OllamaConfig,
    OrchestratorConfig,
    MemoryConfig, ProvidersConfig, SecurityConfig, SkillsHubConfig, SkillsHubEntryConfig,
    VectorStoreConfig,
    XaiConfig,
};
pub use config::editor::{set_channel_enabled, set_primary_llm, set_security_profile};
pub use mcp::{McpError, McpGateway, McpToolInfo};
pub use providers::{
    ApiFamily, ProviderDescriptor, ProviderKind, ProviderProfile, ProviderProfiles,
    ProviderRegistry, EMBEDDING_DESCRIPTORS, LLM_DESCRIPTORS,
};
pub use cortex::DomainEvent;
pub use deps::AppDependencies;
pub use error::{OrchestratorError, SkillError};
pub use events::{EventPublisher, NoopEventPublisher, TracingEventPublisher};
pub use facade::OrchestratorFacade;
pub use llm::{ChatMessage, LlmCapabilities, LlmError, LlmProvider, LlmUsageRecorded};
pub use memory_draft::{BacklinkDraft, BacklinkDraftKind, MemoryDraft};
pub use draft::{DraftError, DraftRepository, DraftStatus, StoredDraft};
pub use security::{
    assert_llm_egress_allowed, assert_text_safe_for_llm, build_security_context,
    build_test_security_context, is_local_llm_provider, scan_secrets, AuditEvent, BehavioralError,
    IntegrityStatus, MemoryDraftValidator, SecurityBootstrapError, SecurityContext,
    SecurityGateError, SecurityProfile, ValidationError,
};
pub use skills::{
    best_skill_match, compute_integrity_hash, suggest_skills, HubError, IntegrityReport,
    MarketplaceCatalog,
    MarketplaceEntry, MarketplaceError, MarketplaceSyncResult, NoopSkill, Skill, SkillContext,
    SkillEntry, SkillHubDescriptor, SkillManifest, SkillOutput, SkillPluginConfig, SkillRegistry,
    SkillSource, SkillsHub, SkillsMarketplace, SubprocessPluginSkill, verify_integrity_hash,
};
#[cfg(feature = "plugins-native")]
pub use skills::{NativePluginError, NativePluginSkill};
#[cfg(feature = "websocket-server")]
pub use daemon::{run_daemon, run_daemon_with_domain_events, DaemonError};
pub use agent::{
    build_agent_adapters, AgentConfig, AgentError, AgentLoop, AgentStreamEvent, AgentStreamSink,
    AgentTurnRequest, AgentTurnResult,
};
pub use communication::AgentMessage;
pub use heartbeat::{AgentHeartbeat, BackgroundTaskReport};
pub use manager::AgentManager;
pub use persistent::{
    AgentIdentity, AgentMemoryStore, AgentStatus, AgentStructure, CortexAgentBridge,
    PersistentAgent, PersistentAgentConfig, PersistentAgentError,
};
pub use registry::AgentRegistry;
pub use worker::{run_agent_tick, spawn_agent_tick_if_enabled, AgentTickReport};
pub use harness::{
    ensure_daemon_token, install_scheduled_task, probe_daemon_status, probe_gateway_status,
    probe_health, probe_harness_services, probe_providers, run_configure, run_doctor, run_onboard,
    run_smoke, scheduled_task_installed, service_badges, set_user_env_var, stop_daemon,
    validate_probe, ChannelStatusRow, CheckStatus, ConfigureOptions, DaemonInstallResult,
    DaemonStopResult, DoctorCheck, DoctorReport, HarnessError, HarnessServiceProbe,
    HarnessSmokeOptions, OnboardOptions, OnboardResult, ProviderProbeResult, ServiceHealth,
    ServiceProbeState, ServiceStatusDetail, SmokeResult, SupervisorSpawnPlan, DAEMON_TOKEN_ENV,
    plan_supervisor, spawn_child, wait_for_spawn,
};
#[cfg(feature = "gateway")]
pub use harness::{disable_channel, enable_channel, list_channel_status};
#[cfg(feature = "gateway")]
pub use gateway::{run_gateway, GatewayError, GatewayRunner};
pub use tools::{
    CapabilityProfileDescriptor, CapabilityProfileRegistry, ToolError, ToolRegistry,
    CAPABILITY_PROFILE_DESCRIPTORS,
};
#[cfg(feature = "gateway")]
pub use gateway::ChannelCatalog;
pub use use_cases::{
    AssimilateFromText, ListMemories, SearchMemories, DEFAULT_ASSIMILATION_SYSTEM_PROMPT,
};
pub use b212::{
    ensure_b212_agents, B212AnalyzeRequest, B212WorkflowResult, B212WorkflowService, B212_AGENTS,
    B212GovernanceService,
};

/// Version du crate alignée sur le workspace.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
