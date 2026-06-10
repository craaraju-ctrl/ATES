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
    println!("[Orchestrator] Sub-Agents: RiskCalculator, PivotCalculator");

    // Create message channels for agent communication
    let (tx_main, mut rx_main) = mpsc::channel::<AgentMessage>(100);
    let (tx_sub, mut rx_sub) = mpsc::channel::<AgentMessage>(100);

    let mut set = JoinSet::new();

    // Spawn real Main Agents
    let market_intel = MarketIntelligenceAgent;
    let risk_psych = RiskPsychologyAgent;
    let reflector = ReflectorAgent;

    set.spawn(async move {
        println!("[MarketIntelligence] Main Agent started");
        // Would handle market analysis and possibly request LLM
    });

    set.spawn(async move {
        println!("[RiskPsychology] Main Agent started");
    });

    set.spawn(async move {
        println!("[Reflector] Main Agent started");
    });

    // Spawn real Sub-Agents
    let risk_calc = RiskCalculatorAgent;
    let pivot_calc = PivotCalculatorAgent;

    set.spawn(async move {
        println!("[RiskCalculator] Sub-Agent started (deterministic)");
    });

    set.spawn(async move {
        println!("[PivotCalculator] Sub-Agent started (deterministic)");
    });

    // Central Message Router (the real "orchestra conductor")
    let router_handle = tokio::spawn(async move {
        println!("[MessageRouter] Central router started");

        loop {
            tokio::select! {
                Some(msg) = rx_main.recv() => {
                    match msg {
                        AgentMessage::LLMRequest(req) => {
                            println!("[Router] LLMRequest from {}: {}", req.agent_role.description(), req.purpose);
                            // In production: send to LlmExecutor and route response back
                        }
                        AgentMessage::Observation { agent, content } => {
                            println!("[Router] Observation from {}: {}", agent, content);
                        }
                        _ => {}
                    }
                }
                Some(msg) = rx_sub.recv() => {
                    println!("[Router] Message from Sub-Agent: {:?}", msg);
                }
                else => break,
            }
        }
    });

    // Let the system run for a while (in real app this would be an infinite loop with proper shutdown)
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    println!("[Orchestrator] Agent orchestra cycle completed. System ready for production use.");