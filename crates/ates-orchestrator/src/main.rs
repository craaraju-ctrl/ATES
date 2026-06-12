use ates_autonomous::state::initialize_autonomous_system;
use ates_autonomous::portfolio_manager::PortfolioManagerAgent;
use ates_core::{TradeDirection, Agent};
use std::time::Duration;
use tokio::time::sleep;
use std::collections::HashMap;

/// Fetch live crypto price from Binance
async fn fetch_binance_price(client: &reqwest::Client, symbol: &str) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}USDT", symbol);
    let resp: serde_json::Value = client.get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .timeout(Duration::from_secs(5))
        .send().await?.json().await?;
    let price_str = resp["price"].as_str().ok_or("price field missing")?;
    Ok(price_str.parse()?)
}

/// Fetch live stock price from Yahoo Finance
async fn fetch_yahoo_price(client: &reqwest::Client, symbol: &str) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let yahoo_symbol = match symbol {
        "NIFTY"    => "^NSEI",
        "RELIANCE" => "RELIANCE.NS",
        other      => other,
    };
    let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1m&range=1d", yahoo_symbol);
    let resp: serde_json::Value = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
        .timeout(Duration::from_secs(5))
        .send().await?.json().await?;
    let price = resp["chart"]["result"][0]["meta"]["regularMarketPrice"]
        .as_f64()
        .ok_or("regularMarketPrice field missing")?;
    Ok(price)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║   ATES — Autonomous Trading Execution System          ║");
    println!("║   LLM-driven agent loop  (Ollama + Kronos)            ║");
    println!("╚══════════════════════════════════════════════════════╝");

    let orchestrator = initialize_autonomous_system().await?;
    let client = reqwest::Client::new();

    // Seed prices (used when live API is unavailable)
    let mut current_prices: HashMap<String, f64> = [
        ("NIFTY",    24500.0),
        ("RELIANCE",  2950.0),
        ("BTC",      67500.0),
        ("ETH",       3500.0),
        ("SOL",        155.0),
    ].iter().map(|(k, v)| (k.to_string(), *v)).collect();

    let assets = vec!["NIFTY", "RELIANCE", "BTC", "ETH", "SOL"];
    let portfolio_mgr = PortfolioManagerAgent::new(orchestrator.state.clone());

    println!("[Orchestrator] 🚀 Active. Monitoring: {:?}", assets);
    println!("[Orchestrator] Each 5-second cycle: price fetch → Kronos → LLM → execute\n");

    let mut cycle: u64 = 0;

    loop {
        cycle += 1;
        println!("\n══════════════════════ Cycle #{} ══════════════════════", cycle);

        // ── 1. Live price refresh & unrealized P&L update ────────────────────
        for symbol in &assets {
            let is_crypto = matches!(*symbol, "BTC" | "ETH" | "SOL");
            let old_price = *current_prices.get(*symbol).unwrap_or(&20000.0);

            let price = match if is_crypto {
                fetch_binance_price(&client, symbol).await
            } else {
                fetch_yahoo_price(&client, symbol).await
            } {
                Ok(p) => {
                    current_prices.insert(symbol.to_string(), p);
                    p
                }
                Err(e) => {
                    // Micro-drift fallback so simulated positions still move
                    let micros = chrono::Utc::now().timestamp_micros();
                    let drift = ((micros % 2000) as f64 - 1000.0) / 1_000_000.0;
                    let p = old_price * (1.0 + drift);
                    current_prices.insert(symbol.to_string(), p);
                    eprintln!("[FeedWarn] {} API error: {}. Using drift: {:.2}", symbol, e, p);
                    p
                }
            };

            // Update unrealised P&L for any open position on this symbol
            let _ = portfolio_mgr.update_position_pnl(symbol, price).await;

            let change = price - old_price;
            let direction = if change >= 0.0 { TradeDirection::Long } else { TradeDirection::Short };

            // Hint SL/TP to the pipeline (LLM will override these)
            let sl_pct = if is_crypto { 0.012 } else { 0.006 };
            let tp_pct = if is_crypto { 0.030 } else { 0.018 };
            let stop   = if direction == TradeDirection::Long { price * (1.0 - sl_pct) } else { price * (1.0 + sl_pct) };
            let target = if direction == TradeDirection::Long { price * (1.0 + tp_pct) } else { price * (1.0 - tp_pct) };

            println!("\n──────── {} @ {}{:.2} ────────",
                symbol,
                if is_crypto { "$" } else { "₹" },
                price
            );

            // ── 2. Full autonomous pipeline (Kronos + LLM + execute) ──────────
            match orchestrator.run_full_pipeline(symbol, direction, price, stop, target).await {
                Ok(summary) => {
                    if summary.executed {
                        println!("✅ [Orchestrator] Trade EXECUTED | {}", summary.reason);
                    } else {
                        println!("⏸  [Orchestrator] No trade | {}", summary.reason);
                    }
                    // Print last LLM reasoning
                    let reason = orchestrator.state.last_llm_reason.read().await;
                    if !reason.is_empty() {
                        println!("🤖 [LLM Reason] {}", *reason);
                    }
                }
                Err(e) => {
                    eprintln!("❌ [Orchestrator] Pipeline error for {}: {}", symbol, e);
                }
            }
        }

        // ── 3. SL / TP monitoring & auto-exit ────────────────────────────────
        let _ = orchestrator.execution.run(None).await;

        // ── 4. Portfolio snapshot ─────────────────────────────────────────────
        {
            let p = orchestrator.state.portfolio.read().await;
            println!("\n📊 [Portfolio] Equity: ₹{:.2} | Cash: ₹{:.2} | Positions: {} | Today P&L: ₹{:.2} | DD: {:.2}%",
                p.total_equity, p.cash_balance,
                p.open_positions.len(),
                p.daily_pnl,
                p.max_drawdown_today * 100.0
            );
            if !p.open_positions.is_empty() {
                println!("   Open positions:");
                for pos in &p.open_positions {
                    println!("   • {} {} {} qty={:.0} entry={:.2} current={:.2} P&L=₹{:.2}",
                        pos.symbol,
                        if pos.direction == ates_core::TradeDirection::Long { "LONG" } else { "SHORT" },
                        if pos.unrealized_pnl >= 0.0 { "🟢" } else { "🔴" },
                        pos.quantity, pos.entry_price, pos.current_price, pos.unrealized_pnl
                    );
                }
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}
