# ATES Development Roadmap

**From Foundation to Production**

## Phase 1: Foundation (Current)

- [x] Create new clean Rust repository
- [x] Define low-resource architecture
- [ ] Implement Disciplined Core in Rust
- [ ] Set up Rust workspace structure
- [ ] Create basic async Orchestrator

## Phase 2: Core Intelligence

- Implement full Disciplined Core with all trading rules
- Build Two-Tier Agent system (Main Agents + Sub-Agents)
- Add Memory system using embedded KV store (`redb`/`sled`)
- Create async message passing between agents

## Phase 3: Trading Agents

- Risk Manager Agent (Rust)
- Reflector / Critic Agent
- Market Intelligence Agent (Hybrid)
- Portfolio Manager Agent
- Execution Agent (with optimistic accounting)

## Phase 4: Observability & Control

- Lightweight Dashboard (axum)
- Prometheus metrics
- Health checks
- Telegram alerts
- Basic Web Control Panel

## Phase 5: Backtesting & Validation

- Historical data backtester
- Performance metrics (Win rate, Drawdown, etc.)
- Strategy configuration testing

## Phase 6: Execution & Safety

- Real broker integration (paper trading first)
- Multi-layer risk checks
- Kill switches and circuit breakers
- Position reconciliation

## Phase 7: Production

- Optimized Docker deployment
- Comprehensive monitoring & alerting
- Gradual live rollout
- Resource optimization for 8GB RAM

## Phase 8: Evolution

- Online learning from trade outcomes
- Self-improvement through memory
- Periodic strategy review

**Goal**: A stable, low-resource, high-discipline autonomous trading system that feels like a professional trading team.