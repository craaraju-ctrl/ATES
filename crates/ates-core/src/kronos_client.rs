use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// OHLCV bar for Kronos input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvBar {
    pub timestamp: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Request payload for Kronos forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KronosForecastRequest {
    pub symbol: String,
    pub ohlcv: Vec<OhlcvBar>,
    pub pred_len: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub sample_count: u32,
}

/// Response from Kronos service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KronosForecastResponse {
    pub symbol: String,
    pub forecasts: Vec<serde_json::Value>,
    pub message: String,
}

/// HTTP client to communicate with the Kronos Forecasting Service
#[derive(Clone)]
pub struct KronosClient {
    client: Client,
    base_url: String,
}

impl KronosClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    /// Call the /forecast endpoint of the Kronos service
    pub async fn forecast(
        &self,
        request: KronosForecastRequest,
    ) -> Result<KronosForecastResponse, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/forecast", self.base_url.trim_end_matches('/'));
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json::<KronosForecastResponse>()
            .await?;
        Ok(response)
    }
}
