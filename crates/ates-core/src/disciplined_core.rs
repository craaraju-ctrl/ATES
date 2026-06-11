use chrono::{DateTime, Utc, Timelike};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketContext {
    pub symbol: String,
    pub current_price: f64,
    pub high: f64,
    pub low: f64,
    pub previous_close: f64,
    pub timestamp: DateTime<Utc>,
    pub daily_pnl: f64,
    pub consecutive_losses: u32,
    pub is_red_folder_day: bool,
    pub trend_direction: Option<TrendDirection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Bullish,
    Bearish,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotLevels {
    pub pivot: f64,
    pub r1: f64,
    pub r2: f64,
    pub r3: f64,
    pub s1: f64,
    pub s2: f64,
    pub s3: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisciplineCheck {
    pub passed: bool,
    pub reasons: Vec<String>,
    pub confluence_score: Option<f64>,
}

// ... rest of the file (functions) remain the same ...