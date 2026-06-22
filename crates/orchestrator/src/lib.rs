//! # Orchestrateur — L'Esprit
//!
//! Couche application : facade, use cases, Skill Registry.
//! Ne dépend que du squelette [`cortex`] et de ses ports.
//!
//! Garde-fous : Rust stable uniquement — pas de `#![feature(...)]`.

#![forbid(unsafe_code)]
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
/// Sondes de disponibilité providers (health bridge).
pub mod health;
/// Ports LLM — génération de [`MemoryDraft`] et chat.
pub mod llm;
/// Brouillon structuré issu des providers IA (`MemoryDraft`).
pub mod memory_draft;
/// Défense en profondeur — validation adversariale des sorties LLM.
pub mod security;
/// Squelette Skills (trait, registre, noop).
pub mod skills;
/// Daemon WebSocket local pour clients visuels (feature `websocket-server`).
#[cfg(feature = "websocket-server")]
pub mod daemon;
/// Boucle agent Phase 7.
pub mod agent;
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
    OrchestratorHandle, OrchestratorThread, Response, SkillSummary,
};
pub use config::{
    AgentSettingsConfig, AuditConfig, BehavioralConfig, ConfigError, DaemonConfig,
    GatewayChannelConfig, GatewayConfig, IntegrityConfig, McpConfig, McpServerConfig, OllamaConfig,
    OrchestratorConfig,
    ProvidersConfig, SecurityConfig, SkillsHubConfig, SkillsHubEntryConfig, VectorStoreConfig,
    XaiConfig,
};
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
pub use security::{
    build_security_context, build_test_security_context, AuditEvent, BehavioralError,
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
    AgentConfig, AgentError, AgentLoop, AgentStreamEvent, AgentStreamSink, AgentTurnRequest,
    AgentTurnResult,
};
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

/// Version du crate alignée sur le workspace.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
