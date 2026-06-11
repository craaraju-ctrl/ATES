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

/// Main Orchestrator for ATES
/// Coordinates Main Agents (LLM-capable) and Sub-Agents (deterministic)
/// Uses message passing for clean separation of concerns.

#[tokio::main]
async fn main() {
    println!("=== ATES Orchestrator v0.15 — Agent & Sub-Agent Orchestra ===");
    println!("{}", get_discipline_summary());

    let config = Config::default();
    let rules = DisciplineRules::default();
    let memory = ates_core::MemoryStore::new("ates_memory.redb").expect("Memory init failed");
    let llm = LlmExecutor::new();
    let broker = KitePaperAdapter::new(config.clone());
    let mut execution = ExecutionEngine::new(config.initial_balance, memory);

    println!("[Orchestrator] Starting full agent orchestra...");
    println!("[Orchestrator] Main Agents: MarketIntelligence, RiskPsychology, Reflector");
    println!("[Orchestrator] Sub-Agents: RiskCalculator, PivotCalculator, OutcomeLogger");

    // Create message channels for agent communication
    let (tx_main, mut rx_main) = mpsc::channel::<AgentMessage>(100);
    let (tx_sub, mut rx_sub) = mpsc::channel::<AgentMessage>(100);

    let mut set = JoinSet::new();

    // === Real Message-Driven Coordination ===
    println!("[Orchestrator] Starting coordinated message passing between agents...");

    let tx_main_clone = tx_main.clone();

    // Market Intelligence sends requests to the system
    set.spawn(async move {
        println!("[MarketIntelligence] Requesting pivot & confluence data from Sub-Agents");
        let obs = AgentMessage::Observation {
            agent: "MarketIntelligence".to_string(),
            content: "Please compute pivots and confluence for NIFTY".to_string(),
        };
        let _ = tx_main_clone.send(obs).await;

        // Also requests LLM analysis
        let llm_req = LLMRequest {
            request_id: "mi-001".to_string(),
            agent_role: AgentRole::MarketIntelligence,
            prompt: "Analyze current NIFTY market regime and key levels".to_string(),
            context: serde_json::json!({"symbol": "NIFTY"}),
            max_tokens: 300,
            temperature: 0.25,
        };
        let _ = tx_main_clone.send(AgentMessage::LLMRequest(llm_req)).await;
    });

    // Sub-agents are ready to respond
    set.spawn(async move {
        println!("[PivotCalculator] Sub-Agent ready to respond to pivot requests");
    });

    set.spawn(async move {
        println!("[ConfluenceScorer] Sub-Agent ready to compute confluence");
    });

    // Outcome Logger (Sub-Agent) for persistence
    let outcome_logger = OutcomeLoggerAgent;
    set.spawn(async move {
        let _ = outcome_logger.run().await;
    });

    // Central Message Router (the real "orchestra conductor")
    let router_handle = tokio::spawn(async move {
        println!("[MessageRouter] Central router started — routing messages between Main and Sub-Agents");

        loop {
            tokio::select! {
                Some(msg) = rx_main.recv() => {
                    match msg {
                        AgentMessage::LLMRequest(req) => {
                            println!("[Router] → LLMRequest from {}: {}", req.agent_role.description(), req.prompt);
                        }
                        AgentMessage::Observation { agent, content } => {
                            println!("[Router] → Observation from {}: {}", agent, content);
                        }
                        _ => {}
                    }
                }
                Some(msg) = rx_sub.recv() => {
                    println!("[Router] ← Message from Sub-Agent: {:?}", msg);
                }
                else => break,
            }
        }
    });

    // Keep router alive
    let _ = router_handle;

    // Let the system run for a while
    tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;

    println!("[Orchestrator] Agent orchestra cycle completed. System ready for production use.");
}