//! AI-Assisted Reasoning Layer
//!
//! Phase 9: Converts structured analysis into natural language explanations
//! using OSS LLM (Phi-2 3.8B, MIT license).
//!
//! Provides semantic search over replay sessions and automated debugging recommendations.

pub mod llm_integration;
pub mod semantic_search;
pub mod explanation_generator;
pub mod incident_patterns;

pub use llm_integration::LLMExplainer;
pub use semantic_search::SemanticSearchEngine;
pub use explanation_generator::ExplanationGenerator;
pub use incident_patterns::IncidentPatternAnalyzer;
