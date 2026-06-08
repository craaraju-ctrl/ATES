# ATES Low-Resource Architecture (8GB RAM Optimized)

**Version**: 1.0  
**Goal**: Build a high-performance, low-memory, trading-dedicated multi-agent system with minimal LLM dependency.

## 1. Hardware & Performance Constraints

- Target: Maximum 8GB RAM
- Focus: Maximum performance with minimum computation
- LLM Policy: Use only when necessary (high uncertainty or complex reasoning)

## 2. Core Philosophy

- **Disciplined Core First**: All agents must respect hard trading rules.
- **Two-Tier Agent Architecture**:
  - **Main Agents** (5–6): LLM-capable coordinators
  - **Sub-Agents**: Lightweight, pure logic agents (no LLM)
- **Hybrid Intelligence**: Rules + Memory > Prompting
- **Dedicated Trading Personality**: Agents should feel like experienced traders, not generic bots.

## 3. Disciplined Core

The heart of the system. Contains non-negotiable rules:
- Pivots, Support/Resistance
- Multi-factor Confluence
- Red-Folder & High-Impact Events
- Session Timing (London/NY focus)
- Risk Management & Psychology rules
- Structured Entry Criteria

Implemented in Rust for speed and safety.

## 4. Technology Stack

- **Core Language**: Rust + Tokio
- **State & KV Cache**: redb or sled (embedded)
- **LLM**: Ollama (selective use)
- **Dashboard**: axum (planned)

## 5. LLM Usage Policy

LLM is treated as a scarce resource. Used only for:
- High uncertainty
- Complex synthesis
- Reflection & learning

Most decisions should be handled by rules + memory + Sub-Agents.

## 6. Next Steps

1. Implement Disciplined Core in Rust
2. Build async Orchestrator
3. Create Main Agents and Sub-Agents
4. Add Memory system

This architecture is designed to deliver strong trading intelligence while staying within tight resource constraints.