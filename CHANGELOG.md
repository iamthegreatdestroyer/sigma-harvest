# Changelog

All notable changes to ΣHARVEST will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-21

### Added
- Initial Tauri 2.0 project scaffold with React frontend
- Cyberpunk terminal UI shell with 5 views (Dashboard, Hunt Console, Wallet Manager, Opportunity Inspector, Analytics Bay)
- Command palette (Ctrl+K) with cmdk integration
- Rust backend module structure: vault, discovery, evaluation, executor, scraper, analytics, db, ipc
- Zustand state management (appStore, walletStore, huntStore)
- Tauri IPC hooks for frontend-backend communication
- SQLite database schema with migrations
- Multi-chain configuration for 6 EVM chains
- CI/CD pipeline (GitHub Actions)
- AGPL-3.0 dual-license
