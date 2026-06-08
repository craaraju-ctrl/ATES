# ATES Agent Design

## Two-Tier Architecture

### Main Agents (LLM-capable)
Limited to 5–6 agents. These can call LLMs when needed.

| Agent | Role | LLM Usage | Key Responsibilities |
|-------|------|-----------|----------------------|
| Market Intelligence | Data fusion & regime detection | Low–Medium | Confluence scoring, technical + on-chain analysis |
| Strategy & Decision | Trade idea generation | Medium | Setup identification, bias determination |
| Risk & Psychology | Risk management + discipline | Low | Position sizing, drawdown control, red-folder enforcement |
| Reflector / Critic | Post-trade review & learning | Medium | Outcome analysis, pattern detection |
| Portfolio Manager | Overall exposure | Low | Correlation, capital allocation |
| Execution Coordinator | Final safety check | Very Low | Slippage, liquidity, kill-switch |

### Sub-Agents (Pure Logic)
Lightweight and fast. No LLM calls.

**Categories**:
- Technical Sub-Agents (Pivot Calculator, Confluence Scorer, Session Timer)
- Risk Sub-Agents (Position Sizer, Drawdown Monitor)
- Psychology Sub-Agents (Red Folder Checker, Overtrading Preventer)
- Memory Sub-Agents (Outcome Logger, Pattern Retriever)

## Design Rules

- Sub-Agents must be **deterministic and fast**.
- Main Agents act as coordinators.
- Most decisions should be resolved by Sub-Agents + Disciplined Core.
- LLM is only used when uncertainty is high or synthesis is complex.

## Goal

Create a system where agents feel like **specialized trading professionals** rather than generic AI assistants.