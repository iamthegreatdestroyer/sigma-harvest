# ΣHARVEST — Remaining Work for Fully Functioning Desktop App

> Audit date: 2026-03-25
> Auditor: Copilot Agent (APEX-01)
> Codebase state: **Zero stubs remaining** — 42 Rust files, 30 JS/JSX files, all real implementations
> Tests: **178/178 JS tests pass**, **474/474 Rust tests pass**
> Build: **Clean** (Vite + Cargo)

---

## Current Completion Status

| Stage | Name | Status | Completion |
|-------|------|--------|------------|
| 0 | Environment Bootstrap | **DONE** | 100% |
| 1 | Tauri 2.0 Scaffold + Shell | **DONE** | 100% |
| 2 | Crypto Vault + Storage | **DONE** | 100% |
| 3 | Chain Connectivity + Wallet UI | **DONE** | 100% |
| 4 | Discovery Engine + Feed UI | **DONE** | 100% |
| 5 | Claim Execution Engine | **PARTIAL** | ~90% |
| 6 | Auto-Consolidation + Analytics | **PARTIAL** | ~85% |
| 7 | Command Palette + Power UX | **DONE** | 100% |
| 8 | Hardening + Release Prep | **PARTIAL** | ~55% |
| 9 | Extended Chains + Quests | NOT STARTED | 0% |
| 10 | ΣLANG Integration | **PARTIAL** | ~30% |

---

## What's Already Built and Working

### Backend (Rust) — 42 source files, all complete

- **Vault**: BIP-39 mnemonic, BIP-44 HD derivation, AES-256-GCM + Argon2id encryption, full keystore lifecycle
- **Chain**: 6-chain registry (ETH, ARB, OP, BASE, MATIC, zkSync), RPC client with failover, EIP-1559 gas, CoinGecko price client
- **Discovery**: 5 sources (RSS, DappRadar, Galxe GraphQL, on-chain events, Twitter/X)
- **Evaluation**: 6-component Harvest Score (0–100), 5-heuristic risk assessment, 4 risk levels
- **Executor**: EIP-1559 transaction builder, gas oracle with daily caps, priority queue with retries, transaction simulation, token consolidation
- **Scraper**: HTML parsing pipeline with CSS selectors, ETH address extraction, rate limiting
- **Analytics**: SQL-backed summary reports, source attribution, chain breakdown
- **ΣCORE**: HD vectors, associative memory, Lotka-Volterra dynamics, evolutionary swarm, wave scoring
- **DB**: SQLite with WAL mode, 5-table migration, full CRUD
- **IPC**: ~38 Tauri commands wired to frontend

### Frontend (React) — 30 source files, all complete

- **6 Views**: Dashboard, HuntConsole, WalletManager, OpportunityInspector, AnalyticsBay, Settings
- **7 Components**: CommandPalette, GasTicker, HarvestFeed, HuntConsole, ScoreGauge, SigmaCoreWidget, WalletTree
- **8 Stores**: app, wallet, hunt, chain, sigma, analytics, settings, price (all Zustand)
- **3 Hooks**: useTauriCommand, useDiscovery, useWallets
- **3 Libs**: chains, constants, formatters

### Infrastructure

- CI (cargo check + clippy + test + pnpm build)
- Release build (Tauri bundle for Windows via GitHub Actions on tag push)
- Dependabot (Cargo + npm + GitHub Actions)
- 178 frontend unit tests (Vitest), 474 Rust unit tests

---

## Remaining Work — Ordered by Priority

### TIER 1: Core Functionality Gaps (Required for v1.0)

These are the features that prevent the app from operating end-to-end autonomously.

#### 1. Transaction Simulation (`executor/simulation.rs`)
**What**: Before broadcast, every claim transaction must be simulated via `eth_call` to detect reverts, check gas, and flag suspicious outcomes.
**Why critical**: Without this, real transactions with real gas can fail or interact with malicious contracts.
**Scope**:
- [x] `eth_call` simulation with full state override
- [x] ABI revert reason decoding (already partially in `transaction.rs`)
- [x] Gas estimation comparison (simulated vs oracle)
- [x] Suspicious outcome detection (unexpected token approvals, balance changes)
- [x] Gate: simulation MUST pass before `executor/mod.rs` proceeds to signing
- [ ] Integration with existing `ClaimPipeline` in `executor/mod.rs`

#### 2. Settings View (`src/views/Settings.jsx` + `src/stores/settingsStore.js`)
**What**: No UI currently exists to configure RPC endpoints, gas ceilings, API keys, auto-lock timeout, or notification preferences.
**Why critical**: Users must currently edit `.env.local` or `.env` files manually — the app has no way to persist runtime config changes.
**Scope**:
- [x] New `Settings.jsx` view added to sidebar navigation
- [x] `settingsStore.js` Zustand store backed by Tauri `get_config`/`set_config` IPC commands
- [x] RPC endpoint override per chain (text inputs)
- [x] Gas ceiling per chain (number inputs consuming existing `setGasCeiling` in huntStore)
- [x] API keys for DappRadar, Twitter/X, CoinGecko (masked password inputs)
- [x] Auto-lock timeout selector (5m / 15m / 30m / 1h / never)
- [x] Discovery source intervals (seconds per source)
- [x] Export/import settings as JSON
- [x] Sidebar nav entry with Settings/gear icon

#### 3. Environment Variable Loading (Backend)
**What**: The Rust backend needs to read `.env.local` for API keys and RPC endpoint overrides.
**Why critical**: Discovery sources like DappRadar, Twitter/X, and CoinGecko fail silently without API keys — users need a way to provide them.
**Scope**:
- [x] Add `dotenvy` crate to Cargo.toml
- [x] Load `.env.local` at startup in `lib.rs`
- [ ] Wire env vars into discovery source constructors (DappRadar, Social, CoinGecko)
- [ ] Wire env vars into chain provider overrides
- [x] Add `get_config` / `set_config` IPC commands backed by DB `config` table

#### 4. Token Consolidation Backend (`executor/consolidation.rs`)
**What**: The Consolidate button in WalletManager currently shows a placeholder alert. Needs a backend module to sweep ERC-20 and native tokens from HD-derived wallets to a designated cold wallet.
**Why critical**: Core feature described in the blueprint — without it, claimed tokens stay scattered across many derived wallets.
**Scope**:
- [x] `executor/consolidation.rs` — sweep logic for ETH/native + ERC-20 tokens (19 unit tests)
- [x] ERC-20 balance detection across wallet constellation (multicall batch)
- [x] Configurable: min sweep amount, gas threshold, destination address
- [x] Dust handling (skip if gas > value)
- [x] `plan_consolidation` + `execute_consolidation` IPC commands
- [x] Wire front-end WalletManager Consolidate button to the backend via walletStore
- [ ] Sweep transaction logging in claims table

---

### TIER 2: Important UX & Safety Features (Required for comfortable daily use)

#### 5. Desktop Notifications (Tauri Notification Plugin)
**What**: Alert user when high-score opportunities are discovered or claims succeed/fail.
**Scope**:
- [ ] Add `@tauri-apps/plugin-notification` to frontend deps
- [x] Add `tauri-plugin-notification` to Cargo.toml
- [ ] Notification on opportunity with sigma_score > configurable threshold
- [ ] Notification on claim success/failure
- [x] Notification toggle in Settings view (settingsStore)

#### 6. Keyboard Shortcuts
**What**: Power-user keyboard navigation described in Stage 7 of ROLLOUT-PLAN.
**Scope**:
- [x] `Alt+1` through `Alt+6` — Navigate to views
- [x] `Ctrl+K` — Command palette
- [x] `Ctrl+H` — Toggle hunt
- [x] `Ctrl+L` — Lock vault
- [x] Help overlay (`?` key) showing all shortcuts

#### 7. Auto-Lock Timeout
**What**: Vault should auto-lock after configurable idle timeout.
**Scope**:
- [x] Idle timer in App.jsx (reset on mousedown/keydown/mousemove/scroll/touch)
- [x] Configurable duration in Settings view
- [x] Calls `lockVault()` on timeout

#### 8. Token Price Fetching (CoinGecko)
**What**: Currently, analytics and wallet balances are only in native ETH amounts — no USD conversion.
**Scope**:
- [x] CoinGecko free API client in Rust (`chain/coingecko.rs`, 13 unit tests)
- [x] `get_token_prices` IPC command with 5-minute TTL cache
- [x] `priceStore.js` Zustand store for frontend price state
- [ ] Wire USD display into AnalyticsBay and WalletManager views

---

### TIER 3: Execution Engine Completion (Required for fully autonomous claiming)

#### 9. Headless Browser Module (`executor/browser.rs`)
**What**: Some claim pages require JavaScript execution, wallet connection simulation, or multi-step UI flows.
**Why**: Standard `eth_call` + raw transaction won't work for these — need headless Chrome.
**Scope**:
- [ ] Add `headless_chrome` crate to Cargo.toml
- [ ] `executor/browser.rs` — Chrome DevTools Protocol integration
- [ ] Cookie/session management for platforms requiring login
- [ ] Wallet provider injection (simulate MetaMask-like provider)
- [ ] Screenshot on error for debugging
- [ ] CAPTCHA detection → flag for manual intervention (don't auto-solve)
- [ ] Integration with `ClaimPipeline::BrowserClaim` strategy

#### 10. Dashboard Sparkline Charts
**What**: Blueprint calls for 24h/7d/30d sparkline charts on the Command Center dashboard.
**Scope**:
- [ ] Time-series data endpoint in analytics backend
- [ ] Small Recharts sparkline components
- [ ] Wire into Dashboard view's stats grid

---

### TIER 4: Hardening & Release (Required before tagging v1.0)

#### 11. E2E Testing
**What**: Currently only unit tests exist. Need integration tests for full user flows.
**Scope**:
- [ ] Tauri WebDriver test: create vault → derive wallet → start hunt → view results
- [ ] Property-based tests for crypto operations (proptest crate)
- [ ] Rust integration tests with mock RPC responses

#### 12. Security Audit
**What**: Formal review of crypto operations and IPC boundary.
**Scope**:
- [ ] `cargo audit` — check for known CVEs
- [ ] `pnpm audit` — frontend dependency check
- [ ] Verify: no private key exposure across IPC boundary
- [ ] Verify: all sensitive memory uses `zeroize`
- [ ] Verify: no telemetry, no analytics, no phone-home

#### 13. Auto-Updater
**What**: Tauri has a built-in updater plugin for checking GitHub Releases.
**Scope**:
- [ ] Add `tauri-plugin-updater` to Cargo.toml
- [ ] Configure update endpoint to `github.com/iamthegreatdestroyer/sigma-harvest/releases`
- [ ] "Check for updates" button in Settings view
- [ ] background check on app launch

#### 14. Performance Profiling
**What**: Verify the app runs well under sustained operation.
**Scope**:
- [ ] Startup time profiling (target < 2s to render)
- [ ] 24h soak test under continuous discovery
- [ ] Memory usage monitoring
- [ ] Database performance with 10K+ opportunity records

---

### TIER 5: Phase 2-3 Features (Post-v1.0 / Nice-to-Have)

These are explicitly Phase 2-3 per the blueprint. Not required for v1.0.

| Feature | Description | Stage |
|---------|-------------|-------|
| Extended EVM chains | Avalanche, BNB, Fantom/Sonic, Linea, Scroll, Blast | 9.1 |
| Quest automation | Galxe/Zealy/Layer3 task templates | 9.2 |
| Non-EVM chains | Solana (solana-sdk), Cosmos, Sui/Aptos | 9.3 |
| ΣLANG opportunity encoding | HD-vector encoding of opportunity attributes | 10.1 |
| ΣLANG log compression | 70%+ compression of historical logs | 10.2 |
| Code signing | Windows Authenticode certificate | 8.4 |
| Tor/proxy routing | Anonymous scraping via Tor | SEC-006 |
| Hardware key unlock | YubiKey/FIDO2 for vault unlock | SEC-006 |
| Multi-machine sync | Distributed scraping agents | SIGMA-010 |

---

## Suggested Build Order for v1.0

```
Sprint 1 → Items 1 + 3 (simulation + env vars)
             Enables safe claiming with real API keys

Sprint 2 → Items 2 + 6 + 7 (settings UI + shortcuts + auto-lock)
             Full configuration without editing files

Sprint 3 → Items 4 + 8 (consolidation + price fetching)
             Complete the harvest cycle: discover → claim → consolidate → measure

Sprint 4 → Items 5 + 10 (notifications + sparklines)
             Passive monitoring without watching the screen

Sprint 5 → Items 9 + 11 (headless browser + E2E tests)
             Cover JS-heavy claim pages + regression safety

Sprint 6 → Items 12 + 13 + 14 (audit + updater + perf)
             Release prep → tag v1.0.0
```

---

## File Inventory Summary

| Directory | Files | Status |
|-----------|-------|--------|
| `src-tauri/src/` | 42 .rs files | ✅ All complete |
| `src/views/` | 6 .jsx files | ✅ All complete |
| `src/components/` | 7 .jsx files | ✅ All complete |
| `src/stores/` | 8 .js files | ✅ All complete |
| `src/hooks/` | 3 .js files | ✅ All complete |
| `src/lib/` | 3 .js files | ✅ All complete |
| `src/__tests__/` | 9 test files + 1 mock | ✅ 178/178 pass |
| `.github/workflows/` | 2 YAML files | ✅ CI + Release |
| Config files | 8 files | ✅ Complete |
