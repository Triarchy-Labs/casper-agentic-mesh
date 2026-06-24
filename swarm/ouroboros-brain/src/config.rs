//! Config loader — читает TOML файлы промптов и моделей.
//! Меняй config/*.toml без перекомпиляции!

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

// ═══════════════════════════════════════════════════════════
// PROMPT CONFIG
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize)]
pub struct PromptConfig {
    pub system: String,
    pub user_template: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default)]
    pub temperature: f32,
}

fn default_max_tokens() -> u32 { 150 }

#[derive(Debug, Clone, Deserialize)]
pub struct DebatePrompts {
    pub bull: PromptConfig,
    pub bear: PromptConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PromptsFile {
    pub debate: DebatePrompts,
    pub macro_judge: PromptConfig,
    pub meta_judge: PromptConfig,
    pub reflection: Option<HashMap<String, PromptConfig>>,
}

// ═══════════════════════════════════════════════════════════
// MODEL CONFIG
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub title: String,
    pub vendor: String,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default)]
    pub priority: u32,
}

fn default_timeout() -> u64 { 30000 }

#[derive(Debug, Clone, Deserialize)]
pub struct ModelsDefaults {
    pub api_base: String,
    pub referer: String,
    pub app_title: String,
    #[serde(default = "default_max_failures")]
    pub max_failures_before_rotate: u32,
}

fn default_max_failures() -> u32 { 2 }

#[derive(Debug, Clone, Deserialize)]
pub struct ModelsFile {
    pub defaults: ModelsDefaults,
    pub debate_pool: Vec<ModelConfig>,
    pub macro_judge_model: ModelConfig,
    pub meta_judge_model: ModelConfig,
}

// ═══════════════════════════════════════════════════════════
// LOADING
// ═══════════════════════════════════════════════════════════

pub fn load_prompts(path: &Path) -> Result<PromptsFile, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let prompts: PromptsFile = toml::from_str(&content)?;
    tracing::info!("Loaded {} prompt configs from {:?}", 4, path);
    Ok(prompts)
}

pub fn load_models(path: &Path) -> Result<ModelsFile, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let models: ModelsFile = toml::from_str(&content)?;
    tracing::info!(
        "Loaded {} debate models + 3 judges from {:?}",
        models.debate_pool.len(), path
    );
    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_load_prompts() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/prompts.toml");
        let prompts = load_prompts(&path).expect("Failed to load prompts.toml");
        assert!(!prompts.debate.bull.user_template.is_empty());
        assert!(!prompts.debate.bear.user_template.is_empty());
        assert!(!prompts.macro_judge.system.is_empty());
    }

    #[test]
    fn test_load_models() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/models.toml");
        let models = load_models(&path).expect("Failed to load models.toml");
        assert_eq!(models.debate_pool.len(), 3);
        assert!(!models.macro_judge_model.id.is_empty());
        assert!(!models.meta_judge_model.id.is_empty());
    }
}
