# ATES — Autonomous Trading Execution System

**Advanced Rust Multi-Agent Trading Platform**

ATES is a **production-grade**, Rust-first hierarchical multi-agent trading system focused on Indian markets (NSE/BSE). It enforces a strong **Disciplined Core** of professional trading rules while incorporating memory-driven learning and selective LLM usage.

## Core Philosophy
- **Rules + Memory > Pure Prompting**
- Strict **Two-Tier Architecture**: Main Agents (LLM-capable) coordinate; Sub-Agents are deterministic and lightweight
- Strong emphasis on risk management, confluence, session timing, and psychology
- Designed for reliability, observability, and safe execution

## Key Features
- Comprehensive **Disciplined Core** (pivots, confluence scoring, risk limits, red-folder handling)
- Robust **Discipline Check Consensus** requiring consensus across all sub-agents (session, drawdown, red-folder, and overtrading checks)
- **Dynamic Short Selling Accounting** with correct realized P&L, cash refunds, and equity contributions under simulation
- **Timezone-Aware Operations** syncing economic event checks to Indian Standard Time (IST)
- Async Tokio-based orchestration
- Embedded memory (`redb`)
- LLM Executor (Ollama integration with fallback)
- Backtesting engine
- Paper trading execution layer with broker adapter stubs (Kite/Zerodha dummy keys)
- **Tauri + Leptos** native desktop UI
- Full message passing between agents

## Project Status
**Production Prototype Ready** — Full backend with Disciplined Core, Execution Engine, and Tauri + Leptos desktop UI implemented. Real backend commands connected. `tredo` wake-up system available.

**Important**: 
- Dummy API keys are in `crates/ates-core/src/config.rs`. **Replace before any live/paper trading**.
- This is a research/educational prototype. Extensive testing required before real capital use.

## Quick Start (Development)

```bash
# Run backend only
cargo run -p ates-orchestrator

# Run with Tauri UI (recommended)
cd src-tauri && cargo tauri dev
```

## Production / Deployment

```bash
# Build release
cargo build --release -p ates-ui

# Or use Docker
docker build -t ates .
docker run -it ates
```

**Security Notes**:
- Never commit real API keys
- Use environment variables or secure secret management in production
- Always run extensive paper trading validation before live use

## Technology Stack

- **Core**: Rust + Tokio
- **State & KV Cache**: redb
- **LLM**: Ollama (selective)
- **Execution**: Paper + Broker adapter stubs
- **Observability**: Axum-ready

## Disclaimer
ATES is a research and educational tool. It is **not financial advice**.