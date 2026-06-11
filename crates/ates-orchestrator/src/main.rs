use ates_core::Agent;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use std::error::Error;

use ates_core::{
    DisciplineRules, get_discipline_summary, LlmExecutor, Config,
    broker::KitePaperAdapter,
    execution::ExecutionEngine,
    messages::{AgentMessage, LLMRequest, LLMResponse},
    role::AgentRole,
};
use ates_agents::{
    MarketIntelligenceAgent,
    RiskPsychologyAgent,
    ReflectorAgent,
    RiskCalculatorAgent,
    PivotCalculatorAgent,
    OutcomeLoggerAgent,
};

// ... rest of the file remains the same ...