//! Real integration test for the Ollama-backed LLM integration in
//! `src/reasoning/llm_integration.rs`.
//!
//! This actually starts (or requires already-running) a local Ollama server
//! and verifies a real model response comes back with `is_from_llm: true`.
//! It's `#[ignore]`d by default so plain `cargo test` never depends on an
//! external service. Run it with:
//!
//!   ./scripts/test_ollama_integration.sh
//!
//! or manually:
//!
//!   ollama serve &
//!   ollama pull qwen2.5:0.5b   # or any small model you prefer
//!   cargo test --test test_ollama_integration -- --ignored --test-threads=1
//!
//! Override the model via `PYROBOREPLAY_TEST_OLLAMA_MODEL` if you've pulled
//! a different one.

use pyroboreplay::reasoning::llm_integration::{InferenceBackend, LLMExplainer};

fn test_model() -> String {
    std::env::var("PYROBOREPLAY_TEST_OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:0.5b".to_string())
}

#[test]
#[ignore]
fn test_ollama_generates_real_explanation() {
    let mut explainer = LLMExplainer::new_ollama(test_model());

    let explanation = explainer.explain_incident(
        "Robot collided with a pallet while navigating the warehouse aisle",
        &["Pallet was outside the ultrasonic sensor's field of view".to_string()],
        "forward_movement_at_0.8m/s",
        0.85,
    );

    assert!(
        explanation.is_from_llm,
        "expected a real Ollama response (is_from_llm=true); got fallback instead - is `ollama serve` \
         running and is the '{}' model pulled? Run scripts/test_ollama_integration.sh",
        test_model()
    );
    assert_eq!(explanation.backend, InferenceBackend::Ollama);
    assert_eq!(explanation.model, test_model());
    assert!(!explanation.text.trim().is_empty(), "Ollama response text should not be empty");
    assert!(explanation.inference_time_ms > 0.0);
}

#[test]
#[ignore]
fn test_ollama_perception_gap_and_recommend_fix_are_real() {
    let mut explainer = LLMExplainer::new_ollama(test_model());

    let gap_explanation = explainer.explain_perception_gap(
        "Object visible in camera but outside ultrasonic range",
        "Robot maintained forward trajectory into obstacle",
    );
    assert!(gap_explanation.is_from_llm);
    assert!(!gap_explanation.text.trim().is_empty());

    let fix_explanation = explainer.recommend_fix(
        "Sensor blind spot at close range",
        "collision_near_miss",
    );
    assert!(fix_explanation.is_from_llm);
    assert!(!fix_explanation.text.trim().is_empty());
}

#[test]
#[ignore]
fn test_ollama_nonexistent_model_falls_back_gracefully() {
    // A real, reachable Ollama server but a model tag that was never
    // pulled must still degrade to the template fallback rather than
    // erroring out.
    let mut explainer = LLMExplainer::new_ollama("this-model-does-not-exist:latest");

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
