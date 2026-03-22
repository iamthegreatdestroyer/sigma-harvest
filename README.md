# ΣHARVEST

> Sub-Linear Autonomous Resource Vacuum for Web3 Tokens

A Tauri 2.0 desktop application that autonomously discovers, evaluates, and claims Web3 freebies — airdrops, faucets, free mints, quests, and incentives — across multiple EVM chains.

## Architecture

| Layer | Tech |
|-------|------|
| **Shell** | Tauri 2.0 (~3MB binary) |
| **Frontend** | React 19 + Tailwind 4 + Viem 2 + Zustand |
| **Backend** | Rust (Tokio, alloy, rusqlite, ring) |
| **Storage** | SQLite + AES-256-GCM + Argon2id |
| **Design** | Cyberpunk terminal aesthetic |

## Supported Chains (Phase 1)

Ethereum · Arbitrum · Optimism · Base · Polygon · zkSync Era

## Getting Started

```bash
# Prerequisites: Rust 1.76+, Node 18+, pnpm
pnpm install
cargo tauri dev
```

## Security

- Private keys **never** leave the Rust process
- All seed material encrypted at rest (AES-256-GCM + Argon2id KDF)
- Every transaction simulated via `eth_call` before execution
- Configurable gas ceilings and spending caps

## License

AGPL-3.0-or-later · Commercial license available