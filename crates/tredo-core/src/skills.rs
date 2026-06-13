// AgentSkill trait - new "skills" abstraction for pluggable capabilities in agents.
// This allows easy addition of tools/skills like sentiment, on-chain, vol calc, etc.
// Inspired by research (TradingAgents tools per role, FINCON modular perception/memory/action).

use async_trait::async_trait;
use std::error::Error;
use crate::agent::{AgentInput, AgentOutput};

#[async_trait]
pub trait AgentSkill: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, input: &AgentInput) -> Result<AgentOutput, Box<dyn Error + Send + Sync>>;
    fn is_available(&self) -> bool { true }
}

// Example skill wrapper to turn sub-agents into skills if needed.
pub struct SkillWrapper<S: AgentSkill + ?Sized> {
    inner: Box<S>,
}

impl<S: AgentSkill> SkillWrapper<S> {
    pub fn new(skill: S) -> Self {
        Self { inner: Box::new(skill) }
    }
}

#[async_trait]
impl<S: AgentSkill> AgentSkill for SkillWrapper<S> {
    fn name(&self) -> &str { self.inner.name() }
    fn description(&self) -> &str { self.inner.description() }
    async fn execute(&self, input: &AgentInput) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        self.inner.execute(input).await
    }
}
