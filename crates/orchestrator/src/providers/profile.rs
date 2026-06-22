use std::collections::HashMap;

use serde::Deserialize;

use super::descriptor::ProviderDescriptor;

/// Profil configurable d'un provider (surcharges TOML).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderProfile {
    /// Active ce profil (sinon ignoré à la résolution).
    pub enabled: bool,
    /// Variable d'environnement de la clé API.
    pub api_key_env: String,
    /// Modèle à utiliser.
    pub model: String,
    /// URL de base (OpenAI-compatible, Ollama, Anthropic).
    pub base_url: String,
    /// Timeout HTTP en secondes.
    pub timeout_secs: u64,
    /// Nombre maximal de tentatives.
    pub max_retries: u32,
}

impl ProviderProfile {
    /// Profil par défaut depuis un descripteur catalogue.
    #[must_use]
    pub fn from_descriptor(descriptor: &ProviderDescriptor) -> Self {
        Self {
            enabled: true,
            api_key_env: descriptor.default_api_key_env.to_string(),
            model: descriptor.default_model.to_string(),
            base_url: descriptor.default_base_url.to_string(),
            timeout_secs: 60,
            max_retries: 2,
        }
    }

    /// Fusionne les surcharges TOML sur le profil.
    pub fn merge(&mut self, section: &ProviderProfileSection) {
        if let Some(v) = section.enabled {
            self.enabled = v;
        }
        if let Some(v) = &section.api_key_env {
            self.api_key_env = v.clone();
        }
        if let Some(v) = &section.model {
            self.model = v.clone();
        }
        if let Some(v) = &section.base_url {
            self.base_url = v.clone();
        }
        if let Some(v) = section.timeout_secs {
            self.timeout_secs = v;
        }
        if let Some(v) = section.max_retries {
            self.max_retries = v;
        }
    }
}

/// Section TOML `[provider_profiles.<id>]`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProviderProfileSection {
    /// Active le profil.
    pub enabled: Option<bool>,
    /// Variable d'environnement clé API.
    pub api_key_env: Option<String>,
    /// Modèle.
    pub model: Option<String>,
    /// URL de base.
    pub base_url: Option<String>,
    /// Timeout secondes.
    pub timeout_secs: Option<u64>,
    /// Retries.
    pub max_retries: Option<u32>,
}

/// Profils surchargeables par identifiant provider.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderProfiles {
    profiles: HashMap<String, ProviderProfile>,
}

impl ProviderProfiles {
    /// Profil effectif pour un identifiant (défaut catalogue + surcharges).
    #[must_use]
    pub fn resolve(&self, id: &str, descriptor: &ProviderDescriptor) -> ProviderProfile {
        let mut profile = ProviderProfile::from_descriptor(descriptor);
        if let Some(saved) = self.profiles.get(id) {
            profile.enabled = saved.enabled;
            if !saved.api_key_env.is_empty() {
                profile.api_key_env = saved.api_key_env.clone();
            }
            if !saved.model.is_empty() {
                profile.model = saved.model.clone();
            }
            if !saved.base_url.is_empty() {
                profile.base_url = saved.base_url.clone();
            }
            profile.timeout_secs = saved.timeout_secs;
            profile.max_retries = saved.max_retries;
        }
        profile
    }

    /// Fusionne une section TOML pour un identifiant provider.
    pub fn merge_section(&mut self, id: impl Into<String>, section: ProviderProfileSection) {
        let id = id.into();
        let entry = self.profiles.entry(id).or_insert_with(|| ProviderProfile {
            enabled: true,
            api_key_env: String::new(),
            model: String::new(),
            base_url: String::new(),
            timeout_secs: 60,
            max_retries: 2,
        });
        entry.merge(&section);
    }
}