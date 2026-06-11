use async_trait::async_trait;
use std::error::Error;

use crate::disciplined_core::{PivotLevels, DisciplineCheck, MarketContext};

/// Distinguishes between Main Agents (permitted to initiate LLM calls)
/// and Sub-Agents (pure deterministic logic only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTier {
    /// Main agents coordinate and may request LLM assistance when necessary.
    Main,
    /// Sub-agents perform fast, rule-based logic and never call LLMs directly.
    Sub,
}

/// Input that can be passed to an agent when it is asked to perform work.
#[derive(Debug, Clone)]
pub enum AgentInput {
    /// Request to calculate pivot points.
    PivotRequest { high: f64, low: f64, close: f64 },
    /// Request to calculate confluence score.
    ConfluenceRequest { context: MarketContext },
    /// Request to evaluate risk / discipline.
    RiskRequest { context: MarketContext },
    /// Request to log an outcome/decision.
    LogOutcome { key: String, value: String },
    /// Generic / no specific input.
    None,
}

/// Structured output returned by an agent after processing.
#[derive(Debug, Clone)]
pub enum AgentOutput {
    /// Result of pivot point calculation.
    PivotResult(PivotLevels),
    /// Result of confluence scoring.
    ConfluenceResult(f64),
    /// Result of risk/discipline evaluation.
    RiskResult(DisciplineCheck),
    /// Agent completed its work (no specific data).
    Done,
    /// No meaningful output produced.
    NoOutput,
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
    /// Agents receive optional structured input and return structured output.
    /// This enables proper message-driven coordination between agents.
    async fn run(&self, input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>>;
}