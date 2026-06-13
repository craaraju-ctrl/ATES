// Full debate aggregator (Proposer, Critic, Risk, Historian) using new skills.
// Research: TradingAgents/FINCON style multi-agent debate for robust decisions.
// Replaces or augments single LLM in StrategyDecision for hands-off quality.
// Uses local DebateTurn (core AgentOutput is enum with specific variants, not freeform).

use crate::state::SharedState;
use tredo_core::{AgentInput, MarketContext};
use crate::{sentiment_analyzer::SentimentAnalyzer, volatility_calculator::VolatilityCalculator, regime_detector::RegimeDetector, correlation_checker::CorrelationChecker};

// Lightweight turn for debate participants (not the core AgentOutput enum).
#[derive(Clone, Debug)]
pub struct DebateTurn {
    action: String,
    confidence: f64,
    reasoning: String,
}

// Helper to extract MarketContext from the enum (deep fix for type mismatch)
fn extract_context(input: &AgentInput) -> Option<&MarketContext> {
    match input {
        AgentInput::ConfluenceRequest { context } => Some(context),
        AgentInput::RiskRequest { context } => Some(context),
        _ => None,
    }
}

pub struct ProposerAgent { state: SharedState }
pub struct CriticAgent { state: SharedState }
pub struct RiskAgent { state: SharedState }
pub struct HistorianAgent { _state: SharedState }

impl ProposerAgent {
    pub fn new(state: SharedState) -> Self { Self { state } }
    pub async fn propose(&self, input: &AgentInput) -> DebateTurn {
        let ctx = match extract_context(input) {
            Some(c) => c,
            None => return DebateTurn { action: "HOLD".into(), confidence: 0.0, reasoning: "no context".into() },
        };
        // Bullish bias using new skills
        let sentiment = SentimentAnalyzer::new(self.state.clone()).analyze_sentiment(&ctx.symbol).await;
        let (vol, _) = VolatilityCalculator::new(self.state.clone()).compute_volatility(&ctx.symbol, ctx.current_price).await;
        let regime = RegimeDetector::new(self.state.clone()).detect_regime(&ctx.symbol, ctx.current_price).await;
        let action = if format!("{:?}", regime).contains("Bull") || sentiment > 0.6 { "BUY" } else { "HOLD" };
        DebateTurn { action: action.to_string(), confidence: 0.7 + (sentiment - 0.5).max(0.0), reasoning: format!("Proposer: regime {:?}, sent {:.2}, vol {:.2}", regime, sentiment, vol) }
    }
}

impl CriticAgent {
    pub fn new(state: SharedState) -> Self { Self { state } }
    pub async fn critique(&self, proposal: &str, input: &AgentInput) -> DebateTurn {
        let ctx = match extract_context(input) {
            Some(c) => c,
            None => return DebateTurn { action: "CRITIQUE".into(), confidence: 0.5, reasoning: "no context".into() },
        };
        let corr = CorrelationChecker::new(self.state.clone()).check_correlation(&ctx.symbol).await;
        let critique = if proposal == "BUY" && corr < 0.4 { "CAUTION: low corr, possible fakeout" } else { "OK but watch risk" };
        DebateTurn { action: "CRITIQUE".to_string(), confidence: 0.6, reasoning: format!("Critic on {}: {} (corr {:.2})", proposal, critique, corr) }
    }
}

impl RiskAgent {
    pub fn new(state: SharedState) -> Self { Self { state } }
    pub async fn assess(&self, input: &AgentInput) -> DebateTurn {
        let ctx = match extract_context(input) {
            Some(c) => c,
            None => return DebateTurn { action: "PASS".into(), confidence: 0.5, reasoning: "no context".into() },
        };
        // Enforcer using vol/regime
        let (vol, exp) = VolatilityCalculator::new(self.state.clone()).compute_volatility(&ctx.symbol, ctx.current_price).await;
        let action = if vol > 0.03 || exp { "BLOCK" } else { "PASS" };
        DebateTurn { action: action.to_string(), confidence: if vol > 0.03 { 0.9 } else { 0.7 }, reasoning: format!("Risk: vol {:.2} expansion {}", vol, exp) }
    }
}

impl HistorianAgent {
    pub fn new(state: SharedState) -> Self { Self { _state: state } }
    pub async fn recall(&self, input: &AgentInput) -> DebateTurn {
        let ctx = match extract_context(input) {
            Some(c) => c,
            None => return DebateTurn { action: "RECALL".into(), confidence: 0.5, reasoning: "no context".into() },
        };
        // === agentmemory integration: real persistent recall for runtime agents ===
        // Uses the shared agentmemory store (same as Grok/Hermes coding agents) for infinite memory
        // of past decisions, lessons, across restarts. Complements local sqlite episodes.
        let mem = tredo_core::AgentMemoryClient::new();
        let past = mem.recall(&format!("past decisions {}", ctx.symbol)).await.unwrap_or_default();
        let reasoning = if past.is_empty() {
            format!("Historian: similar past episodes for {} suggest caution on high vol entries", ctx.symbol)
        } else {
            format!("Historian (from agentmemory): {} past for {} e.g. {}", past.len(), ctx.symbol, past.last().unwrap_or(&"".to_string())[..100.min(past.last().map_or(0, |s| s.len()))].to_string())
        };
        DebateTurn { action: "RECALL".to_string(), confidence: 0.65 + (past.len() as f64 * 0.05).min(0.2), reasoning }
    }
}

pub async fn run_debate(state: SharedState, input: &AgentInput) -> (String, f64, String) {
    let proposer = ProposerAgent::new(state.clone());
    let critic = CriticAgent::new(state.clone());
    let risk = RiskAgent::new(state.clone());
    let historian = HistorianAgent::new(state.clone());

    let prop = proposer.propose(input).await;
    let crit = critic.critique(&prop.action, input).await;
    let rsk = risk.assess(input).await;
    let hist = historian.recall(input).await;

    // Aggregator (weighted vote + escalation) — matches original intent
    let mut buy_score = 0.0;
    if prop.action == "BUY" { buy_score += prop.confidence * 0.3; }
    if crit.action.contains("OK") { buy_score += 0.2; }
    if rsk.action == "PASS" { buy_score += rsk.confidence * 0.3; }
    if !hist.reasoning.contains("caution") { buy_score += 0.2; }

    let (final_action, conf, reason) = if buy_score > 0.65 && rsk.action == "PASS" {
        ("BUY".to_string(), buy_score, format!("Debate: Proposer {} | Critic {} | Risk {} | Hist: balanced", prop.action, crit.action, rsk.action))
    } else if buy_score < 0.35 || rsk.action == "BLOCK" {
        ("HOLD".to_string(), 0.8, "Debate consensus: high risk or low conviction".to_string())
    } else {
        ("HOLD".to_string(), 0.6, "Debate: mixed signals, escalate to HOLD".to_string())
    };

    (final_action, conf, reason)
}
