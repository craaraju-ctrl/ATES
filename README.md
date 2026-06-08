# ATES — Agentic Trading Exception System

**Low-resource. High-discipline. Dedicated trading intelligence.**

ATES is a **Rust-first**, production-oriented multi-agent trading system designed to run efficiently on constrained hardware (target: 8GB RAM class).

It combines professional trading discipline with adaptive, memory-driven intelligence while keeping LLM usage minimal.

Built for traders who want **real structure and performance** — not generic chatbot behavior.

## Core Philosophy

- **Rules + Memory > Prompting**
- **Two-Tier Architecture**: Small number of LLM-capable Main Agents + many lightweight Sub-Agents
- **Dedicated Trading Personality**: Agents behave like experienced, disciplined traders
- **Maximum Performance, Minimum Computation**

## Key Features

- Strong **Disciplined Core** (pivots, confluence, risk, psychology, session timing)
- Hybrid intelligence (rules + memory + selective LLM)
- Async Rust architecture (Tokio)
- Low memory footprint using embedded KV stores
- Full observability and control

## Architecture

See `docs/ATES_LOW_RESOURCE_ARCHITECTURE.md` for the detailed design.

## Technology Stack

- **Core**: Rust + Tokio
- **State & KV Cache**: redb / sled
- **LLM**: Ollama (used selectively)
- **Dashboard**: axum (planned)

## Project Status

Early development. Currently building the Rust foundation.

## Disclaimer
ATES is a research and educational tool. It is **not financial advice**.