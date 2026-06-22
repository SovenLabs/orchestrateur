/// Famille d'API HTTP du provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiFamily {
    /// API OpenAI `/v1/chat/completions`.
    OpenAiCompatible,
    /// API xAI (OpenAI-compatible, endpoint dédié).
    Xai,
    /// API Ollama locale.
    Ollama,
    /// API Anthropic Messages.
    Anthropic,
    /// API OpenAI `/v1/embeddings`.
    OpenAiEmbeddings,
    /// API Ollama `/api/embeddings`.
    OllamaEmbeddings,
}

/// Type de provider (LLM ou embeddings).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    /// Génération LLM.
    Llm,
    /// Embeddings vectoriels.
    Embedding,
}

/// Descripteur statique d'un provider enregistré.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderDescriptor {
    /// Identifiant TOML (`openai`, `groq`, …).
    pub id: &'static str,
    /// Nom affiché.
    pub display_name: &'static str,
    /// LLM ou embedding.
    pub kind: ProviderKind,
    /// Famille d'API.
    pub api_family: ApiFamily,
    /// URL de base par défaut (LLM OpenAI-compatible).
    pub default_base_url: &'static str,
    /// Modèle par défaut.
    pub default_model: &'static str,
    /// Variable d'environnement de la clé API.
    pub default_api_key_env: &'static str,
}

/// Catalogue LLM — 12 providers Phase 9.
pub const LLM_DESCRIPTORS: &[ProviderDescriptor] = &[
    ProviderDescriptor {
        id: "xai",
        display_name: "xAI Grok",
        kind: ProviderKind::Llm,
        api_family: ApiFamily::Xai,
        default_base_url: "https://api.x.ai/v1",
        default_model: "grok-3-latest",
        default_api_key_env: "XAI_API_KEY",
    },
    ProviderDescriptor {
        id: "ollama",
        display_name: "Ollama (local)",
        kind: ProviderKind::Llm,
        api_family: ApiFamily::Ollama,
        default_base_url: "http://127.0.0.1:11434",
        default_model: "qwen3:8b",
        default_api_key_env: "",
    },
    ProviderDescriptor {
        id: "openai",
        display_name: "OpenAI",
        kind: ProviderKind::Llm,
        api_family: ApiFamily::OpenAiCompatible,
        default_base_url: "https://api.openai.com/v1",
        default_model: "gpt-4o",
        default_api_key_env: "OPENAI_API_KEY",
    },
    ProviderDescriptor {
        id: "anthropic",
        display_name: "Anthropic Claude",
        kind: ProviderKind::Llm,
        api_family: ApiFamily::Anthropic,
        default_base_url: "https://api.anthropic.com",
        default_model: "claude-3-5-sonnet-20241022",
        default_api_key_env: "ANTHROPIC_API_KEY",
    },
    ProviderDescriptor {
        id: "groq",
        display_name: "Groq",
        kind: ProviderKind::Llm,
        api_family: ApiFamily::OpenAiCompatible,
        default_base_url: "https://api.groq.com/openai/v1",
        default_model: "llama-3.3-70b-versatile",
        default_api_key_env: "GROQ_API_KEY",
    },
    ProviderDescriptor {
        id: "openrouter",
        display_name: "OpenRouter",
        kind: ProviderKind::Llm,
        api_family: ApiFamily::OpenAiCompatible,
        default_base_url: "https://openrouter.ai/api/v1",
        default_model: "openai/gpt-4o",
        default_api_key_env: "OPENROUTER_API_KEY",
    },
    ProviderDescriptor {
        id: "together",
        display_name: "Together AI",
        kind: ProviderKind::Llm,
        api_family: ApiFamily::OpenAiCompatible,
        default_base_url: "https://api.together.xyz/v1",
        default_model: "meta-llama/Llama-3.3-70B-Instruct-Turbo",
        default_api_key_env: "TOGETHER_API_KEY",
    },
    ProviderDescriptor {
        id: "deepseek",
        display_name: "DeepSeek",
        kind: ProviderKind::Llm,
        api_family: ApiFamily::OpenAiCompatible,
        default_base_url: "https://api.deepseek.com/v1",
        default_model: "deepseek-chat",
        default_api_key_env: "DEEPSEEK_API_KEY",
    },
    ProviderDescriptor {
        id: "mistral",
        display_name: "Mistral AI",
        kind: ProviderKind::Llm,
        api_family: ApiFamily::OpenAiCompatible,
        default_base_url: "https://api.mistral.ai/v1",
        default_model: "mistral-large-latest",
        default_api_key_env: "MISTRAL_API_KEY",
    },
    ProviderDescriptor {
        id: "perplexity",
        display_name: "Perplexity",
        kind: ProviderKind::Llm,
        api_family: ApiFamily::OpenAiCompatible,
        default_base_url: "https://api.perplexity.ai",
        default_model: "sonar",
        default_api_key_env: "PERPLEXITY_API_KEY",
    },
    ProviderDescriptor {
        id: "lmstudio",
        display_name: "LM Studio (local)",
        kind: ProviderKind::Llm,
        api_family: ApiFamily::OpenAiCompatible,
        default_base_url: "http://127.0.0.1:1234/v1",
        default_model: "local-model",
        default_api_key_env: "LMSTUDIO_API_KEY",
    },
    ProviderDescriptor {
        id: "azure_openai",
        display_name: "Azure OpenAI",
        kind: ProviderKind::Llm,
        api_family: ApiFamily::OpenAiCompatible,
        default_base_url: "https://example.openai.azure.com/openai/deployments/gpt-4o",
        default_model: "gpt-4o",
        default_api_key_env: "AZURE_OPENAI_API_KEY",
    },
];

/// Catalogue embeddings — 5 providers Phase 9.
pub const EMBEDDING_DESCRIPTORS: &[ProviderDescriptor] = &[
    ProviderDescriptor {
        id: "ollama",
        display_name: "Ollama Embeddings",
        kind: ProviderKind::Embedding,
        api_family: ApiFamily::OllamaEmbeddings,
        default_base_url: "http://127.0.0.1:11434",
        default_model: "qwen3-embedding:8b",
        default_api_key_env: "",
    },
    ProviderDescriptor {
        id: "openai",
        display_name: "OpenAI Embeddings",
        kind: ProviderKind::Embedding,
        api_family: ApiFamily::OpenAiEmbeddings,
        default_base_url: "https://api.openai.com/v1",
        default_model: "text-embedding-3-small",
        default_api_key_env: "OPENAI_API_KEY",
    },
    ProviderDescriptor {
        id: "voyage",
        display_name: "Voyage AI",
        kind: ProviderKind::Embedding,
        api_family: ApiFamily::OpenAiEmbeddings,
        default_base_url: "https://api.voyageai.com/v1",
        default_model: "voyage-3",
        default_api_key_env: "VOYAGE_API_KEY",
    },
    ProviderDescriptor {
        id: "huggingface",
        display_name: "HuggingFace Inference",
        kind: ProviderKind::Embedding,
        api_family: ApiFamily::OpenAiEmbeddings,
        default_base_url: "https://api-inference.huggingface.co",
        default_model: "sentence-transformers/all-MiniLM-L6-v2",
        default_api_key_env: "HUGGINGFACE_API_KEY",
    },
    ProviderDescriptor {
        id: "fastembed",
        display_name: "FastEmbed (local stub)",
        kind: ProviderKind::Embedding,
        api_family: ApiFamily::OllamaEmbeddings,
        default_base_url: "http://127.0.0.1:11434",
        default_model: "nomic-embed-text",
        default_api_key_env: "",
    },
];