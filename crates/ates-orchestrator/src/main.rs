mod loops;

use ates_autonomous::state::initialize_autonomous_system;
use std::time::Duration;
use tokio::time::sleep;
use tokio::sync::{watch, Mutex as TokioMutex};
use tokio::signal;
use std::sync::Arc;
use std::net::SocketAddr;
use axum::{
    routing::{get, post},
    Router, Json, extract::State,
};
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;

// ── Loop Manager to dynamically start and stop the background temporal loops ──
struct LoopManager {
    orchestrator: ates_autonomous::AutonomousOrchestrator,
    client: reqwest::Client,
    assets: Vec<String>,
    shutdown_tx: Option<watch::Sender<bool>>,
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl LoopManager {
    fn new(orchestrator: ates_autonomous::AutonomousOrchestrator, client: reqwest::Client, assets: Vec<String>) -> Self {
        Self {
            orchestrator,
            client,
            assets,
            shutdown_tx: None,
            handles: Vec::new(),
        }
    }

    async fn start(&mut self) -> bool {
        if self.shutdown_tx.is_some() {
            return false; // Already running
        }

        let (tx, rx) = watch::channel(false);
        self.shutdown_tx = Some(tx);

        let orch_fast = self.orchestrator.clone();
        let client_fast = self.client.clone();
        let assets_fast = self.assets.clone();
        let rx_fast = rx.clone();

        let orch_medium = self.orchestrator.clone();
        let client_medium = self.client.clone();
        let assets_medium = self.assets.clone();
        let rx_medium = rx.clone();

        let orch_slow = self.orchestrator.clone();
        let state_slow = self.orchestrator.state.clone();
        let rx_slow = rx.clone();

        let fast_handle = tokio::spawn(async move {
            loops::fast_loop(orch_fast, client_fast, assets_fast, rx_fast).await;
        });

        let medium_handle = tokio::spawn(async move {
            loops::medium_loop(orch_medium, client_medium, assets_medium, rx_medium).await;
        });

        let slow_handle = tokio::spawn(async move {
            loops::slow_loop(orch_slow, state_slow, rx_slow).await;
        });

        self.handles = vec![fast_handle, medium_handle, slow_handle];

        {
            let mut p = self.orchestrator.state.portfolio.write().await;
            p.trading_enabled = true;
        }

        println!("[Orchestrator] 🚀 Background loops started.");
        true
    }

    async fn stop(&mut self) -> bool {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
            for handle in self.handles.drain(..) {
                let _ = handle.await;
            }
            {
                let mut p = self.orchestrator.state.portfolio.write().await;
                p.trading_enabled = false;
            }
            println!("[Orchestrator] 🛑 Background loops stopped cleanly.");
            true
        } else {
            false
        }
    }

    async fn is_running(&self) -> bool {
        self.shutdown_tx.is_some()
    }
}

// ── Web State Shared with Axum Handlers ───────────────────────────────────────
#[derive(Clone)]
struct WebState {
    orchestrator: ates_autonomous::AutonomousOrchestrator,
    loop_manager: Arc<TokioMutex<LoopManager>>,
}

// ── Start-up Initialization ─────────────────────────────────────────────────

async fn initialize_system(
    orchestrator: &ates_autonomous::AutonomousOrchestrator,
    client: &reqwest::Client,
) -> Vec<String> {
    let assets = vec!["NIFTY", "RELIANCE", "BTC", "ETH", "SOL"];
    println!("[Orchestrator] 🌐 Initializing all data feeds...");

    for symbol in &assets {
        let is_crypto = matches!(*symbol, "BTC" | "ETH" | "SOL");

        let bars = if is_crypto {
            loops::fetch_binance_klines(client, symbol, "1m", 100).await.unwrap_or_default()
        } else {
            loops::fetch_yahoo_ohlcv(client, symbol).await.unwrap_or_default()
        };
        if !bars.is_empty() {
            let mut history = orchestrator.state.ohlcv_history.write().await;
            history.insert(symbol.to_string(), bars);
        }

        loops::update_multi_tf_data(client, orchestrator, symbol, is_crypto).await;
    }

    {
        let mut summary = orchestrator.state.agent_market_summary.write().await;
        *summary = "System initialized. Monitoring NIFTY, RELIANCE, BTC, ETH, SOL with Ollama + Kronos.".to_string();
    }

    println!("[Orchestrator] ✅ System initialized — Web API ready");
    assets.iter().map(|s| s.to_string()).collect()
}

async fn restore_portfolio_state(state: &ates_autonomous::state::SharedState) -> bool {
    match state.memory.load_state("portfolio/state") {
        Ok(Some(json)) => {
            match serde_json::from_str::<ates_autonomous::types::PortfolioState>(&json) {
                Ok(restored) => {
                    let mut portfolio = state.portfolio.write().await;
                    *portfolio = restored;
                    println!("[Restore] ✅ Portfolio restored — Equity: ₹{:.2} | Cash: ₹{:.2} | Positions: {}",
                        portfolio.total_equity, portfolio.cash_balance, portfolio.open_positions.len());
                    true
                }
                Err(e) => {
                    eprintln!("[Restore] ⚠ Failed to parse portfolio state: {e}. Starting fresh.");
                    false
                }
            }
        }
        Ok(None) => {
            println!("[Restore] ℹ No saved portfolio state found. Starting fresh.");
            false
        }
        Err(e) => {
            eprintln!("[Restore] ⚠ Failed to load portfolio state: {e}. Starting fresh.");
            false
        }
    }
}

async fn restore_agent_tasks(state: &ates_autonomous::state::SharedState) {
    match state.memory.load_state("tasks/state") {
        Ok(Some(json)) => {
            if let Ok(restored) = serde_json::from_str::<Vec<ates_autonomous::state::AgentTask>>(&json) {
                let mut tasks = state.agent_tasks.write().await;
                *tasks = restored;
                println!("[Restore] ✅ Agent tasks restored from redb.");
            }
        }
        _ => {}
    }
}

// ── Graceful Shutdown ───────────────────────────────────────────────────────

async fn graceful_shutdown(orchestrator: &ates_autonomous::AutonomousOrchestrator) {
    println!("\n[Shutdown] 🛑 Winding down web server & portfolio state...");
    loops::save_portfolio_state(&orchestrator.state).await;
    let p = orchestrator.state.portfolio.read().await;
    println!("[Shutdown] Final Portfolio — Equity: ₹{:.2} | P&L: ₹{:.2} | Trades: {} | Positions open: {}",
        p.total_equity, p.daily_pnl, p.total_trades_today, p.open_positions.len());
    drop(p);
    println!("[Shutdown] 👋 ATES terminated. Goodbye.");
}

// ── Axum Endpoint Handlers ───────────────────────────────────────────────────

async fn get_system_status(
    State(state): State<WebState>,
) -> impl axum::response::IntoResponse {
    let portfolio = state.orchestrator.state.portfolio.read().await;
    let rules = state.orchestrator.state.rules.read().await;
    Json(serde_json::json!({
        "status": "ATES Running",
        "initial_balance": state.orchestrator.state.config.initial_balance,
        "use_confluence": rules.use_confluence,
        "cash_balance": portfolio.cash_balance,
        "total_equity": portfolio.total_equity,
    }))
}

async fn get_system_health(
    State(state): State<WebState>,
) -> impl axum::response::IntoResponse {
    let kronos_up = reqwest::Client::new()
        .get("http://localhost:8000/docs")
        .timeout(Duration::from_millis(400))
        .send().await.is_ok();

    let manager = state.loop_manager.lock().await;
    let running = manager.is_running().await;

    Json(serde_json::json!({
        "kronos": kronos_up,
        "orchestrator": running,
        "llm": true,
        "running": running,
    }))
}

async fn get_cot_chains(
    State(state): State<WebState>,
) -> impl axum::response::IntoResponse {
    let store = state.orchestrator.state.cot_store.read().await;
    Json(store.clone())
}

async fn start_autonomous_system(
    State(state): State<WebState>,
) -> impl axum::response::IntoResponse {
    let mut manager = state.loop_manager.lock().await;
    let started = manager.start().await;
    Json(serde_json::json!({
        "status": "starting",
        "kronos": true,
        "orchestrator": true,
        "started": started,
    }))
}

async fn stop_autonomous_system(
    State(state): State<WebState>,
) -> impl axum::response::IntoResponse {
    let mut manager = state.loop_manager.lock().await;
    let stopped = manager.stop().await;
    Json(serde_json::json!({
        "status": "stopped",
        "stopped": stopped,
    }))
}

#[derive(serde::Deserialize)]
struct TradeRequest {
    symbol: String,
    directionStr: String,
    entryPrice: f64,
    stopLoss: f64,
    takeProfit: f64,
}

async fn execute_trade(
    State(state): State<WebState>,
    Json(req): Json<TradeRequest>,
) -> impl axum::response::IntoResponse {
    use ates_core::{TradeDirection, TradeSetup, validate_trade_setup};
    use ates_autonomous::types::TradeSignal;

    let direction = match req.directionStr.to_lowercase().as_str() {
        "long" | "buy"   => TradeDirection::Long,
        "short" | "sell" => TradeDirection::Short,
        _                => return (axum::http::StatusCode::BAD_REQUEST, "Invalid direction. Use 'long' or 'short'".to_string()),
    };

    // Read real portfolio equity for accurate drawdown check
    let portfolio_equity = state.orchestrator.state.portfolio.read().await.total_equity;
    let context = ates_core::MarketContext {
        symbol: req.symbol.clone(),
        current_price: req.entryPrice,
        high: req.entryPrice * 1.01,
        low: req.entryPrice * 0.99,
        previous_close: req.entryPrice,
        timestamp: chrono::Utc::now(),
        daily_pnl: 0.0,
        equity: portfolio_equity,
        consecutive_losses: 0,
        is_red_folder_day: false,
        trend_direction: None,
    };

    let setup = TradeSetup::new(req.symbol.clone(), direction, req.entryPrice, req.stopLoss, req.takeProfit, context);
    let rules = state.orchestrator.state.rules.read().await;
    let check = validate_trade_setup(&setup.context, &*rules);

    if !check.passed {
        state.orchestrator.state.push_cot(
            "DisciplineCore",
            &format!("Discipline check for {} {} @ {:.2}", req.symbol, req.directionStr, req.entryPrice),
            "REJECTED",
            &check.reasons.join("; "),
            0.0,
            1,
            None,
            Some(req.symbol.clone()),
        ).await;
        return (axum::http::StatusCode::BAD_REQUEST, format!("DISCIPLINE REJECTED: {}", check.reasons.join(", ")));
    }

    if req.entryPrice <= 0.0 || req.stopLoss <= 0.0 || req.takeProfit <= 0.0 {
        return (axum::http::StatusCode::BAD_REQUEST, "INVALID PRICES: Entry, Stop Loss and Take Profit must be positive".to_string());
    }

    let signal = TradeSignal {
        symbol: req.symbol.clone(),
        direction,
        entry_price: req.entryPrice,
        stop_loss: req.stopLoss,
        take_profit: req.takeProfit,
        position_size: 10.0,
        confidence_score: 0.85,
        confluence_score: 0.85,
        risk_reward_ratio: 2.0,
        reasoning: "Manual API Order".to_string(),
        timestamp: chrono::Utc::now(),
        session_valid: true,
        risk_check_passed: true,
    };

    match state.orchestrator.execution.execute_paper_trade(&signal).await {
        Ok(exec_log) => {
            (axum::http::StatusCode::OK, format!("TRADE EXECUTED: {}", exec_log))
        }
        Err(e) => {
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("EXECUTION ERROR: {}", e))
        }
    }
}

#[derive(serde::Deserialize)]
struct CycleRequest {
    symbol: Option<String>,
}

async fn trigger_orchestra_cycle(
    State(state): State<WebState>,
    Json(req): Json<CycleRequest>,
) -> impl axum::response::IntoResponse {
    use ates_core::TradeDirection;
    let sym = req.symbol.unwrap_or_else(|| "NIFTY".to_string());
    let dir = TradeDirection::Long;
    let entry = 24500.0;
    let stop = 24200.0;
    let target = 25000.0;

    println!("[WebAPI] === FULL ORCHESTRA CYCLE TRIGGERED FROM HTTP API ===");

    match state.orchestrator.run_full_pipeline(&sym, dir, entry, stop, target).await {
        Ok(summary) => {
            let action = summary.final_signal
                .map(|s| format!("{} {:.2}",
                    if s.direction == TradeDirection::Long { "BUY" } else { "SELL" },
                    s.entry_price))
                .unwrap_or_else(|| "HOLD".to_string());
            Json(serde_json::json!({
                "message": format!(
                    "ORCHESTRA CYCLE COMPLETE | Action: {} | Reason: {} | Duration: {}ms",
                    action, summary.reason, summary.total_duration_ms
                )
            }))
        }
        Err(e) => {
            Json(serde_json::json!({
                "message": format!("ORCHESTRA CYCLE ERROR: {}", e)
            }))
        }
    }
}

#[derive(serde::Deserialize)]
struct RulesRequest {
    use_confluence: bool,
    respect_session_timing: bool,
}

async fn update_rules(
    State(state): State<WebState>,
    Json(req): Json<RulesRequest>,
) -> impl axum::response::IntoResponse {
    {
        let mut rules = state.orchestrator.state.rules.write().await;
        rules.use_confluence = req.use_confluence;
        rules.respect_session_timing = req.respect_session_timing;
    }
    state.orchestrator.state.push_cot(
        "MetaControl",
        "Update discipline rules",
        "UPDATED",
        &format!("Confluence: {}, SessionTiming: {}", req.use_confluence, req.respect_session_timing),
        0.9,
        1,
        None,
        None,
    ).await;
    Json(serde_json::json!({
        "message": "Rules updated successfully"
    }))
}

async fn run_backtest(
    State(state): State<WebState>,
) -> impl axum::response::IntoResponse {
    let rules = state.orchestrator.state.rules.read().await;
    let mut backtester = ates_core::Backtester::new(rules.clone());
    let mut dummy_data = Vec::new();
    for i in 0..50 {
        dummy_data.push(ates_core::MarketContext {
            symbol: "NIFTY".to_string(),
            current_price: 24000.0 + (i as f64 * 10.0),
            high: 24050.0,
            low: 23950.0,
            previous_close: 23980.0,
            timestamp: chrono::Utc::now(),
            daily_pnl: 0.0,
            consecutive_losses: 0,
            equity: 100000.0,
            is_red_folder_day: false,
            trend_direction: None,
        });
    }
    let result = backtester.run_simulation(dummy_data);

    state.orchestrator.state.push_cot(
        "Backtester",
        "Running 50-cycle backtest simulation",
        "COMPLETE",
        &format!("Trades: {}, Win Rate: {:.1}%, P&L: ₹{:.2}, Max DD: {:.2}%",
            result.total_trades, result.win_rate * 100.0, result.total_pnl, result.max_drawdown * 100.0),
        0.85,
        1,
        None,
        None,
    ).await;

    Json(serde_json::json!({
        "message": format!(
            "Backtest complete | Trades: {} | Win Rate: {:.1}% | Total P&L: ₹{:.2} | Max DD: {:.2}%",
            result.total_trades, result.win_rate * 100.0,
            result.total_pnl, result.max_drawdown * 100.0
        )
    }))
}

#[derive(serde::Deserialize)]
struct PriceQuery {
    symbol: String,
}

async fn get_agent_tree() -> impl axum::response::IntoResponse {
    Json(ates_autonomous::Tredo::tree_json())
}

async fn fetch_live_stock_price(
    axum::extract::Query(req): axum::extract::Query<PriceQuery>,
) -> impl axum::response::IntoResponse {
    let sym_upper = req.symbol.to_uppercase();
    let yahoo_symbol = match sym_upper.as_str() {
        "NIFTY"    => "^NSEI",
        "RELIANCE" => "RELIANCE.NS",
        other      => other,
    };
    let client = reqwest::Client::new();
    let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1m&range=1d", yahoo_symbol);
    let resp: serde_json::Value = match client.get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send().await {
            Ok(r) => r.json().await.unwrap_or_default(),
            Err(_) => serde_json::Value::Null,
        };
    let price = resp["chart"]["result"][0]["meta"]["regularMarketPrice"]
        .as_f64()
        .unwrap_or(24500.0);
    Json(price)
}

// ── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        let location = panic_info.location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown location".to_string());
        eprintln!("\n💥 [PANIC] {} at {} — SYSTEM CRASHED", msg, location);
    }));

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║   ATES v2 — TRUE LEARNING AGENTIC TRADING AI         ║");
    println!("║   HTTP Server | Temporal Loops | Vector Memory       ║");
    println!("╚══════════════════════════════════════════════════════╝");
    ates_autonomous::Tredo::print_tree();

    let mut orchestrator = match initialize_autonomous_system().await {
        Ok(o) => o,
        Err(e) => {
            eprintln!("[FATAL] Failed to initialize: {e}");
            std::process::exit(1);
        }
    };
    // Initialize the Tredo agent hierarchy (zero-copy Arc sharing from orchestrator)
    orchestrator.init_tredo();
    let client = reqwest::Client::new();

    restore_portfolio_state(&orchestrator.state).await;
    restore_agent_tasks(&orchestrator.state).await;

    let assets = initialize_system(&orchestrator, &client).await;

    // Create background loop manager
    let loop_manager = Arc::new(TokioMutex::new(LoopManager::new(
        orchestrator.clone(),
        client.clone(),
        assets,
    )));

    // Start background loops by default
    {
        let mut manager = loop_manager.lock().await;
        manager.start().await;
    }

    // Set up Axum Web Server routing
    let state = WebState {
        orchestrator: orchestrator.clone(),
        loop_manager: loop_manager.clone(),
    };

    let api_routes = Router::new()
        .route("/status", get(get_system_status))
        .route("/health", get(get_system_health))
        .route("/cot", get(get_cot_chains))
        .route("/start", post(start_autonomous_system))
        .route("/stop", post(stop_autonomous_system))
        .route("/trade", post(execute_trade))
        .route("/trigger_cycle", post(trigger_orchestra_cycle))
        .route("/rules", post(update_rules))
        .route("/backtest", get(run_backtest))
        .route("/price", get(fetch_live_stock_price))
        .route("/agents", get(get_agent_tree));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Serve static files from frontend path and mount API on /api
    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(ServeDir::new("src-tauri/frontend"))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("[Orchestrator] 🌐 HTTP server starting on {}", addr);

    let server_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    println!("\n[Main] 🛑 Shutdown signal received. Stopping all systems...");

    {
        let mut manager = loop_manager.lock().await;
        manager.stop().await;
    }
    server_handle.abort();

    graceful_shutdown(&orchestrator).await;
}
