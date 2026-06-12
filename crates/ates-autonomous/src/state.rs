use std::sync::Arc;
use tokio::sync::RwLock;
use ates_core::{MemoryStore, Config, DisciplineRules, LlmExecutor};
use crate::types::{PortfolioState, TradeSignal, MarketRegime};

#[derive(Clone)]
pub struct SharedState {
    pub portfolio: Arc<RwLock<PortfolioState>>,
    pub memory: Arc<MemoryStore>,
    pub rules: Arc<RwLock<DisciplineRules>>,
    pub config: Arc<Config>,
    pub last_signals: Arc<RwLock<Vec<TradeSignal>>>,
    pub market_regime: Arc<RwLock<Option<MarketRegime>>>,
    pub llm: Arc<LlmExecutor>,
    /// Kronos forecast stored by MarketIntelligenceAgent (Phase 2) for use in StrategyDecisionAgent (Phase 5).
    pub last_forecast: Arc<RwLock<Option<serde_json::Value>>>,
    /// LLM reasoning from last cycle — stored for debugging / UI display.
    pub last_llm_reason: Arc<RwLock<String>>,
}

impl SharedState {
    pub fn new(memory: MemoryStore, rules: DisciplineRules, config: Config) -> Self {
        let portfolio = PortfolioState {
            cash_balance: config.initial_balance,
            total_equity: config.initial_balance,
            daily_pnl: 0.0,
            daily_pnl_pct: 0.0,
            open_positions: Vec::new(),
            total_trades_today: 0,
            winning_trades_today: 0,
            losing_trades_today: 0,
            consecutive_losses: 0,
            max_drawdown_today: 0.0,
            last_trade_time: None,
            trading_enabled: true,
        };

        Self {
            portfolio: Arc::new(RwLock::new(portfolio)),
            memory: Arc::new(memory),
            rules: Arc::new(RwLock::new(rules)),
            config: Arc::new(config),
            last_signals: Arc::new(RwLock::new(Vec::new())),
            market_regime: Arc::new(RwLock::new(None)),
            llm: Arc::new(LlmExecutor::new()),
            last_forecast: Arc::new(RwLock::new(None)),
            last_llm_reason: Arc::new(RwLock::new(String::new())),
        }
    }
}

pub async fn initialize_autonomous_system() -> Result<crate::AutonomousOrchestrator, Box<dyn std::error::Error + Send + Sync>> {
    let memory = MemoryStore::new("ates_autonomous.redb")?;
    let rules = DisciplineRules::default();
    let config = Config::default();
    let state = SharedState::new(memory, rules, config);
    Ok(crate::AutonomousOrchestrator::new(state))
}