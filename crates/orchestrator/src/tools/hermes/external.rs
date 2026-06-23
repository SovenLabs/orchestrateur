//! Outils externes Hermess — web, browser, multimédia (stubs configurables).

use std::sync::Arc;

use serde_json::{json, Value};

use super::json_result;
use crate::tools::registry::ToolRegistry;
use crate::tools::tool::{Tool, ToolContext, ToolError, ToolResult};

pub fn register(registry: &mut ToolRegistry) {
    registry.register(Arc::new(WebSearchTool));
    registry.register(Arc::new(BrowserNavigateTool));
    registry.register(Arc::new(OpenPageTool));
    registry.register(Arc::new(ImageGenerateTool));
    registry.register(Arc::new(TextToSpeechTool));
    registry.register(Arc::new(VisionAnalyzeTool));
}

macro_rules! stub_tool {
    ($name:ident, $tool_name:expr, $desc:expr, $schema:expr, $hint:expr) => {
        pub struct $name;

        #[async_trait::async_trait]
        impl Tool for $name {
            fn name(&self) -> &'static str {
                $tool_name
            }
            fn description(&self) -> &'static str {
                $desc
            }
            fn parameters_schema(&self) -> &'static str {
                $schema
            }
            async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
                let _ = ctx;
                let _ = args;
                Ok(ToolResult {
                    content: json_result(&json!({
                        "status": "requires_provider",
                        "tool": $tool_name,
                        "hint": $hint,
                    })),
                })
            }
        }
    };
}

stub_tool!(
    WebSearchTool,
    "web_search",
    "Recherche web (provider externe requis — plugins Hermess tavily/brave/searxng).",
    r#"{"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer"}},"required":["query"]}"#,
    "Installer une skill P6 subprocess ou configurer un provider web (voir Hermess plugins/web/)."
);

stub_tool!(
    BrowserNavigateTool,
    "browser_navigate",
    "Navigation navigateur avec snapshot accessibilité (provider browser requis).",
    r#"{"type":"object","properties":{"url":{"type":"string"}},"required":["url"]}"#,
    "Brancher agent-browser ou Browserbase (voir Hermess tools/browser_tool.py)."
);

stub_tool!(
    OpenPageTool,
    "open_page",
    "Alias Hermess — ouvre et lit une page (utiliser browser_navigate ou web_extract).",
    r#"{"type":"object","properties":{"url":{"type":"string"}},"required":["url"]}"#,
    "Équivalent open_page → browser_navigate dans Hermess."
);

stub_tool!(
    ImageGenerateTool,
    "image_generate",
    "Génération d'images text-to-image (FAL / gateway requis).",
    r#"{"type":"object","properties":{"prompt":{"type":"string"},"aspect_ratio":{"type":"string"}},"required":["prompt"]}"#,
    "Configurer plugins/image_gen (Hermess) ou skill subprocess locale."
);

stub_tool!(
    TextToSpeechTool,
    "text_to_speech",
    "Synthèse vocale (Edge TTS, ElevenLabs, OpenAI, …).",
    r#"{"type":"object","properties":{"text":{"type":"string"},"output_path":{"type":"string"}},"required":["text"]}"#,
    "Configurer agent/tts_registry (Hermess) ou provider TTS local."
);

stub_tool!(
    VisionAnalyzeTool,
    "vision_analyze",
    "Analyse d'image via modèle vision (auxiliary client requis).",
    r#"{"type":"object","properties":{"image_url":{"type":"string"},"question":{"type":"string"}},"required":["image_url"]}"#,
    "Utiliser un LLM multimodal primaire ou auxiliary vision (Hermess vision_tools.py)."
);