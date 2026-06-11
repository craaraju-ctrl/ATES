use chrono::{DateTime, Utc, Timelike};

#[derive(Debug, Clone)]
pub struct DisciplineRules {
    pub use_daily_pivots: bool,
    pub pivot_method: PivotMethod,
    pub respect_session_timing: bool,
    pub london_open_hour: u32,
    pub london_close_hour: u32,
    pub ny_open_hour: u32,
    pub ny_close_hour: u32,
    pub max_risk_per_trade: f64,
    pub max_daily_drawdown: f64,
    pub max_consecutive_losses: u32,
    pub use_confluence: bool,
    pub min_confluence_score: f64,
    pub red_folder_discipline: bool,
    pub require_trend_filter: bool,
}

impl Default for DisciplineRules {
    fn default() -> Self {
        Self {
            use_daily_pivots: true,
            pivot_method: PivotMethod::Classic,
            respect_session_timing: true,
            london_open_hour: 8,
            london_close_hour: 16,
            ny_open_hour: 13,
            ny_close_hour: 21,
            max_risk_per_trade: 0.01,
            max_daily_drawdown: 0.03,
            max_consecutive_losses: 3,
            use_confluence: true,
            min_confluence_score: 0.65,
            red_folder_discipline: true,
            require_trend_filter: true,
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    Bullish,
    Bearish,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct PivotLevels {
    pub pivot: f64,
    pub r1: f64,
    pub r2: f64,
    pub r3: f64,
    pub s1: f64,
    pub s2: f64,
    pub s3: f64,
}

#[derive(Debug, Clone)]
pub struct DisciplineCheck {
    pub passed: bool,
    pub reasons: Vec<String>,
    pub confluence_score: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PivotMethod {
    Classic,
    Woodie,
    Fibonacci,
}

pub fn calculate_pivot_points(high: f64, low: f64, close: f64, method: PivotMethod) -> PivotLevels {
    let pivot = match method {
        PivotMethod::Classic => (high + low + close) / 3.0,
        PivotMethod::Woodie => (high + low + 2.0 * close) / 4.0,
        PivotMethod::Fibonacci => (high + low + close) / 3.0,
    };
    let r1 = 2.0 * pivot - low;
    let s1 = 2.0 * pivot - high;
    PivotLevels {
        pivot,
        r1,
        r2: pivot + (high - low),
        r3: high + 2.0 * (pivot - low),
        s1,
        s2: pivot - (high - low),
        s3: low - 2.0 * (high - pivot),
    }
}

pub fn is_in_trading_session(timestamp: DateTime<Utc>, rules: &DisciplineRules) -> bool {
    if !rules.respect_session_timing { return true; }
    let hour = timestamp.hour();
    let in_london = hour >= rules.london_open_hour && hour < rules.london_close_hour;
    let in_ny = hour >= rules.ny_open_hour && hour < rules.ny_close_hour;
    in_london || in_ny
}

pub fn check_risk_limits(context: &MarketContext, rules: &DisciplineRules) -> DisciplineCheck {
    let mut reasons = Vec::new();
    let mut passed = true;
    if context.daily_pnl <= -rules.max_daily_drawdown {
        reasons.push(format!("Daily drawdown limit reached: {:.2}%", rules.max_daily_drawdown * 100.0));
        passed = false;
    }
    if context.consecutive_losses >= rules.max_consecutive_losses {
        reasons.push(format!("Maximum consecutive losses ({}) reached", rules.max_consecutive_losses));
        passed = false;
    }
    if context.is_red_folder_day && rules.red_folder_discipline {
        reasons.push("Red folder / high-impact event day – trading restricted".to_string());
        passed = false;
    }
    DisciplineCheck { passed, reasons, confluence_score: None }
}

pub fn calculate_confluence_score(context: &MarketContext, pivots: &PivotLevels) -> f64 {
    let mut score: f64 = 0.5;
    let distance_to_pivot = (context.current_price - pivots.pivot).abs() / context.current_price;
    if distance_to_pivot < 0.002 { score += 0.15; }
    if let Some(trend) = context.trend_direction {
        match trend {
            TrendDirection::Bullish if context.current_price > pivots.pivot => score += 0.2,
            TrendDirection::Bearish if context.current_price < pivots.pivot => score += 0.2,
            _ => {}
        }
    }
    score.min(1.0)
}

pub fn validate_trade_setup(context: &MarketContext, rules: &DisciplineRules) -> DisciplineCheck {
    let mut all_reasons = Vec::new();
    let mut overall_passed = true;

    if !is_in_trading_session(context.timestamp, rules) {
        all_reasons.push("Outside allowed trading sessions (London/NY)".to_string());
        overall_passed = false;
    }

    let risk_check = check_risk_limits(context, rules);
    if !risk_check.passed {
        all_reasons.extend(risk_check.reasons);
        overall_passed = false;
    }

    let mut confluence_score = None;
    if rules.use_daily_pivots {
        let pivots = calculate_pivot_points(context.high, context.low, context.previous_close, rules.pivot_method);
        if rules.use_confluence {
            let score = calculate_confluence_score(context, &pivots);
            confluence_score = Some(score);
            if score < rules.min_confluence_score {
                all_reasons.push(format!("Confluence score too low: {:.2}", score));
                overall_passed = false;
            }
        }
    }

    DisciplineCheck { passed: overall_passed, reasons: all_reasons, confluence_score }
}

pub fn get_discipline_summary() -> &'static str {
    "Disciplined Core v0.2 – Pivot points, session timing, risk limits, and confluence active"
}