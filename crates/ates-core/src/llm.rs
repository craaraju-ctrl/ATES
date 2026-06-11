use std::error::Error;

#[derive(Debug, Clone, Default)]
pub struct LlmExecutor {
    // Placeholder for Ollama / LLM client
}

impl LlmExecutor {
    pub fn new() -> Self { Self {} }

    pub async fn execute(&self, request: crate::messages::LLMRequest) -> Result<crate::messages::LLMResponse, Box<dyn Error + Send + Sync>> {
        println!("[LlmExecutor] Executing prompt...");
        Ok(crate::messages::LLMResponse {
            content: format!("Simulated LLM response to: {}", request.prompt),
            tokens_used: Some(42),
        })
    }
}