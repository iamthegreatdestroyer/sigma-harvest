---
name: sigma-harvest
description: ΣHARVEST - Web3 autonomous freebie collection system built with Tauri 2.0, React, and Rust
---

# ΣHARVEST Development Skill

## Project Overview
ΣHARVEST is a desktop application that autonomously discovers, evaluates, and claims Web3 freebies
(airdrops, faucets, free mints, quests) across multiple EVM chains.

## Architecture
- **Frontend**: React 19 + Tailwind 4 + Viem 2 + Zustand + Recharts
- **Backend**: Rust with Tauri 2.0 (Tokio async runtime)
- **Storage**: SQLite with AES-256-GCM encryption at rest
- **Design**: Cyberpunk terminal aesthetic (#00FF41 primary, #0A0E1A background)

## Key Modules (Rust - src-tauri/src/)
- `vault/` — BIP-39 seed, AES-256-GCM encryption, Argon2id KDF, HD wallet derivation
- `discovery/` — Multi-source scraping (DappRadar, Galxe, RSS, on-chain, social)
- `evaluation/` — Harvest Score 0-100 algorithm, risk assessment
- `executor/` — Transaction building, gas oracle, claim queue with retry
- `scraper/` — Pipeline orchestration, HTML parsing
- `analytics/` — ROI tracking, report generation
- `db/` — SQLite schema, migrations
- `ipc/` — Tauri command handlers (bridge to frontend)

## Key Modules (React - src/)
- `views/` — Dashboard, HuntConsole, WalletManager, OpportunityInspector, AnalyticsBay
- `stores/` — Zustand: appStore, walletStore, huntStore
- `hooks/` — useTauriCommand, useDiscovery, useWallets
- `components/` — CommandPalette (Ctrl+K), GasTicker, HarvestFeed, ScoreGauge

## Security Rules
- Private keys NEVER leave the Rust process
- All signing happens in Rust via `ring` crate
- Frontend only sees public addresses
- Database encrypted at rest (SQLCipher / AES-256-GCM + Argon2id)

## Supported Chains (Phase 1)
Ethereum, Arbitrum, Optimism, Base, Polygon, zkSync Era

## Coding Conventions
- Rust: Edition 2021, `thiserror` for error types, `tracing` for logging
- React: JSX (not TSX), Zustand for state, TanStack Query for async
- Styles: Tailwind 4 with custom theme tokens in globals.css
- Commits: Conventional commits (feat/fix/docs/refactor/test/chore)
