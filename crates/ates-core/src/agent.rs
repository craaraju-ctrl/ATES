use async_trait::async_trait;
use std::error::Error;

/// Distinguishes between Main Agents (permitted to initiate LLM calls)
/// and Sub-Agents (pure deterministic logic only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTier {
    /// Main agents coordinate and may request LLM assistance when necessary.
    Main,
    /// Sub-agents perform fast, rule-based logic and never call LLMs directly.
    Sub,
}

/// Core trait for all agents in the ATES two-tier architecture.
/// 
/// Main agents may return or handle `LLMRequest` objects.
/// Sub-agents must remain fully deterministic and respect the Disciplined Core.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Human-readable name of the agent.
    fn name(&self) -> &str;

    /// Returns the tier of this agent (Main or Sub).
    fn tier(&self) -> AgentTier;

    /// Primary execution entry point.
    /// 
    /// For Sub-Agents this should execute pure logic.
    /// For Main Agents this may also trigger controlled LLM calls via messages.
    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}