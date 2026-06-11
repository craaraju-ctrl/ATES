#[derive(Debug, Clone)]
pub struct Config {
    pub initial_balance: f64,
    pub max_position_size: f64,
    pub api_key: String,
    pub api_secret: String,
    /// URL of the Kronos Forecasting Service (FastAPI)
    pub kronos_service_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            initial_balance: 100_000.0,
            max_position_size: 0.05,
            api_key: "DUMMY_API_KEY".to_string(),
            api_secret: "DUMMY_API_SECRET".to_string(),
            kronos_service_url: "http://localhost:8000".to_string(),
        }
    }
}
