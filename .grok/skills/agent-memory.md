---
name: agent-memory
description: Use agentmemory (via REST or MCP tools) for persistent project memory. Call when you need to recall past decisions, architecture, fixes, or store new learnings about tredo. Complements built-in Grok memory.
---
# Agent Memory Skill for tredo

When working on tredo:
- Use `agentmemory recall <topic>` or MCP tools to pull previous context (UI changes, LLM behavior, loop logic, fixes).
- After major changes or decisions, `agentmemory seed "key fact here"`.
- This gives the coding agent (Grok/Hermes) and potentially runtime agents infinite persistent memory across sessions.
