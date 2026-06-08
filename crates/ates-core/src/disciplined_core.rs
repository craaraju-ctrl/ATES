//! ATES Disciplined Core

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DisciplineRules {
    pub use_daily_pivots: bool,
    pub use_confluence: bool,
    pub respect_session_timing: bool,
    pub red_folder_discipline: bool,
    pub max_risk_per_trade: f64,
    pub max_daily_drawdown: f64,
}

impl Default for DisciplineRules {
    fn default() -> Self {
        Self {
            use_daily_pivots: true,
            use_confluence: true,
            respect_session_timing: true,
            red_folder_discipline: true,
            max_risk_per_trade: 0.01,
            max_daily_drawdown: 0.03,
        }
    }
}

pub fn get_discipline_summary() -> &'static str {
    "Disciplined Core loaded successfully"
}