use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}