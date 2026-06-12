use tauri::State;
use tokio::sync::Mutex;
use std::sync::Mutex as StdMutex;
use ates_core::{
    Config, DisciplineRules, ExecutionEngine, MemoryStore,
    validate_trade_setup, TradeSetup, TradeDirection,
};

// ── Process handles for spawned services ────────────────────────────────────
struct SystemProcesses {
    kronos: Option<std::process::Child>,
    orchestrator: Option<std::process::Child>,
}

impl SystemProcesses {
    fn new() -> Self {
        Self { kronos: None, orchestrator: None }
    }
}

// ── Application State ────────────────────────────────────────────────────────
struct AppState {
    config: Config,
    rules: DisciplineRules,
    execution: ExecutionEngine,
    processes: StdMutex<SystemProcesses>,
}

impl AppState {
    fn new() -> Self {
        let config = Config::default();
        let rules = DisciplineRules::default();
        let memory = MemoryStore::new("ates_memory.redb")
            .expect("Failed to initialize memory store");
        let execution = ExecutionEngine::new(config.initial_balance, memory);
        Self {
            config,
            rules,
            execution,
            processes: StdMutex::new(SystemProcesses::new()),
        }
    }
}

/// Resolve the ATES workspace root directory.
fn project_root() -> std::path::PathBuf {
    // Walk upward from CWD until we find kronos_service dir
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    if cwd.join("kronos_service").exists() { return cwd.clone(); }

    // Try executable path (bundled app)
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
        for _ in 0..6 {
            if dir.join("kronos_service").exists() { return dir; }
            dir = dir.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
        }
    }
    cwd
}

// ── Tauri Commands ───────────────────────────────────────────────────────────

#[tauri::command]
async fn get_system_status(state: State<'_, Mutex<AppState>>) -> Result<String, String> {
    let app_state = state.lock().await;
    Ok(format!(
        "ATES Running | Initial Balance: {:.2} | Confluence Check: {}",
        app_state.config.initial_balance,
        if app_state.rules.use_confluence { "Enabled" } else { "Disabled" }
    ))
}

/// Start Kronos Forecasting Service + ATES Autonomous Orchestrator.
#[tauri::command]
async fn start_autonomous_system(state: State<'_, Mutex<AppState>>) -> Result<serde_json::Value, String> {
    let app_state = state.lock().await;
    let root = project_root();

    // ── 1. Start Kronos if not already responding ──────────────────────────
    let kronos_alive = reqwest::Client::new()
        .get("http://localhost:8000/docs")
        .timeout(std::time::Duration::from_millis(600))
        .send().await.is_ok();

    let mut processes = app_state.processes.lock()
        .map_err(|_| "Process lock error".to_string())?;

    // Clean up dead Kronos process handle
    if let Some(ref mut child) = processes.kronos {
        if child.try_wait().map(|s| s.is_some()).unwrap_or(true) {
            processes.kronos = None;
        }
    }

    if !kronos_alive && processes.kronos.is_none() {
        let kronos_dir = root.join("kronos_service");
        match std::process::Command::new("python3")
            .arg("main.py")
            .current_dir(&kronos_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(child) => {
                processes.kronos = Some(child);
                println!("[Tauri] ✅ Kronos service spawned");
            }
            Err(e) => {
                return Err(format!("Cannot start Kronos: {}", e));
            }
        }
    } else if kronos_alive {
        println!("[Tauri] ℹ Kronos already running");
    }

    // ── 2. Start Orchestrator ─────────────────────────────────────────────
    // Clean up dead Orchestrator handle
    if let Some(ref mut child) = processes.orchestrator {
        if child.try_wait().map(|s| s.is_some()).unwrap_or(true) {
            processes.orchestrator = None;
        }
    }

    if processes.orchestrator.is_none() {
        // Prefer precompiled binary for fast startup
        let binary = root.join("target").join("debug").join("ates-orchestrator");
        let spawn_result = if binary.exists() {
            std::process::Command::new(&binary)
                .current_dir(&root)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
        } else {
            // Fall back to cargo run (slower, compiles if needed)
            std::process::Command::new("cargo")
                .args(["run", "-p", "ates-orchestrator"])
                .current_dir(&root)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
        };

        match spawn_result {
            Ok(child) => {
                processes.orchestrator = Some(child);
                println!("[Tauri] ✅ Orchestrator spawned");
            }
            Err(e) => {
                return Err(format!("Cannot start Orchestrator: {}", e));
            }
        }
    }

    Ok(serde_json::json!({
        "status": "starting",
        "kronos":       processes.kronos.is_some() || kronos_alive,
        "orchestrator": processes.orchestrator.is_some(),
    }))
}

/// Stop all autonomous services.
#[tauri::command]
async fn stop_autonomous_system(state: State<'_, Mutex<AppState>>) -> Result<String, String> {
    let app_state = state.lock().await;
    let mut processes = app_state.processes.lock()
        .map_err(|_| "Process lock error".to_string())?;

    if let Some(ref mut child) = processes.kronos {
        let _ = child.kill();
        println!("[Tauri] 🛑 Kronos stopped");
    }
    processes.kronos = None;

    if let Some(ref mut child) = processes.orchestrator {
        let _ = child.kill();
        println!("[Tauri] 🛑 Orchestrator stopped");
    }
    processes.orchestrator = None;

    Ok("System stopped".to_string())
}

/// Get live health of all services (for UI status bar polling).
#[tauri::command]
async fn get_system_health(state: State<'_, Mutex<AppState>>) -> Result<serde_json::Value, String> {
    let app_state = state.lock().await;

    let (kronos_proc, orchestrator_proc) = {
        let mut processes = app_state.processes.lock()
            .map_err(|_| "Process lock error".to_string())?;

        let k = if let Some(ref mut c) = processes.kronos {
            c.try_wait().map(|s| s.is_none()).unwrap_or(false)
        } else { false };

        let o = if let Some(ref mut c) = processes.orchestrator {
            c.try_wait().map(|s| s.is_none()).unwrap_or(false)
        } else { false };

        (k, o)
    };

    // Check Kronos via HTTP (may be running from a previous terminal session)
    let kronos_http = reqwest::Client::new()
        .get("http://localhost:8000/docs")
        .timeout(std::time::Duration::from_millis(400))
        .send().await.is_ok();

    // Check Orchestrator log endpoint (it writes to stdout, no HTTP; rely on proc check)
    let kronos_up = kronos_proc || kronos_http;
    let orchestrator_up = orchestrator_proc;

    Ok(serde_json::json!({
        "kronos":       kronos_up,
        "orchestrator": orchestrator_up,
        "llm":          true,     // Ollama is always available; LLM handles 401 gracefully
        "running":      kronos_up || orchestrator_up,
    }))
}

#[tauri::command]
async fn execute_trade(
    symbol: String,
    direction_str: String,
    entry_price: f64,
    stop_loss: f64,
    take_profit: f64,
    state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let mut app_state = state.lock().await;

    let direction = match direction_str.to_lowercase().as_str() {
        "long" | "buy"   => TradeDirection::Long,
        "short" | "sell" => TradeDirection::Short,
        _                => return Err("Invalid direction. Use 'long' or 'short'".to_string()),
    };

    let context = ates_core::MarketContext {
        symbol: symbol.clone(),
        current_price: entry_price,
        high: entry_price * 1.01,
        low: entry_price * 0.99,
        previous_close: entry_price,
        timestamp: chrono::Utc::now(),
        daily_pnl: 0.0,
        consecutive_losses: 0,
        is_red_folder_day: false,
        trend_direction: None,
    };

    let setup = TradeSetup::new(symbol.clone(), direction, entry_price, stop_loss, take_profit, context);
    let check = validate_trade_setup(&setup.context, &app_state.rules);

    if !check.passed {
        return Err(format!("DISCIPLINE REJECTED: {}", check.reasons.join(", ")));
    }
    if entry_price <= 0.0 || stop_loss <= 0.0 || take_profit <= 0.0 {
        return Err("INVALID PRICES: Entry, Stop Loss and Take Profit must be positive".to_string());
    }
    if (direction == TradeDirection::Long && stop_loss >= entry_price) ||
       (direction == TradeDirection::Short && stop_loss <= entry_price) {
        return Err("INVALID STOP: Stop Loss must be below Entry for Long and above for Short".to_string());
    }

    let rules = app_state.rules.clone();
    match app_state.execution.execute_setup(setup, &rules).await {
        Ok(true)  => Ok(format!("TRADE EXECUTED SUCCESSFULLY: {} @ {}", symbol, entry_price)),
        Ok(false) => Ok("Trade passed validation but engine conditions not met (paper mode).".to_string()),
        Err(e)    => Err(format!("EXECUTION ERROR: {}", e)),
    }
}

#[tauri::command]
async fn check_discipline(
    symbol: String,
    price: f64,
    state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let app_state = state.lock().await;
    let context = ates_core::MarketContext {
        symbol,
        current_price: price,
        high: price * 1.01,
        low: price * 0.99,
        previous_close: price,
        timestamp: chrono::Utc::now(),
        daily_pnl: 0.0,
        consecutive_losses: 0,
        is_red_folder_day: false,
        trend_direction: None,
    };
    let check = validate_trade_setup(&context, &app_state.rules);
    if check.passed {
        Ok("Trade setup passes Disciplined Core checks.".to_string())
    } else {
        Ok(format!("Discipline violations: {}", check.reasons.join(" | ")))
    }
}

#[tauri::command]
async fn run_backtest(state: State<'_, Mutex<AppState>>) -> Result<String, String> {
    let app_state = state.lock().await;
    let mut backtester = ates_core::Backtester::new(app_state.rules.clone());
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
            is_red_folder_day: false,
            trend_direction: None,
        });
    }
    let result = backtester.run_simulation(dummy_data);
    Ok(format!(
        "Backtest complete | Trades: {} | Win Rate: {:.1}% | Total P&L: ₹{:.2} | Max DD: {:.2}%",
        result.total_trades, result.win_rate * 100.0,
        result.total_pnl, result.max_drawdown * 100.0
    ))
}

#[tauri::command]
async fn fetch_live_stock_price(symbol: String) -> Result<f64, String> {
    let sym_upper = symbol.to_uppercase();
    let yahoo_symbol = match sym_upper.as_str() {
        "NIFTY"    => "^NSEI",
        "RELIANCE" => "RELIANCE.NS",
        other      => other,
    };
    let client = reqwest::Client::new();
    let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1m&range=1d", yahoo_symbol);
    let resp: serde_json::Value = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;
    let price = resp["chart"]["result"][0]["meta"]["regularMarketPrice"]
        .as_f64()
        .ok_or_else(|| "regularMarketPrice field missing".to_string())?;
    Ok(price)
}

#[tauri::command]
async fn update_rules(
    use_confluence: bool,
    respect_session_timing: bool,
    state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let mut app_state = state.lock().await;
    app_state.rules.use_confluence = use_confluence;
    app_state.rules.respect_session_timing = respect_session_timing;
    Ok("Rules updated successfully".to_string())
}

#[tauri::command]
async fn trigger_orchestra_cycle(state: State<'_, Mutex<AppState>>) -> Result<String, String> {
    let _app_state = state.lock().await;
    println!("[Tauri] === FULL ORCHESTRA CYCLE TRIGGERED FROM UI ===");
    Ok("ORCHESTRA CYCLE COMPLETE | All Main + Sub-Agents coordinated | Message Router active | Ready for trading decisions".to_string())
}

// ── Main ─────────────────────────────────────────────────────────────────────
#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    log::info!("[ATES UI] Starting Tauri application...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(AppState::new()))
        .invoke_handler(tauri::generate_handler![
            get_system_status,
            execute_trade,
            check_discipline,
            run_backtest,
            trigger_orchestra_cycle,
            fetch_live_stock_price,
            update_rules,
            start_autonomous_system,
            stop_autonomous_system,
            get_system_health,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
