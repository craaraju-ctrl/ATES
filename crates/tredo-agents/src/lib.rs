pub mod main_agents;
pub mod sub_agents;

// NOTE (duplication fix): This crate (tredo-agents) appears to be a parallel/legacy implementation
// of the agent hierarchy also present in tredo-autonomous (with main_agents/sub_agents mirroring
// the Tredo structure). The active code path used by tredo-orchestrator and Tauri is tredo-autonomous.
// This crate is kept for now but should be consolidated in a future refactor to avoid maintenance burden.
// (deprecated stub removed — tredo-agents duplicates tredo-autonomous; see comment above)

// Re-export common agents
pub use main_agents::*;
pub use sub_agents::*;