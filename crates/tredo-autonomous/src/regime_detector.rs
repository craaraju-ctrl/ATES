// RegimeDetector helper (new skill/tool)
// HMM-inspired regime detection (vol + slope) for adaptive strategies.
// Research: Boosts robustness like in QuantStart/TradingAgents regime filters.
// Pure helper.

use crate::state::SharedState;
use crate::types::MarketRegime;

pub struct RegimeDetector {
    pub state: SharedState,
}

impl RegimeDetector {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    pub async fn detect_regime(&self, symbol: &str, price: f64) -> MarketRegime {
        let history = self.state.ohlcv_history.read().await;
        if let Some(bars) = history.get(symbol) {
            if bars.len() < 20 { return MarketRegime::Ranging; }
            let recent = &bars[bars.len()-10..];
            let vol: f64 = recent.windows(2).map(|w| (w[1].close - w[0].close).abs() / w[0].close).sum::<f64>() / 9.0;
            let slope = (price - bars[bars.len()-10].close) / bars[bars.len()-10].close;

            if vol > 0.025 { return MarketRegime::Volatile; }
            if slope > 0.02 { return MarketRegime::TrendingBull; }
            if slope < -0.02 { return MarketRegime::TrendingBear; }
            return MarketRegime::Ranging;
        }
        MarketRegime::Ranging
    }
}
