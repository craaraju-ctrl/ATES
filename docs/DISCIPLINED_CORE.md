# ATES Disciplined Core Specification

## Purpose

The Disciplined Core is the **non-negotiable foundation** of ATES. It encodes professional trading discipline so that agents make consistent, rule-based decisions instead of relying purely on LLM prompting.

## Core Categories

### 1. Technical Rules
- Daily Pivot Points (Classic method)
- Support & Resistance levels
- 200 EMA trend filter
- Session Timing (London & New York focus)

### 2. Confluence Requirements
Must consider multiple factors before taking a trade:
- DXY movement
- 10-Year Treasury Yields
- BTC Dominance
- Funding Rates
- On-Chain flows

### 3. Risk Management (Hard Rules)
- Maximum risk per trade: 1% of account
- Maximum daily drawdown: 3% (drawdown limit triggers halt state)
- Position sizing based on stop distance
- No trading if daily loss limit is hit
- Dynamic accounting support for both **LONG and SHORT** positions with correct unrealized P&L, cash balance, and equity contribution calculations

### 4. Psychology & Discipline
- Red Folder / High-impact news filter (synchronized to Indian Standard Time (IST) offset)
- Reduce position size after consecutive losses
- Avoid overtrading (limit enforced via consensus of sub-agents)
- Respect session timing (enforced via Session Timer sub-agent)

### 5. Entry Criteria
A trade must pass structured checks:
- Minimum confluence score
- Favorable session
- No red-folder event
- Risk parameters approved

## Implementation Principles

- Written in **Rust** for speed and reliability
- Loaded at startup
- Sub-Agents can make many decisions using only this core
- Main Agents consult it before using LLM

## Goal

Create agents that behave like **experienced, disciplined traders** who follow rules first and use intelligence second.