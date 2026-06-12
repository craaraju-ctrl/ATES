use std::error::Error;
use reqwest::Client;
use serde_json::json;
use serde::Deserialize;

/// Structured trade decision returned by the LLM.
#[derive(Debug, Clone)]
pub struct LlmTradeDecision {
    /// "BUY", "SELL", or "HOLD"
    pub action: String,
    pub entry: f64,
    pub sl: f64,
    pub tp: f64,
    pub reason: String,
}

impl Default for LlmTradeDecision {
    fn default() -> Self {
        Self {
            action: "HOLD".to_string(),
            entry: 0.0,
            sl: 0.0,
            tp: 0.0,
            reason: "LLM fallback: defaulting to HOLD".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LlmExecutor {
    client: Client,
    model: String,
    endpoint: String,
}

impl Default for LlmExecutor {
    fn default() -> Self {
        Self {
            client: Client::new(),
            model: std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "ministral-3:3b-cloud".to_string()),
            endpoint: std::env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| "http://localhost:11434/api/generate".to_string()),
        }
    }
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    eval_count: Option<u32>,
}

impl LlmExecutor {
    pub fn new() -> Self { Self::default() }

    /// Generic prompt execution — returns raw response string.
    pub async fn execute(&self, request: crate::messages::LLMRequest) -> Result<crate::messages::LLMResponse, Box<dyn Error + Send + Sync>> {
        println!("[LlmExecutor] Sending prompt to Ollama model: {}", self.model);

        let body = json!({
            "model": self.model,
            "prompt": request.prompt,
            "stream": false
        });

        let res = self.client.post(&self.endpoint)
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(format!("Ollama API error: {} - {}", status, error_text).into());
        }

        let ollama_res: OllamaResponse = res.json().await?;

        Ok(crate::messages::LLMResponse {
            content: ollama_res.response,
            tokens_used: ollama_res.eval_count,
        })
    }

    /// Ask the LLM to produce a structured trade decision (BUY/SELL/HOLD).
    /// Returns LlmTradeDecision::default() (HOLD) on any error, so the pipeline always continues.
    pub async fn ask_for_trade_decision(
        &self,
        symbol: &str,
        price: f64,
        confluence: f64,
        trend: &str,
        pivot: f64,
        r1: f64,
        s1: f64,
        forecast_summary: &str,
        portfolio_heat: f64,
        session_open: bool,
        consecutive_losses: u32,
    ) -> LlmTradeDecision {
        // Build the structured prompt
        let sl_long  = price * 0.990;
        let tp_long  = price * 1.025;
        let sl_short = price * 1.010;
        let tp_short = price * 0.975;

        let prompt = format!(
            r#"You are an autonomous trading agent for Indian and crypto markets.
Analyze the following market data and decide whether to BUY, SELL, or HOLD.

Market Context:
- Symbol: {symbol}
- Current Price: {price:.2}
- Kronos 5-bar Forecast Summary: {forecast_summary}
- Trend Direction: {trend}
- Confluence Score: {confluence:.1}%
- Pivot: {pivot:.2} | R1: {r1:.2} | S1: {s1:.2}
- Portfolio Heat: {portfolio_heat:.1}%
- Session Open: {session_open}
- Consecutive Losses: {consecutive_losses}

Rules you MUST follow:
1. Only BUY or SELL if confluence > 60%, otherwise HOLD.
2. For BUY: sl must be BELOW entry, tp must be ABOVE entry.
3. For SELL: sl must be ABOVE entry, tp must be BELOW entry.
4. Risk:Reward must be >= 2:1.
5. HOLD if session is closed (Indian markets 09:15-15:30 IST, crypto 24x7).
6. HOLD if consecutive_losses >= 3.
7. Suggested entry is current price. Suggested SL for BUY={sl_long:.2}, TP for BUY={tp_long:.2}. SL for SELL={sl_short:.2}, TP for SELL={tp_short:.2}.

Respond ONLY with a single line of valid JSON — no markdown, no explanation outside JSON:
{{"action":"BUY","entry":{price:.2},"sl":{sl_long:.2},"tp":{tp_long:.2},"reason":"Your short reason here"}}
"#,
            symbol = symbol,
            price = price,
            forecast_summary = forecast_summary,
            trend = trend,
            confluence = confluence * 100.0,
            pivot = pivot,
            r1 = r1,
            s1 = s1,
            portfolio_heat = portfolio_heat * 100.0,
            session_open = session_open,
            consecutive_losses = consecutive_losses,
            sl_long = sl_long,
            tp_long = tp_long,
            sl_short = sl_short,
            tp_short = tp_short,
        );

        println!("[LlmExecutor] Requesting trade decision from Ollama for {} @ {:.2}", symbol, price);

        let body = json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false
        });

        let res = match self.client
            .post(&self.endpoint)
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                println!("[LlmExecutor] ⚠ Ollama request failed (network): {}. Defaulting to HOLD.", e);
                return LlmTradeDecision::default();
            }
        };

        let status = res.status();
        if !status.is_success() {
            let err_text = res.text().await.unwrap_or_default();
            println!("[LlmExecutor] ⚠ Ollama error {}: {}. Defaulting to HOLD.", status, err_text.trim());
            return LlmTradeDecision::default();
        }

        let ollama_res: OllamaResponse = match res.json().await {
            Ok(r) => r,
            Err(e) => {
                println!("[LlmExecutor] ⚠ Failed to parse Ollama response: {}. Defaulting to HOLD.", e);
                return LlmTradeDecision::default();
            }
        };

        println!("[LlmExecutor] Raw LLM response: {}", ollama_res.response.trim());
        Self::parse_llm_trade_decision(&ollama_res.response, price)
    }

    /// Parse the JSON trade decision from the LLM response text.
    /// Robust: finds the first `{...}` block, handles extra text.
    pub fn parse_llm_trade_decision(raw: &str, current_price: f64) -> LlmTradeDecision {
        // Find JSON object in response
        let start = raw.find('{');
        let end   = raw.rfind('}');

        if let (Some(s), Some(e)) = (start, end) {
            if s <= e {
                let json_str = &raw[s..=e];
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
                    let action = v["action"].as_str().unwrap_or("HOLD").to_uppercase();
                    let entry  = v["entry"].as_f64().unwrap_or(current_price);
                    let sl     = v["sl"].as_f64().unwrap_or(0.0);
                    let tp     = v["tp"].as_f64().unwrap_or(0.0);
                    let reason = v["reason"].as_str().unwrap_or("LLM provided no reason").to_string();

                    // Sanity-check: for BUY sl < entry < tp, for SELL tp < entry < sl
                    let valid = match action.as_str() {
                        "BUY"  => sl > 0.0 && tp > 0.0 && sl < entry && tp > entry,
                        "SELL" => sl > 0.0 && tp > 0.0 && sl > entry && tp < entry,
                        _      => true, // HOLD is always valid
                    };

                    if valid {
                        println!("[LlmExecutor] ✅ Parsed decision: {} entry={:.2} sl={:.2} tp={:.2}", action, entry, sl, tp);
                        return LlmTradeDecision { action, entry, sl, tp, reason };
                    } else {
                        println!("[LlmExecutor] ⚠ LLM returned invalid SL/TP for {}. Defaulting to HOLD.", action);
                    }
                }
            }
        }

        println!("[LlmExecutor] ⚠ Could not parse LLM JSON response. Defaulting to HOLD.");
        LlmTradeDecision {
            reason: format!("Parse failed for: {}", &raw[..raw.len().min(120)]),
            ..LlmTradeDecision::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::LLMRequest;

    #[tokio::test]
    #[ignore] // Run manually via: cargo test -p ates-core -- --ignored test_ollama_inference
    async fn test_ollama_inference() {
        let executor = LlmExecutor::new();
        let request = LLMRequest {
            request_id: "test-req".to_string(),
            agent_role: crate::role::AgentRole::MarketIntelligence,
            prompt: "What is 2 + 2? Answer in one short sentence.".to_string(),
            context: serde_json::json!({}),
            max_tokens: 50,
            temperature: 0.1,
        };

        let result = executor.execute(request).await.expect("Failed to execute LLM request");
        println!("Response: {}", result.content);
        println!("Tokens used: {:?}", result.tokens_used);

        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    #[ignore] // Run manually via: cargo test -p ates-core -- --ignored test_trade_decision
    async fn test_trade_decision() {
        let executor = LlmExecutor::new();
        let decision = executor.ask_for_trade_decision(
            "NIFTY", 24500.0, 0.75, "Bullish",
            24400.0, 24600.0, 24200.0,
            "Kronos predicts +0.5% over next 5 candles",
            0.05, true, 0,
        ).await;

        println!("Action: {} | Entry: {:.2} | SL: {:.2} | TP: {:.2}", decision.action, decision.entry, decision.sl, decision.tp);
        println!("Reason: {}", decision.reason);
    }

    #[test]
    fn test_parse_llm_decision_valid_buy() {
        let raw = r#"{"action":"BUY","entry":24500.0,"sl":24250.0,"tp":25000.0,"reason":"Bullish trend"}"#;
        let d = LlmExecutor::parse_llm_trade_decision(raw, 24500.0);
        assert_eq!(d.action, "BUY");
        assert_eq!(d.entry, 24500.0);
    }

    #[test]
    fn test_parse_llm_decision_invalid_sl() {
        // SL above entry for BUY — should default to HOLD
        let raw = r#"{"action":"BUY","entry":24500.0,"sl":24800.0,"tp":25000.0,"reason":"Bad SL"}"#;
        let d = LlmExecutor::parse_llm_trade_decision(raw, 24500.0);
        assert_eq!(d.action, "HOLD");
    }

    #[test]
    fn test_parse_llm_decision_garbage() {
        let raw = "Sorry, I cannot process this request right now.";
        let d = LlmExecutor::parse_llm_trade_decision(raw, 24500.0);
        assert_eq!(d.action, "HOLD");
    }
}