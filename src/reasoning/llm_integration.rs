//! LLM Integration: Ollama-backed explanation generation, with a
//! template-based fallback.
//!
//! The `Ollama` backend makes a real HTTP call to a local Ollama server
//! (`http://localhost:11434` by default, overridable via the
//! `OLLAMA_BASE_URL` environment variable or [`LLMExplainer::with_ollama_base_url`])
//! using its `/api/generate` REST endpoint. `is_from_llm` is only ever set to
//! `true` when a real model response came back from Ollama; any failure to
//! reach Ollama (not installed, server not running, model not pulled, etc.)
//! falls back to the template-based generator so callers never hard-fail
//! just because a user doesn't have Ollama set up.
//!
//! Other backend variants (`LlamaCpp`, `Huggingface`, `LocalPython`) are not
//! implemented and also fall back to the template generator; `explain_*`
//! callers should not assume every non-`Fallback` backend produces a real
//! LLM response - check `is_from_llm` on the returned [`LLMExplanation`].
//!
//! See `scripts/test_ollama_integration.sh` for a script that pulls a small
//! model and runs the real (`#[ignore]`d) integration test in
//! `tests/test_ollama_integration.rs`.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// LLM-generated explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMExplanation {
    /// The generated explanation
    pub text: String,

    /// Confidence in explanation (0.0-1.0)
    pub confidence: f32,

    /// Which LLM generated this
    pub model: String,

    /// Inference time (milliseconds)
    pub inference_time_ms: f32,

    /// Whether this came from LLM or fallback
    pub is_from_llm: bool,

    /// Which backend actually produced this explanation. Reflects what
    /// really ran, not what was requested - e.g. an `Ollama`-configured
    /// explainer that couldn't reach the server reports `Fallback` here.
    pub backend: InferenceBackend,
}

/// Configuration for LLM inference
#[derive(Debug, Clone)]
pub struct LLMConfig {
    /// Model name (phi-2, mistral-7b, etc.)
    pub model_name: String,

    /// Model size (in billions of parameters)
    pub model_size_b: f32,

    /// Inference backend (llama.cpp, ollama, etc.)
    pub backend: InferenceBackend,

    /// Context window size (tokens)
    pub context_window: usize,

    /// Max tokens to generate
    pub max_tokens: usize,

    /// Temperature (creativity 0.0-1.0)
    pub temperature: f32,

    /// Whether to use LLM or fallback only
    pub enabled: bool,
}

/// Supported inference backends
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InferenceBackend {
    LlamaCpp,      // llama.cpp (local inference) - not implemented, falls back
    Ollama,        // Ollama (local) - real HTTP integration
    Huggingface,   // Hugging Face Inference API - not implemented, falls back
    LocalPython,   // Direct Python integration - not implemented, falls back
    Fallback,      // Template-based (no LLM)
}

impl std::fmt::Display for InferenceBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InferenceBackend::LlamaCpp => write!(f, "llama.cpp"),
            InferenceBackend::Ollama => write!(f, "Ollama"),
            InferenceBackend::Huggingface => write!(f, "Hugging Face"),
            InferenceBackend::LocalPython => write!(f, "Local Python"),
            InferenceBackend::Fallback => write!(f, "Fallback"),
        }
    }
}

/// Default Ollama server URL, used unless overridden by the
/// `OLLAMA_BASE_URL` environment variable or [`LLMExplainer::with_ollama_base_url`].
const DEFAULT_OLLAMA_BASE_URL: &str = "http://localhost:11434";

fn default_ollama_base_url() -> String {
    std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| DEFAULT_OLLAMA_BASE_URL.to_string())
}

/// LLM-based explainer for robot incidents
pub struct LLMExplainer {
    config: LLMConfig,
    prompt_cache: std::collections::HashMap<String, String>,
    ollama_base_url: String,
    /// Timeout for the HTTP call to Ollama. Kept short by default so a
    /// hung/unreachable server degrades to the template fallback quickly
    /// instead of blocking the caller.
    ollama_timeout: Duration,
}

impl LLMExplainer {
    /// Create new LLM explainer
    pub fn new(config: LLMConfig) -> Self {
        LLMExplainer {
            config,
            prompt_cache: std::collections::HashMap::new(),
            ollama_base_url: default_ollama_base_url(),
            ollama_timeout: Duration::from_secs(30),
        }
    }

    /// Create with defaults (fallback mode)
    pub fn new_fallback() -> Self {
        LLMExplainer {
            config: LLMConfig {
                model_name: "phi-2".to_string(),
                model_size_b: 3.8,
                backend: InferenceBackend::Fallback,
                context_window: 2048,
                max_tokens: 512,
                temperature: 0.7,
                enabled: false,
            },
            prompt_cache: std::collections::HashMap::new(),
            ollama_base_url: default_ollama_base_url(),
            ollama_timeout: Duration::from_secs(30),
        }
    }

    /// Create an explainer that generates via a real local Ollama server.
    /// `model_name` must be a model tag Ollama already has pulled (check
    /// with `ollama list`; pull one with e.g. `ollama pull llama3.2:1b`).
    ///
    /// If Ollama is unreachable or the model isn't available at call time,
    /// [`Self::explain_incident`] (and friends) transparently fall back to
    /// the template-based generator rather than failing - always check
    /// `LLMExplanation::is_from_llm` if the caller cares which path ran.
    pub fn new_ollama(model_name: impl Into<String>) -> Self {
        LLMExplainer {
            config: LLMConfig {
                model_name: model_name.into(),
                model_size_b: 0.0, // unknown/managed by Ollama itself
                backend: InferenceBackend::Ollama,
                context_window: 2048,
                max_tokens: 512,
                temperature: 0.7,
                enabled: true,
            },
            prompt_cache: std::collections::HashMap::new(),
            ollama_base_url: default_ollama_base_url(),
            ollama_timeout: Duration::from_secs(30),
        }
    }

    /// Override the Ollama server base URL (default: `http://localhost:11434`,
    /// or the `OLLAMA_BASE_URL` environment variable if set).
    pub fn with_ollama_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.ollama_base_url = base_url.into();
        self
    }

    /// Override the HTTP timeout for calls to Ollama (default: 30s).
    pub fn with_ollama_timeout(mut self, timeout: Duration) -> Self {
        self.ollama_timeout = timeout;
        self
    }

    /// Generate explanation for incident
    pub fn explain_incident(
        &mut self,
        incident_summary: &str,
        perception_gaps: &[String],
        robot_behavior: &str,
        confidence: f32,
    ) -> LLMExplanation {
        let prompt = self.build_prompt(
            incident_summary,
            perception_gaps,
            robot_behavior,
            confidence,
        );

        if self.config.enabled && self.config.backend != InferenceBackend::Fallback {
            self.generate_via_llm(&prompt)
        } else {
            self.generate_fallback(&prompt)
        }
    }

    /// Build prompt for LLM
    fn build_prompt(
        &self,
        incident_summary: &str,
        perception_gaps: &[String],
        robot_behavior: &str,
        _confidence: f32,
    ) -> String {
        format!(
            "As a robot debugging expert, analyze this incident:\n\n\
             Incident: {}\n\n\
             Perception Gaps:\n{}\n\n\
             Robot Behavior: {}\n\n\
             Provide a concise root cause analysis and recommendation.",
            incident_summary,
            perception_gaps
                .iter()
                .map(|g| format!("- {}", g))
                .collect::<Vec<_>>()
                .join("\n"),
            robot_behavior
        )
    }

    /// Generate via a real LLM backend. Currently only `Ollama` is actually
    /// implemented; other backend variants fall back to the template
    /// generator (with a log warning) rather than pretending to call an
    /// unimplemented API.
    fn generate_via_llm(&self, prompt: &str) -> LLMExplanation {
        match self.config.backend {
            InferenceBackend::Ollama => self.generate_via_ollama(prompt),
            other => {
                tracing::warn!(
                    "LLM backend {} is not implemented; using template-based fallback",
                    other
                );
                self.generate_fallback(prompt)
            }
        }
    }

    /// Call a local Ollama server's `/api/generate` REST endpoint for real.
    /// Falls back to the template generator (clearly labeled, `is_from_llm:
    /// false`) on any network/parse/model error so a missing/unreachable
    /// Ollama installation never hard-fails the caller.
    fn generate_via_ollama(&self, prompt: &str) -> LLMExplanation {
        let start = Instant::now();
        match Self::call_ollama_generate(
            &self.ollama_base_url,
            self.ollama_timeout,
            &self.config.model_name,
            prompt,
            self.config.temperature,
            self.config.max_tokens,
        ) {
            Ok(text) => LLMExplanation {
                text,
                confidence: 0.85,
                model: self.config.model_name.clone(),
                inference_time_ms: start.elapsed().as_secs_f32() * 1000.0,
                is_from_llm: true,
                backend: InferenceBackend::Ollama,
            },
            Err(e) => {
                tracing::warn!(
                    "Ollama request failed ({}); falling back to template-based generation",
                    e
                );
                self.generate_fallback(prompt)
            }
        }
    }

    /// Perform the actual blocking HTTP call to Ollama's `/api/generate`
    /// endpoint (non-streaming) and return the generated text.
    fn call_ollama_generate(
        base_url: &str,
        timeout: Duration,
        model: &str,
        prompt: &str,
        temperature: f32,
        max_tokens: usize,
    ) -> Result<String, String> {
        #[derive(Serialize)]
        struct OllamaOptions {
            temperature: f32,
            num_predict: i64,
        }

        #[derive(Serialize)]
        struct OllamaGenerateRequest<'a> {
            model: &'a str,
            prompt: &'a str,
            stream: bool,
            options: OllamaOptions,
        }

        #[derive(Deserialize)]
        struct OllamaGenerateResponse {
            #[serde(default)]
            response: String,
            #[serde(default)]
            error: Option<String>,
        }

        let url = format!("{}/api/generate", base_url.trim_end_matches('/'));
        let request_body = OllamaGenerateRequest {
            model,
            prompt,
            stream: false,
            options: OllamaOptions {
                temperature,
                num_predict: max_tokens as i64,
            },
        };

        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            .build()
            .new_agent();

        let mut response = agent
            .post(&url)
            .send_json(&request_body)
            .map_err(|e| format!("failed to reach Ollama at {}: {}", url, e))?;

        let parsed: OllamaGenerateResponse = response
            .body_mut()
            .read_json()
            .map_err(|e| format!("failed to parse Ollama response: {}", e))?;

        if let Some(err) = parsed.error {
            return Err(format!("Ollama returned an error: {}", err));
        }
        if parsed.response.trim().is_empty() {
            return Err("Ollama returned an empty response (model may not be pulled)".to_string());
        }

        Ok(parsed.response)
    }

    /// Generate using template fallback (always works, no LLM)
    fn generate_fallback(&self, prompt: &str) -> LLMExplanation {
        let explanation = Self::template_explanation(prompt);

        LLMExplanation {
            text: explanation,
            confidence: 0.65, // Lower confidence for template-based
            model: "template-based".to_string(),
            inference_time_ms: 0.0,
            is_from_llm: false,
            backend: InferenceBackend::Fallback,
        }
    }

    /// Template-based explanation (works without LLM)
    fn template_explanation(prompt: &str) -> String {
        // Extract key phrases from prompt for template filling
        let has_perception_gap = prompt.to_lowercase().contains("perception");
        let has_sensor = prompt.to_lowercase().contains("sensor");
        let has_collision = prompt.to_lowercase().contains("collision");

        let mut explanation = String::new();

        explanation.push_str("ROOT CAUSE ANALYSIS\n\n");

        if has_perception_gap {
            explanation.push_str("Issue: Robot perception limitation\n\n");
            explanation.push_str(
                "The robot lacked complete environmental understanding during operation. \
                 Analysis of replay footage reveals objects or conditions that were not \
                 perceived by the robot's onboard sensors.\n\n",
            );
        }

        if has_sensor {
            explanation.push_str("Contributing Factor: Sensor Configuration\n");
            explanation.push_str(
                "The robot's sensor suite had limited range or field of view for certain \
                 object types or positions.\n\n",
            );
        }

        if has_collision {
            explanation.push_str("Impact: Collision Risk\n");
            explanation.push_str(
                "The perception gap directly contributed to the collision by allowing \
                 the robot to maintain its trajectory into an obstacle.\n\n",
            );
        }

        explanation.push_str("RECOMMENDATION\n\n");
        explanation.push_str(
            "1. Add sensor coverage for the identified blind spot\n\
             2. Implement camera-based detection as supplementary safety\n\
             3. Increase detection timeout and safety margins\n\
             4. Re-test in similar environment before deployment\n",
        );

        explanation
    }

    /// Generate explanation for perception gap
    pub fn explain_perception_gap(
        &mut self,
        gap_description: &str,
        impact: &str,
    ) -> LLMExplanation {
        let prompt = format!(
            "Explain this robot perception gap:\n\n{}\n\nImpact on behavior: {}",
            gap_description, impact
        );

        if self.config.enabled && self.config.backend != InferenceBackend::Fallback {
            self.generate_via_llm(&prompt)
        } else {
            self.generate_fallback(&prompt)
        }
    }

    /// Generate recommendation for preventing recurrence
    pub fn recommend_fix(
        &mut self,
        root_cause: &str,
        failure_mode: &str,
    ) -> LLMExplanation {
        let prompt = format!(
            "Given this robot failure:\n\nRoot cause: {}\nFailure mode: {}\n\n\
             What are the top 3 engineering recommendations to prevent recurrence?",
            root_cause, failure_mode
        );

        if self.config.enabled && self.config.backend != InferenceBackend::Fallback {
            self.generate_via_llm(&prompt)
        } else {
            self.generate_fallback(&prompt)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_creation() {
        let config = LLMConfig {
            model_name: "phi-2".to_string(),
            model_size_b: 3.8,
            backend: InferenceBackend::LlamaCpp,
            context_window: 2048,
            max_tokens: 512,
            temperature: 0.7,
            enabled: true,
        };

        assert_eq!(config.model_name, "phi-2");
        assert_eq!(config.backend, InferenceBackend::LlamaCpp);
    }

    #[test]
    fn test_fallback_explainer() {
        let mut explainer = LLMExplainer::new_fallback();

        let explanation = explainer.explain_incident(
            "Robot collided with pallet",
            &["Pallet outside sensor range".to_string()],
            "forward_movement",
            0.85,
        );

        assert!(!explanation.text.is_empty());
        assert_eq!(explanation.backend, InferenceBackend::Fallback);
    }

    #[test]
    fn test_perception_gap_explanation() {
        let mut explainer = LLMExplainer::new_fallback();

        let explanation = explainer.explain_perception_gap(
            "Object visible in camera but outside ultrasonic range",
            "Robot maintained forward trajectory into obstacle",
        );

        assert_eq!(explanation.is_from_llm, false); // Fallback mode
        assert!(explanation.text.contains("perception"));
    }

    #[test]
    fn test_unimplemented_backend_falls_back_instead_of_faking_llm_output() {
        // LlamaCpp/Huggingface/LocalPython are not implemented; requesting
        // them must not silently claim `is_from_llm: true`.
        let mut explainer = LLMExplainer::new(LLMConfig {
            model_name: "phi-2".to_string(),
            model_size_b: 3.8,
            backend: InferenceBackend::LlamaCpp,
            context_window: 2048,
            max_tokens: 512,
            temperature: 0.7,
            enabled: true,
        });

        let explanation = explainer.explain_incident(
            "Robot collided with pallet",
            &["Pallet outside sensor range".to_string()],
            "forward_movement",
            0.85,
        );

        assert!(!explanation.is_from_llm);
        assert_eq!(explanation.backend, InferenceBackend::Fallback);
        assert_eq!(explanation.model, "template-based");
    }

    #[test]
    fn test_ollama_unreachable_falls_back_gracefully() {
        // Point at a port nothing is listening on so the request fails fast
        // and deterministically, proving the graceful-fallback path works
        // without requiring a real Ollama server for this (non-ignored) test.
        let mut explainer = LLMExplainer::new_ollama("llama3.2:1b")
            .with_ollama_base_url("http://127.0.0.1:1")
            .with_ollama_timeout(std::time::Duration::from_millis(500));

        let explanation = explainer.explain_incident(
            "Robot collided with pallet",
            &["Pallet outside sensor range".to_string()],
            "forward_movement",
            0.85,
        );

        assert!(!explanation.is_from_llm);
        assert_eq!(explanation.backend, InferenceBackend::Fallback);
        assert_eq!(explanation.model, "template-based");
    }
}
