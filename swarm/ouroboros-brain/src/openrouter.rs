//! OpenRouter API клиент с retry, pool rotation и 429 handling.
//! Замена LangChain: 4 строки reqwest вместо 10K строк wrapper.

use crate::config::{ModelConfig, ModelsDefaults};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::time::Duration;

// ═══════════════════════════════════════════════════════════
// API TYPES
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize, Clone)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    content: String,
}

// ═══════════════════════════════════════════════════════════
// MODEL POOL — Lock-free rotation
// ═══════════════════════════════════════════════════════════

pub struct ModelPool {
    models: Vec<ModelConfig>,
    current_idx: AtomicUsize,
    consecutive_failures: AtomicU32,
    max_failures: u32,
    health: std::sync::Mutex<std::collections::HashMap<String, ModelHealth>>,
}

/// Per-model health tracking.
#[derive(Debug, Clone)]
pub struct ModelHealth {
    pub total_calls: u64,
    pub total_errors: u64,
    pub avg_latency_ms: f64,
    pub last_success_ts: i64,
    pub last_error: Option<String>,
}

impl ModelPool {
    pub fn new(models: Vec<ModelConfig>, max_failures: u32) -> Self {
        Self {
            models,
            current_idx: AtomicUsize::new(0),
            consecutive_failures: AtomicU32::new(0),
            max_failures,
            health: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Получить текущую активную модель.
    pub fn current(&self) -> &ModelConfig {
        let idx = self.current_idx.load(Ordering::Relaxed) % self.models.len();
        &self.models[idx]
    }

    /// Зарегистрировать успех — сбросить счётчик ошибок.
    pub fn report_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
    }

    /// Зарегистрировать успех с latency для health tracking.
    pub fn report_success_with_latency(&self, model_id: &str, latency_ms: u64) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
        if let Ok(mut health) = self.health.lock() {
            let h = health.entry(model_id.to_string()).or_insert(ModelHealth {
                total_calls: 0, total_errors: 0, avg_latency_ms: 0.0,
                last_success_ts: 0, last_error: None,
            });
            h.total_calls += 1;
            h.avg_latency_ms = (h.avg_latency_ms * (h.total_calls - 1) as f64 + latency_ms as f64) / h.total_calls as f64;
            h.last_success_ts = chrono::Utc::now().timestamp();
        }
    }

    /// Зарегистрировать ошибку с описанием.
    pub fn report_failure_with_reason(&self, model_id: &str, reason: &str) -> &ModelConfig {
        if let Ok(mut health) = self.health.lock() {
            let h = health.entry(model_id.to_string()).or_insert(ModelHealth {
                total_calls: 0, total_errors: 0, avg_latency_ms: 0.0,
                last_success_ts: 0, last_error: None,
            });
            h.total_calls += 1;
            h.total_errors += 1;
            h.last_error = Some(reason.to_string());
        }
        self.report_failure()
    }

    /// Получить health snapshot всех моделей.
    pub fn health_snapshot(&self) -> Vec<(String, ModelHealth)> {
        if let Ok(health) = self.health.lock() {
            health.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        } else {
            vec![]
        }
    }

    /// Зарегистрировать ошибку — ротировать если превышен лимит.
    pub fn report_failure(&self) -> &ModelConfig {
        let fails = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        if fails >= self.max_failures {
            let old_idx = self.current_idx.load(Ordering::Relaxed);
            let new_idx = (old_idx + 1) % self.models.len();
            self.current_idx.store(new_idx, Ordering::Relaxed);
            self.consecutive_failures.store(0, Ordering::Relaxed);
            tracing::warn!(
                "🔄 POOL ROTATE: [{}] → [{}] (after {} failures)",
                self.models[old_idx % self.models.len()].title,
                self.models[new_idx].title,
                fails
            );
        }
        self.current()
    }

    pub fn pool_size(&self) -> usize {
        self.models.len()
    }
}

// ═══════════════════════════════════════════════════════════
// OPENROUTER CLIENT
// ═══════════════════════════════════════════════════════════

pub struct OpenRouterClient {
    client: Client,
    api_key: String,
    api_base: String,
    referer: String,
    app_title: String,
}

#[derive(Debug)]
pub enum LlmError {
    RateLimited,
    Timeout,
    ApiError(String),
    ParseError(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::RateLimited => write!(f, "429 Rate Limited"),
            LlmError::Timeout => write!(f, "Request Timeout"),
            LlmError::ApiError(e) => write!(f, "API Error: {e}"),
            LlmError::ParseError(e) => write!(f, "Parse Error: {e}"),
        }
    }
}

impl OpenRouterClient {
    pub fn new(api_key: String, defaults: &ModelsDefaults) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            api_key,
            api_base: defaults.api_base.clone(),
            referer: defaults.referer.clone(),
            app_title: defaults.app_title.clone(),
        }
    }

    /// Основной метод: отправить промпт на указанную модель.
    pub async fn chat(
        &self,
        model: &ModelConfig,
        system: &str,
        user: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> Result<String, LlmError> {
        let payload = ChatRequest {
            model: model.id.clone(),
            messages: vec![
                Message { role: "system".into(), content: system.into() },
                Message { role: "user".into(), content: user.into() },
            ],
            temperature,
            max_tokens,
            response_format: None,
        };

        let timeout = Duration::from_millis(model.timeout_ms);

        let response = self.client
            .post(&self.api_base)
            .bearer_auth(&self.api_key)
            .header("HTTP-Referer", &self.referer)
            .header("X-Title", &self.app_title)
            .timeout(timeout)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmError::Timeout
                } else {
                    LlmError::ApiError(e.to_string())
                }
            })?;

        let status = response.status();

        if status.as_u16() == 429 {
            tracing::warn!("⚠️ 429 RATE LIMITED on [{}]", model.title);
            return Err(LlmError::RateLimited);
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(format!("{status}: {body}")));
        }

        let chat_resp: ChatResponse = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        chat_resp
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .ok_or_else(|| LlmError::ParseError("Empty choices array".into()))
    }

    /// Отправить с автоматическим retry через pool.
    pub async fn chat_with_pool(
        &self,
        pool: &ModelPool,
        system: &str,
        user: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> Result<String, LlmError> {
        let max_attempts = pool.pool_size();
        let mut last_error = LlmError::ApiError("No models available".into());

        for attempt in 0..max_attempts {
            let model = pool.current();
            tracing::debug!("[attempt {}/{}] Trying [{}]", attempt + 1, max_attempts, model.title);

            let start = std::time::Instant::now();
            match self.chat(model, system, user, temperature, max_tokens).await {
                Ok(response) => {
                    let latency = start.elapsed().as_millis() as u64;
                    pool.report_success_with_latency(&model.id, latency);
                    return Ok(response);
                }
                Err(e) => {
                    let reason = format!("{e}");
                    tracing::warn!("[{}] failed: {} (attempt {}/{})", model.title, e, attempt + 1, max_attempts);
                    pool.report_failure_with_reason(&model.id, &reason);
                    last_error = e;
                    // Exponential backoff: 500ms → 1s → 2s → 4s
                    let backoff_ms = 500u64 * (1u64 << attempt.min(3));
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                }
            }
        }

        Err(last_error)
    }

    /// Отправить запрос с принудительным JSON output.
    /// Модели которые поддерживают `response_format: json_object`
    /// вернут чистый JSON без markdown обёртки.
    pub async fn chat_json(
        &self,
        model: &ModelConfig,
        system: &str,
        user: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> Result<String, LlmError> {
        let payload = ChatRequest {
            model: model.id.clone(),
            messages: vec![
                Message { role: "system".into(), content: format!("{system} You MUST respond with valid JSON only.") },
                Message { role: "user".into(), content: user.into() },
            ],
            temperature,
            max_tokens,
            response_format: Some(ResponseFormat {
                format_type: "json_object".to_string(),
            }),
        };

        let timeout = Duration::from_millis(model.timeout_ms);

        let response = self.client
            .post(&self.api_base)
            .bearer_auth(&self.api_key)
            .header("HTTP-Referer", &self.referer)
            .header("X-Title", &self.app_title)
            .timeout(timeout)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() { LlmError::Timeout }
                else { LlmError::ApiError(e.to_string()) }
            })?;

        let status = response.status();
        if status.as_u16() == 429 { return Err(LlmError::RateLimited); }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(format!("{status}: {body}")));
        }

        let chat_resp: ChatResponse = response
            .json().await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        chat_resp.choices.first()
            .map(|c| c.message.content.trim().to_string())
            .ok_or_else(|| LlmError::ParseError("Empty choices array".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_rotation() {
        let models = vec![
            ModelConfig {
                id: "model-a".into(), title: "A".into(), vendor: "V1".into(),
                timeout_ms: 1000, max_tokens: 100, priority: 1,
            },
            ModelConfig {
                id: "model-b".into(), title: "B".into(), vendor: "V2".into(),
                timeout_ms: 1000, max_tokens: 100, priority: 2,
            },
        ];
        let pool = ModelPool::new(models, 2);

        assert_eq!(pool.current().title, "A");

        // 1 failure — should NOT rotate
        pool.report_failure();
        assert_eq!(pool.current().title, "A");

        // 2nd failure — should rotate to B
        pool.report_failure();
        assert_eq!(pool.current().title, "B");

        // Success — reset counter
        pool.report_success();
        assert_eq!(pool.current().title, "B"); // stays on B

        // 2 more failures — wrap around to A
        pool.report_failure();
        pool.report_failure();
        assert_eq!(pool.current().title, "A");
    }
}
