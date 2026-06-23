//! Mémoire opérationnelle — dédup, prompts d'extraction.

mod dedup;
mod insight;

pub use dedup::{dedup_jaccard_score, is_duplicate_draft};
pub use insight::{
    build_insight_user_prompt, generate_insight_draft, parse_insight_response,
    INSIGHT_ASSIMILATION_SYSTEM_PROMPT,
};