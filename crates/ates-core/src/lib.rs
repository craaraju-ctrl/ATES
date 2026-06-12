pub mod disciplined_core;
pub mod agent;
pub mod memory;
pub mod messages;
pub mod role;
pub mod llm;
pub mod backtest;
pub mod config;
pub mod broker;
pub mod execution;
pub mod paper_engine;
pub mod kronos_client;
pub mod calendar;
pub mod goals;
pub mod episode;
pub mod news;
pub mod vector_memory;
pub mod patterns;


pub use disciplined_core::{
    calculate_confluence_score,
    calculate_pivot_points,
    check_risk_limits,
    is_in_trading_session,
    validate_trade_setup,
    DisciplineCheck,
    DisciplineRules,
    MarketContext,
    PivotLevels,
    PivotMethod,
    TrendDirection,
    get_discipline_summary,
};
pub use backtest::{TradeSetup, TradeDirection, Backtester, BacktestResult};
pub use agent::{Agent, AgentTier, AgentInput, AgentOutput};
pub use memory::MemoryStore;
pub use messages::{AgentMessage, LLMRequest, LLMResponse};
pub use role::AgentRole;
pub use llm::{LlmExecutor, LlmTradeDecision};
pub use config::Config;
pub use execution::ExecutionEngine;
pub use kronos_client::{KronosClient, KronosForecastRequest, KronosForecastResponse, OhlcvBar, KronosForecastTool};
pub use calendar::{CalendarEvent, EventImpact, generate_economic_calendar};
pub use goals::{TradingGoals, TradingMode};
pub use episode::{
    TradingEpisode, MarketStateSnapshot, ReasoningStep,
    TradeOutcome, PostTradeReflection,
};
pub use news::{NewsFetcher, NewsItem, NewsContext};
pub use vector_memory::{VectorMemory, VectorEntry, SimilarResult};
pub use patterns::{CandlestickPattern, detect_patterns, format_patterns,
    MultiTfPatternConfirmation, ConfirmationLevel,
    detect_patterns_multi_tf, format_mtf_confirmation};
pub use paper_engine::*;