use crate::Config;

#[derive(Debug, Clone)]
pub struct KitePaperAdapter {
    pub config: Config,
}

impl KitePaperAdapter {
    pub fn new(config: Config) -> Self { Self { config } }

    pub async fn place_order(&self, _symbol: &str, _direction: &str, _qty: i32) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        println!("[KitePaperAdapter] Simulating order...");
        Ok("ORDER12345".to_string())
    }
}