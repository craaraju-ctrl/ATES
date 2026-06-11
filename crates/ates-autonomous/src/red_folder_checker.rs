use async_trait::async_trait;
use std::error::Error;
use chrono::Utc;

use crate::state::SharedState;
use ates_core::{Agent, AgentInput, AgentOutput, DisciplineCheck};

/// SUB AGENT 6 — RedFolderCheckerAgent (Deterministic)
/// Checks for high-impact economic events that warrant trading restrictions
pub struct RedFolderCheckerAgent {
    pub state: SharedState,
    /// List of known high-impact event dates (YYYY-MM-DD format)
    pub red_folder_dates: Vec<String>,
}

impl RedFolderCheckerAgent {
    pub fn new(state: SharedState) -> Self {
        // Pre-load common Indian market high-impact dates
        let red_folder_dates = vec![
            "2026-02-06".to_string(), "2026-04-08".to_string(),
            "2026-06-05".to_string(), "2026-08-07".to_string(),
            "2026-10-08".to_string(), "2026-12-03".to_string(),
            "2026-02-01".to_string(), // Union Budget
            "2026-04-15".to_string(), "2026-07-15".to_string(),
            "2026-10-15".to_string(), "2026-01-15".to_string(),
        ];

        Self { state, red_folder_dates }
    }

    fn is_red_folder_today(&self) -> bool {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        self.red_folder_dates.contains(&today)
    }

    /// Check if any red folder event is within the next N days
    fn upcoming_red_folder(&self, days: i64) -> Vec<String> {
        let now = Utc::now();
        self.red_folder_dates.iter()
            .filter(|date| {
                if let Ok(event_date) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                    let event_datetime = event_date.and_hms_opt(0, 0, 0).unwrap();
                    let diff = event_datetime.signed_duration_since(now.naive_utc()).num_days();
                    diff > 0 && diff <= days
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
}

#[async_trait]
impl Agent for RedFolderCheckerAgent {
    fn name(&self) -> &str { "RedFolderCheckerAgent" }
    fn tier(&self) -> ates_core::AgentTier { ates_core::AgentTier::Sub }

    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        let is_red = self.is_red_folder_today();
        let upcoming = self.upcoming_red_folder(3);

        if is_red {
            println!("[RedFolderChecker] 🚨 RED FOLDER DAY - Trading restricted");
        } else if !upcoming.is_empty() {
            println!("[RedFolderChecker] ⚠️ Upcoming high-impact event in next 3 days: {:?}", upcoming);
        } else {
            println!("[RedFolderChecker] ✅ No red folder events today");
        }

        let check = DisciplineCheck {
            passed: !is_red,
            reasons: if is_red { vec!["Red folder event today - trading restricted".to_string()] } else { vec![] },
            confluence_score: None,
        };

        Ok(AgentOutput::RiskResult(check))
    }
}