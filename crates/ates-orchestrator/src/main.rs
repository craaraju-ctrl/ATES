use ates_autonomous::state::initialize_autonomous_system;
use ates_core::{TradeDirection, Agent};
use std::time::Duration;
use tokio::time::sleep;
use std::collections::HashMap;

// Helper to fetch live crypto prices from Binance API
async fn fetch_binance_price(client: &reqwest::Client, symbol: &str) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}USDT", symbol);
    let resp: serde_json::Value = client.get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .timeout(Duration::from_secs(3))
        .send()
        .await?
        .json()
        .await?;
    let price_str = resp["price"].as_str().ok_or("price field missing")?;
    let price: f64 = price_str.parse()?;
    Ok(price)
}

// Helper to fetch live stock prices from Yahoo Finance API
async fn fetch_yahoo_price(client: &reqwest::Client, symbol: &str) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let yahoo_symbol = match symbol {
        "NIFTY" => "^NSEI",
        "RELIANCE" => "RELIANCE.NS",
        other => other,
    };
    let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1m&range=1d", yahoo_symbol);
    let resp: serde_json::Value = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
        .timeout(Duration::from_secs(3))
        .send()
        .await?
        .json()
        .await?;
    let price = resp["chart"]["result"][0]["meta"]["regularMarketPrice"]
        .as_f64()
        .ok_or("regularMarketPrice field missing")?;
    Ok(price)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== ATES Autonomous Orchestrator (Live 24x7 Multi-Market Feed) ===");

    // Initialize the real autonomous state
    let orchestrator = initialize_autonomous_system().await?;
    let client = reqwest::Client::new();

    // In-memory fallback tracking for 24x7 continuous uptime
    let mut current_prices = HashMap::new();
    current_prices.insert("NIFTY".to_string(), 24500.0);
    current_prices.insert("RELIANCE".to_string(), 2950.0);
    current_prices.insert("BTC".to_string(), 67500.0);
    current_prices.insert("ETH".to_string(), 3500.0);
    current_prices.insert("SOL".to_string(), 155.0);

    let assets = vec!["NIFTY", "RELIANCE", "BTC", "ETH", "SOL"];
    println!("[Orchestrator] Active and ready. Running pipeline loops for: {:?}", assets);

    loop {
        for symbol in &assets {
            let is_crypto = match *symbol {
                "BTC" | "ETH" | "SOL" => true,
                _ => false,
            };

            // Attempt to pull real-time ticks
            let old_price = *current_prices.get(*symbol).unwrap_or(&20000.0);
            let price;
            let mut source = "Live Stream API";

            let fetch_result = if is_crypto {
                fetch_binance_price(&client, symbol).await
            } else {
                fetch_yahoo_price(&client, symbol).await
            };

            match fetch_result {
                Ok(p) => {
                    price = p;
                    current_prices.insert(symbol.to_string(), price);
                }
                Err(e) => {
                    // Failover drift logic for offline/off-market hours
                    let micros = chrono::Utc::now().timestamp_micros();
                    let random_pct = ((micros % 2000) as f64 - 1000.0) / 1000000.0; // small drift
                    price = old_price * (1.0 + random_pct);
                    current_prices.insert(symbol.to_string(), price);
                    source = "Fallback Drift (API Timeout / Market Closed)";
                    eprintln!("[Feed Warning] Price fetch error for {}: {}. Drifted to: {}", symbol, e, price);
                }
            }

            let change = price - old_price;
            let direction = if change >= 0.0 { TradeDirection::Long } else { TradeDirection::Short };

            // Determine parameters
            let entry = price;
            let sl_pct = if is_crypto { 0.01 } else { 0.005 }; // 1.0% crypto vs 0.5% stocks
            let tp_pct = if is_crypto { 0.025 } else { 0.015 }; // 2.5% crypto vs 1.5% stocks

            let stop = if direction == TradeDirection::Long { entry * (1.0 - sl_pct) } else { entry * (1.0 + sl_pct) };
            let target = if direction == TradeDirection::Long { entry * (1.0 + tp_pct) } else { entry * (1.0 - tp_pct) };

            println!("\n--------------------------------------------------");
            println!(
                "[Tick Feed] {} @ {} {:.2} ({})",
                symbol,
                if is_crypto { "$" } else { "₹" },
                price,
                source
            );

            // Execute core agent decision engines
            match orchestrator.run_full_pipeline(symbol, direction, entry, stop, target).await {
                Ok(summary) => {
                    if summary.executed {
                        println!("[Orchestrator] Setup PASSED Confluences. Trade executed! Reason: {}", summary.reason);
                    } else {
                        println!("[Orchestrator] Setup REJECTED or Paper conditions not met. Reason: {}", summary.reason);
                    }
                }
                Err(e) => {
                    println!("[Orchestrator] Pipeline process error: {}", e);
                }
            }
        }

        // Check active positions and trigger automated Stop Loss / Take Profit exits
        let _ = orchestrator.execution.run(None).await;

        // Print active margin balance summary
        {
            let portfolio = orchestrator.state.portfolio.read().await;
            println!(
                "[Portfolio Status] Equity: {:.2} | Margin Cash: {:.2} | Positions: {} | Today's P&L: {:.2}",
                portfolio.total_equity, portfolio.cash_balance, portfolio.open_positions.len(), portfolio.daily_pnl
            );
        }

        sleep(Duration::from_secs(5)).await;
    }
}
