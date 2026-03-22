# ΣHARVEST — Autonomous Rollout Action Plan

> **Objective**: Maximum autonomy and automation at every stage.  
> **Source of Truth**: `w3hunter-blueprint.jsx` (10 sections, VIS-001 through SIGMA-010)  
> **Repo**: `iamthegreatdestroyer/sigma-harvest`  
> **Date**: 2026-03-21

---

## Legend

| Symbol  | Meaning                                                    |
| ------- | ---------------------------------------------------------- |
| **[A]** | Fully automatable by Copilot Agent — no human input needed |
| **[S]** | Semi-automated — agent generates, human reviews/approves   |
| **[H]** | Human-required — credentials, legal decisions, signing     |

---

## STAGE 0: Environment Bootstrap `[A/H]`

Everything before a single line of app code is written.

### 0.1 Toolchain Installation Verification `[A]`

```
□ Verify Rust toolchain (rustup, cargo, rustc ≥ 1.76)
□ Verify Node.js ≥ 20 + pnpm (preferred over npm for speed/disk)
□ Verify Tauri CLI 2.x (`cargo install tauri-cli --version "^2"`)
□ Verify system WebView2 (Windows — should be preinstalled)
□ Verify Git configuration and SSH key for github push
```

### 0.2 Secrets & Credentials Setup `[H]`

```
□ Create .env.local template for RPC endpoints (Alchemy/Infura free keys)
□ Create .env.local template for API keys (DappRadar, Twitter/X, etc.)
□ Document required free-tier signups in SETUP.md
□ NEVER commit .env files — .gitignore enforced from Step 0
```

### 0.3 Repository Initialization `[A]`

```
□ Initialize Git with .gitignore (Rust, Node, Tauri, env, SQLite DBs)
□ Create branch protection rules (main: require PR, develop: direct push)
□ Set up conventional commits (commitlint + husky)
□ Create AGPL-3.0 LICENSE file with commercial dual-license header
□ Push w3hunter-blueprint.jsx as historical artifact
```

**Estimated Effort**: 1 session (agent) + 30 min (human credentials)

---

## STAGE 1: Tauri 2.0 Project Scaffold `[A]`

Full project skeleton — compilable, launchable, zero functionality.

### 1.1 Tauri + React Scaffold `[A]`

```
□ `pnpm create tauri-app sigma-harvest --template react-ts`
    → Convert to JSX if preferred (blueprint uses .jsx)
□ Configure tauri.conf.json:
    - App name: "ΣHARVEST"
    - Bundle identifier: com.thegreatdestroyer.sigma-harvest
    - Window: 1400x900, dark, resizable, title "ΣHARVEST v0.1.0"
    - Permissions: fs (scoped), shell (restricted), http (fetch)
□ Verify `pnpm tauri dev` launches empty window
```

### 1.2 Frontend Dependencies `[A]`

```
□ pnpm add viem @tanstack/react-query zustand recharts lucide-react
□ pnpm add @radix-ui/react-dialog @radix-ui/react-tooltip @radix-ui/react-tabs
□ pnpm add framer-motion cmdk
□ pnpm add -D tailwindcss @tailwindcss/vite postcss autoprefixer
□ Configure Tailwind with cyberpunk palette:
    - primary: #00FF41 (phosphor green)
    - bg: #0A0E1A (deep navy)
    - surface: #0d1117
    - amber: #FFB800
    - danger: #FF0055
    - accent: #00D4FF
□ Create globals.css with JetBrains Mono / Fira Code font stack
```

### 1.3 Rust Backend Skeleton `[A]`

```
□ Set up Cargo.toml with ALL dependencies from TECH-007:
    tauri 2, tokio, reqwest, rusqlite, serde, serde_json,
    alloy, bip39, coins-bip32, ring, argon2, aes-gcm, zeroize,
    headless_chrome, scraper, feed-rs, cron, tracing, tracing-subscriber
□ Create module tree matching REPO-009:
    src-tauri/src/
    ├── vault/mod.rs (empty pub mod stubs)
    ├── discovery/mod.rs
    ├── evaluation/mod.rs
    ├── executor/mod.rs
    ├── scraper/mod.rs
    ├── analytics/mod.rs
    ├── db/mod.rs
    └── ipc/mod.rs
□ Wire up tracing subscriber in main.rs
□ Verify `cargo build` succeeds with all deps resolving
```

### 1.4 Frontend Shell `[A]`

```
□ Create App.jsx with sidebar + main content layout (matching UI-004)
□ Create 5 placeholder view components:
    Dashboard.jsx, HuntConsole.jsx, WalletManager.jsx,
    OpportunityInspector.jsx, AnalyticsBay.jsx
□ Create Zustand stores (appStore, walletStore, huntStore) with initial shape
□ Implement sidebar navigation with cyberpunk styling
□ Create useTauriCommand.js hook (invoke wrapper)
□ Verify full shell renders in `pnpm tauri dev`
```

### 1.5 CI/CD Pipeline `[A]`

```
□ Create .github/workflows/ci.yml:
    - Trigger: push to develop, PR to main
    - Jobs: cargo check, cargo clippy, cargo test
    - Jobs: pnpm lint, pnpm type-check
    - Matrix: windows-latest (primary), ubuntu-latest (validation)
□ Create .github/workflows/build.yml:
    - Trigger: push tag v*
    - Build Tauri bundle for Windows (.msi + .exe)
    - Upload artifacts
□ Create .github/dependabot.yml for Cargo + npm
```

### 1.6 Developer Experience Setup `[A]`

```
□ .editorconfig (2-space indent JS, 4-space Rust)
□ Prettier config (.prettierrc)
□ ESLint config (flat config, react + hooks)
□ clippy.toml (Rust lint configuration)
□ rust-analyzer settings in .vscode/settings.json
□ SKILL.md (GitHub Copilot workspace skill file)
□ Makefile or justfile with common commands:
    dev, build, test, lint, clean, db-migrate
```

**Estimated Effort**: 1-2 agent sessions, zero human input  
**Deliverable**: Fully compilable Tauri app with empty shell UI

---

## STAGE 2: Crypto Vault & Storage (Phase 1 Core) `[A]`

The security foundation — everything else depends on this.

### 2.1 Encrypted Database Layer `[A]`

```
□ Implement db/schema.rs:
    - wallets (id, path, public_address, chain, label, created_at)
    - opportunities (id, source, chain, type, harvest_score, status, ...)
    - claims (id, opportunity_id, wallet_id, tx_hash, gas_cost, status, ...)
    - config (key-value settings store)
    - scraper_state (source, last_run, cursor, ...)
□ Implement db/mod.rs:
    - Connection pool with rusqlite
    - Encryption at rest via SQLCipher or manual AES wrapping
    - Migration runner (embed SQL migrations at compile time)
□ Write initial migration SQL files
□ Unit tests for all CRUD operations
```

### 2.2 Vault Module `[A]`

```
□ vault/seed.rs:
    - BIP-39 mnemonic generation (12/24 word)
    - Mnemonic validation
    - Seed derivation from mnemonic + optional passphrase
□ vault/encryption.rs:
    - Argon2id KDF (passphrase → encryption key)
    - AES-256-GCM encrypt/decrypt for seed storage
    - Secure memory zeroing (zeroize crate on all sensitive buffers)
□ vault/derivation.rs:
    - BIP-44 HD derivation (m/44'/60'/0'/0/n for ETH)
    - Per-chain derivation paths
    - Derive N wallets from single seed
□ vault/keystore.rs:
    - Save encrypted seed to SQLite
    - Load + decrypt on unlock
    - Re-lock on idle timeout
    - Public address extraction (never expose private key to IPC)
□ Comprehensive tests:
    - Known test vectors for BIP-39/44
    - Encrypt → decrypt roundtrip
    - Derivation path correctness
    - Zeroize verification
```

### 2.3 IPC Command Layer (Vault) `[A]`

```
□ ipc/commands.rs — Tauri #[command] functions:
    - create_wallet(passphrase) → { mnemonic_preview, address_0 }
    - unlock_vault(passphrase) → { success, wallet_count }
    - lock_vault() → { success }
    - list_wallets() → Vec<WalletInfo> (public data only)
    - derive_next_wallet(chain) → { address, path }
    - get_vault_status() → { locked, wallet_count, last_unlock }
□ Wire into Tauri app builder plugin
□ Frontend hooks: useVaultStatus(), useWallets()
```

**Deliverable**: Secure wallet creation, encryption, derivation, lock/unlock — fully testable

---

## STAGE 3: Chain Connectivity & Wallet UI `[A]`

Connect to real blockchains. Make wallets visible.

### 3.1 Chain Client Module `[A]`

```
□ Create chain configuration registry (chains.rs):
    - Chain ID, name, RPC URLs (primary + fallback), block explorer
    - Supported: Ethereum, Arbitrum, Optimism, Base, Polygon, zkSync
□ Implement multi-provider with automatic failover
□ Balance checking (native ETH + ERC-20 via multicall batch)
□ Gas price fetching (EIP-1559 aware)
□ Rate limiting / backoff to stay within free RPC tiers
□ Unit tests with mock RPC responses
```

### 3.2 Wallet Manager UI `[A]`

```
□ WalletTree component — visual HD derivation tree
□ Per-wallet card: address, balances per chain, copy button, QR code
□ "Derive New Wallet" button → calls derive_next_wallet IPC
□ Consolidation target selector (designate cold wallet address)
□ Balance refresh with loading states
□ Zustand walletStore integration
```

### 3.3 Gas Ticker Component `[A]`

```
□ Real-time gas price display for all Phase 1 chains
□ Color-coded: green (low), amber (medium), red (high)
□ Configurable threshold lines
□ Auto-refresh every 15 seconds
```

**Deliverable**: Working wallet manager connected to live chains

---

## STAGE 4: Discovery Engine `[A/S]`

The autonomous intelligence layer.

### 4.1 Discovery Framework `[A]`

```
□ Define Opportunity data model (struct + DB schema):
    - source, chain, type, title, description, url
    - estimated_value_usd, gas_cost_estimate
    - deadline, contract_address, verified
    - harvest_score, risk_level
    - status (discovered, evaluating, qualified, claimed, expired, rejected)
□ Discovery trait / interface that all sources implement:
    async fn discover(&self) -> Vec<RawOpportunity>
□ Scheduler using cron crate — configurable intervals per source
□ Deduplication logic (URL + contract address + chain as composite key)
```

### 4.2 Source: RSS/API Scrapers `[A]`

```
□ discovery/rss.rs — airdrops.io RSS feed parser (feed-rs crate)
□ discovery/dappradar.rs — DappRadar airdrop API client
□ discovery/galxe.rs — Galxe GraphQL campaign query
□ Each returns Vec<RawOpportunity> in normalized format
□ Error handling: log + skip on individual source failure
□ Integration tests with saved fixture responses
```

### 4.3 Source: On-Chain Event Listener `[A]`

```
□ discovery/onchain.rs:
    - Subscribe to ERC-20 Transfer events on known airdrop contracts
    - Detect new contract deployments with "claim" function signatures
    - Filter by relevance (token value, contract verification)
□ Uses alloy's event subscription with auto-reconnect
□ Configurable contract watchlist (community-maintained JSON)
```

### 4.4 Source: Social Signals `[S]`

```
□ discovery/social.rs:
    - Twitter/X API v2 search for airdrop keywords
    - Sentiment/signal extraction (keyword matching, not full NLP)
    - Convert high-signal tweets → RawOpportunity
□ [H] Requires Twitter API bearer token setup
□ Rate limit awareness (free tier: 500k tweets/month reads)
```

### 4.5 Evaluation Engine `[A]`

```
□ evaluation/harvest_score.rs — Harvest Score v1:
    Weighted formula:
    - Gas efficiency: (estimated_value / gas_cost) × 25
    - Contract verified on Etherscan: +20
    - Project funding signals: +15
    - Community size: +10
    - Time urgency: +15 (exponential as deadline approaches)
    - Sybil risk penalty: -10 to -30
□ evaluation/risk.rs — Risk flags:
    - Unverified contract
    - Unlimited approve() calls
    - Known scam contract database match
    - Abnormally high estimated value (too good to be true)
□ Unit tests with fixture opportunities at various score levels
```

### 4.6 Discovery Feed UI `[A]`

```
□ HarvestFeed component — live scrolling opportunity feed
□ ScoreGauge component — visual harvest score indicator (0-100)
□ Filtering: by chain, by type, by score range, by status
□ Sorting: score, deadline, estimated value
□ Click to expand → OpportunityInspector view
□ Zustand huntStore integration for active hunt configuration
```

**Deliverable**: Autonomous discovery daemon populating a live feed

---

## STAGE 5: Claim Execution Engine `[A/S]`

The money machine. Where discovery becomes collection.

### 5.1 Transaction Builder `[A]`

```
□ executor/transaction.rs:
    - Build raw transactions from opportunity claim data
    - EIP-1559 gas parameter calculation
    - Nonce management (local nonce tracking + on-chain sync)
    - Transaction signing via vault (private key never leaves Rust)
□ executor/gas_oracle.rs:
    - Multi-source gas price aggregation
    - Configurable max gas ceiling per chain
    - Daily/weekly spending cap tracking
    - "Wait for low gas" queue mode
```

### 5.2 Dry-Run Simulation `[A]`

```
□ executor/simulation.rs:
    - eth_call simulation before every real tx
    - Decode revert reasons
    - Estimate actual gas usage
    - Flag suspicious outcomes (unexpected token approvals, etc.)
    - Gate: simulation must pass before execution proceeds
```

### 5.3 Claim Executor `[A]`

```
□ executor/mod.rs — Main execution pipeline:
    SIMULATE → CHECK GAS → CHECK CAPS → SIGN → BROADCAST → MONITOR → RECORD
□ executor/queue.rs:
    - Priority queue (by harvest score)
    - Retry logic with exponential backoff
    - Max retry count per opportunity
    - Status tracking: pending, simulating, executing, confirmed, failed
□ Template strategies:
    - SimpleClaim: Single contract call
    - MultiStep: Sequential transactions
    - BrowserClaim: Headless Chrome for JS-heavy pages
```

### 5.4 Headless Browser Module `[S]`

```
□ executor/browser.rs:
    - headless_chrome integration for claim pages needing JS
    - Cookie/session injection
    - Wallet connection simulation (inject provider)
    - Screenshot on error for debugging
    - CAPTCHA detection → flag for manual intervention
□ [H] Some claim flows may need human CAPTCHA solving
```

### 5.5 Hunt Console UI `[A]`

```
□ Start/Stop/Pause controls for entire pipeline
□ Per-source toggle switches
□ Real-time log stream (colored by severity)
□ Active claim queue display with status indicators
□ Manual opportunity URL submission field
□ Gas ceiling and spending cap configuration
```

**Deliverable**: End-to-end autonomous claim pipeline with safety guards

---

## STAGE 6: Auto-Consolidation & Analytics `[A]`

Collect → Consolidate → Measure.

### 6.1 Token Sweep / Consolidation `[A]`

```
□ Detect all ERC-20 balances across wallet constellation
□ Auto-sweep to designated cold wallet on schedule (cron)
□ Configurable: min sweep amount, gas threshold for sweep
□ Native token (ETH/MATIC/etc.) consolidation with dust handling
□ Sweep transaction logging in claims table
```

### 6.2 Analytics Engine `[A]`

```
□ analytics/reports.rs:
    - Total value collected (USD, per token)
    - Gas spent vs value received (ROI)
    - Chain-by-chain breakdown
    - Source attribution (which scrapers found best opportunities)
    - Time-series aggregation (hourly, daily, weekly, monthly)
□ Token price fetching via CoinGecko free API
□ Store historical snapshots for trend analysis
```

### 6.3 Analytics Bay UI `[A]`

```
□ Recharts time-series: collection value over time
□ ROI gauge: gas spent vs collected value
□ Source performance bar chart
□ Chain distribution pie chart
□ Top tokens leaderboard
□ Export to CSV/JSON for tax reporting
```

### 6.4 Dashboard (Command Center) `[A]`

```
□ Combine all widgets:
    - Live harvest feed (top 5 by score)
    - Wallet constellation summary (total balance)
    - Gas tickers (all chains)
    - 24h/7d/30d sparkline charts
    - Active claims count + success rate
□ Desktop notifications for high-score opportunities (Tauri notification API)
```

**Deliverable**: Full-cycle autonomous system with real-time visibility

---

## STAGE 7: Command Palette & Power-User Features `[A]`

### 7.1 Command Palette (Ctrl+K) `[A]`

```
□ cmdk integration with global keyboard shortcut
□ Commands: navigate views, start/stop hunts, derive wallet,
  check gas, sweep funds, toggle sources, open settings
□ Fuzzy search across opportunities and wallets
□ Recent commands history
```

### 7.2 Settings Panel `[A]`

```
□ RPC endpoint configuration per chain
□ Gas ceiling and spending cap settings
□ Auto-lock timeout configuration
□ Discovery source enable/disable + intervals
□ Notification preferences
□ Theme customization (accent colors)
□ Export/import all settings as JSON
```

### 7.3 Keyboard Shortcut System `[A]`

```
□ Navigate views: Alt+1 through Alt+5
□ Toggle hunt: Ctrl+H
□ Lock vault: Ctrl+L
□ Quick sweep: Ctrl+Shift+S
□ All shortcuts displayed in help overlay (?)
```

---

## STAGE 8: Hardening & Release Prep `[A/S]`

### 8.1 Testing Suite `[A]`

```
□ Rust unit tests: ≥90% coverage on vault, evaluation, executor
□ Rust integration tests: full pipeline with mock RPC
□ Frontend tests: React Testing Library for critical flows
□ E2E test: Tauri WebDriver test for wallet create → claim flow
□ Property-based tests (proptest crate) for crypto operations
□ Fuzz testing on input parsers (cargo-fuzz)
```

### 8.2 Security Audit `[S]`

```
□ cargo audit — check for known vulnerabilities
□ pnpm audit — frontend dependency check
□ Manual review: key management, IPC boundary, no key leaks to JS
□ Verify all sensitive memory uses zeroize
□ Verify SQLCipher or equivalent encryption at rest
□ Check: no telemetry, no analytics, no phone-home
```

### 8.3 Performance `[A]`

```
□ Profile startup time (target: <2s to render)
□ Profile scraper pipeline throughput
□ Memory usage under continuous operation (24h soak test)
□ Database performance with 10K+ opportunities
□ Concurrent chain queries optimization
```

### 8.4 Build & Distribution `[A]`

```
□ Tauri bundle: Windows .msi installer + portable .exe
□ Code signing (optional, [H] requires certificate)
□ Auto-update via Tauri's built-in updater (GitHub Releases)
□ README.md with installation, setup, and usage guide
□ CHANGELOG.md with semantic versioning
```

---

## STAGE 9: Extended Chain & Quest Automation (Phase 2-3) `[A]`

### 9.1 Extended EVM Chains `[A]`

```
□ Add chain configs: Avalanche, BNB, Fantom/Sonic, Linea, Scroll, Blast
□ RPC provider setup per chain
□ Validate discovery + claiming works per chain
□ Chain-specific gas oracle tuning
```

### 9.2 Quest Platform Automation `[S]`

```
□ Galxe quest completion templates
□ Zealy task automation (where possible without CAPTCHA)
□ Layer3/Intract campaign detection
□ Quest progress tracking per wallet
□ [H] Some quests require manual social actions
```

### 9.3 Non-EVM Preparation (Future) `[S]`

```
□ Solana SDK integration (solana-sdk Rust crate)
□ Solana wallet derivation (Ed25519)
□ SPL token balance checking
□ Solana-specific discovery sources
```

---

## STAGE 10: ΣLANG Integration (Innovation Layer) `[A]`

### 10.1 Opportunity HD Vector Encoding `[A]`

```
□ Encode opportunity attributes into hyperdimensional vectors
□ Use product quantizer for 192× compression
□ Build similarity index for pattern matching
□ "This looks like the Arbitrum airdrop pattern" detection
□ Anomaly scoring: unknown patterns = higher risk
```

### 10.2 Log Compression `[A]`

```
□ ΣLANG-compress historical logs (70%+ reduction target)
□ Semantic search over compressed logs
□ Archival pipeline: active logs → compressed archive on schedule
```

---

## Automation Maximization Strategy

### What Copilot Agent Can Do Fully Autonomously

| Category           | Scope                                       | Autonomy                      |
| ------------------ | ------------------------------------------- | ----------------------------- |
| **Scaffolding**    | All project setup, file structure, configs  | **100%**                      |
| **Rust modules**   | All 6 core modules + DB + IPC layer         | **100%**                      |
| **React UI**       | All 5 views + components + stores + hooks   | **100%**                      |
| **CI/CD**          | GitHub Actions, Dependabot, linting         | **100%**                      |
| **Testing**        | Unit, integration, property, fuzz harnesses | **100%**                      |
| **Documentation**  | README, SKILL.md, code comments, JSDoc      | **100%**                      |
| **Git Operations** | Commits, branches, PRs, releases            | **95%** (push needs approval) |

### What Requires Human Input

| Category              | What                                   | Why                               |
| --------------------- | -------------------------------------- | --------------------------------- |
| **API Keys**          | Alchemy, DappRadar, Twitter, CoinGecko | Signup on external services       |
| **Master Passphrase** | Vault encryption passphrase            | Security-critical personal choice |
| **Code Signing Cert** | Windows code signing (optional)        | Purchase/setup                    |
| **CAPTCHA Solving**   | Some claim pages use CAPTCHAs          | Can't automate responsibly        |
| **Legal Review**      | AGPL + commercial dual licensing       | Legal implications                |
| **Social Quests**     | Twitter follow/retweet type tasks      | Platform ToS                      |

### Recommended Agent Workflow Per Stage

```
For each stage:
1. Agent generates ALL code + tests + configs
2. Agent runs `cargo check` + `cargo test` + `pnpm build`
3. Agent commits to feature/* branch with conventional commit
4. Agent creates PR with description matching blueprint refs
5. Human: quick review → merge
6. Repeat for next stage
```

### Parallelization Opportunities

```
Can run simultaneously:
├── Rust vault module ─────┐
├── Frontend shell UI ─────┤── No dependency between these
├── CI/CD pipeline ────────┘
│
After vault + UI complete:
├── Chain connectivity ────┐
├── Discovery sources ─────┤── Independent of each other
├── Analytics engine ──────┘
│
After chain + discovery:
└── Claim executor (depends on both)
```

---

## Execution Order Summary

| Stage  | Name                           | Deps        | Autonomy |
| ------ | ------------------------------ | ----------- | -------- |
| **0**  | Environment Bootstrap          | None        | A/H      |
| **1**  | Tauri Scaffold + Shell         | Stage 0     | **A**    |
| **2**  | Crypto Vault + DB              | Stage 1     | **A**    |
| **3**  | Chain Connectivity + Wallet UI | Stage 2     | **A**    |
| **4**  | Discovery Engine + Feed UI     | Stage 1     | **A/S**  |
| **5**  | Claim Executor + Hunt Console  | Stage 2+3+4 | **A/S**  |
| **6**  | Consolidation + Analytics      | Stage 3+5   | **A**    |
| **7**  | Command Palette + Power UX     | Stage 1-6   | **A**    |
| **8**  | Testing + Security + Release   | Stage 1-7   | **A/S**  |
| **9**  | Extended Chains + Quests       | Stage 5     | **A/S**  |
| **10** | ΣLANG Integration              | Stage 4+6   | **A**    |

---

## Next Immediate Action

**Ready to execute Stage 0 + Stage 1 right now.**

Say the word and I will:

1. Verify your toolchain (Rust, Node, Tauri CLI)
2. Scaffold the full Tauri 2.0 + React project
3. Install all dependencies from the blueprint
4. Create the complete Rust module tree
5. Build the cyberpunk UI shell with all 5 views
6. Set up CI/CD, linting, and developer tooling
7. Commit everything and push to `iamthegreatdestroyer/sigma-harvest`

**All of this can be done in a single session with zero human input.**
