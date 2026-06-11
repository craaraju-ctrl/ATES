use chrono::{DateTime, Utc, Timelike, Datelike, Duration, Weekday, FixedOffset};
use ates_core::{DisciplineRules};
use crate::types::{TradeSignal, MarketRegime, SessionInfo};

pub fn get_indian_session_info(now: DateTime<Utc>) -> SessionInfo { /* implementation */ SessionInfo { market_open: true, session_name: "Normal".to_string(), time_to_close: None, time_to_open: None, is_pre_open: false, is_post_close: false, minutes_since_open: 0 } }

pub fn calculate_position_size(account_balance: f64, risk_pct: f64, entry_price: f64, stop_loss: f64) -> f64 { 0.0 }

pub fn calculate_risk_reward(entry: f64, stop: f64, target: f64, direction: ates_core::TradeDirection) -> f64 { 0.0 }

pub fn estimate_market_regime(prices: &[f64], highs: &[f64], lows: &[f64]) -> MarketRegime { MarketRegime::Ranging }

pub fn signal_quality_check(signal: &TradeSignal, rules: &DisciplineRules) -> (bool, Vec<String>) { (true, vec![]) }