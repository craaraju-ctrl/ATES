// VolatilityCalculator helper (new tool/skill)
// Computes ATR, expansion for breakout/vol strategies.
// Upgrades risk and MI with regime-aware vol detection.
// Note: Pure helper (methods only), not full Agent impl, to match core AgentInput enum.

use crate::state::SharedState;

pub struct VolatilityCalculator {
    pub state: SharedState,
}

impl VolatilityCalculator {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    pub async fn compute_volatility(&self, symbol: &str, price: f64) -> (f64, bool) {
        let history = self.state.ohlcv_history.read().await;
        if let Some(bars) = history.get(symbol) {
            if bars.len() < 15 { return (0.01, false); }
            let mut atr = 0.0;
            for i in (bars.len()-14)..bars.len() {
                let range = (bars[i].high - bars[i].low).abs();
                atr += range;
            }
            atr /= 14.0;
            let atr_pct = atr / price;
            let expansion = atr_pct > 0.015; // vol expansion threshold
            return (atr_pct, expansion);
        }
        (0.01, false)
    }
}
