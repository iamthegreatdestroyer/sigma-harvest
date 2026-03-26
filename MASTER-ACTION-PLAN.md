# ΣHARVEST — Next Steps Master Action Plan

> Created: 2026-03-25
> Objective: Complete, fully functioning desktop application with state-of-the-art UI
> Starting point: 681 tests passing (202 FE + 479 Rust), 42 Rust + 34 JS/JSX files
> Target: v1.0.0 release-ready

---

## Plan Overview

```
Sprint 5  │ Close v1.0 Backend Gaps           │ Wire env vars, USD display, sweep logging
Sprint 6  │ Headless Browser Module            │ Chrome DevTools Protocol for JS-heavy claims
Sprint 7  │ State-of-the-Art UI Polish         │ Animations, toasts, skeletons, glassmorphism
Sprint 8  │ E2E Testing + Security Audit       │ Integration tests, cargo/pnpm audit, zeroize
Sprint 9  │ Auto-Updater + Performance         │ tauri-plugin-updater, startup profiling, soak test
Sprint 10 │ Extended Chains + Phase 2          │ New EVM chains, quest automation templates
```

---

## Sprint 5: Close Remaining v1.0 Gaps

**Goal:** Eliminate every remaining `[ ]` checkbox in REMAINING-WORK.md for Tiers 1-2.

### 5A — Wire Env Vars into Discovery Sources
**Files:** `discovery/dappradar.rs`, `discovery/social.rs`, `chain/provider.rs`
**What:**
- [ ] Read `DAPPRADAR_API_KEY` from env in DappRadar discovery constructor
- [ ] Read `TWITTER_BEARER_TOKEN` from env in Social discovery constructor
- [ ] Read `RPC_<CHAIN>_URL` env overrides into chain provider
- [ ] Fallback gracefully when env vars are absent

### 5B — USD Display in AnalyticsBay + WalletManager
**Files:** `src/views/AnalyticsBay.jsx`, `src/views/WalletManager.jsx`
**What:**
- [ ] Import `usePriceStore` and fetch prices on mount
- [ ] Display ETH balance × USD price alongside native amounts in WalletManager
- [ ] Add USD column to chain breakdown table in AnalyticsBay
- [ ] Format all monetary values with `$` prefix and 2 decimal places

### 5C — Sweep Transaction Logging
**Files:** `executor/consolidation.rs`, `db/mod.rs`
**What:**
- [ ] Log each sweep transaction into the `claims` table with status "Consolidation"
- [ ] Record gas_cost_usd and value_received_usd for sweep operations
- [ ] Add `consolidation_type` field handling in analytics queries

---

## Sprint 6: Headless Browser Module

**Goal:** Support claiming on platforms that require JavaScript execution.

### 6A — Chrome DevTools Protocol Integration
**Files:** `executor/browser.rs` (new)
**What:**
- [ ] `BrowserSession` struct wrapping `headless_chrome::Browser`
- [ ] `navigate_and_claim(url, wallet_provider)` method
- [ ] Cookie/session management
- [ ] MetaMask-like wallet provider injection via CDP
- [ ] Screenshot on error → save to app data dir
- [ ] CAPTCHA detection → flag claim for manual intervention
- [ ] Configurable timeout (default 30s)

### 6B — Claim Pipeline Integration
**Files:** `executor/mod.rs`, `ipc/commands.rs`
**What:**
- [ ] Add `BrowserClaim` variant to claim processing flow
- [ ] `execute_browser_claim` IPC command
- [ ] Route JS-heavy opportunities through browser path
- [ ] Tests with mock browser behavior

---

## Sprint 7: State-of-the-Art UI Polish

**Goal:** Transform from functional to stunning. Cyberpunk terminal aesthetic with modern UX.

### 7A — View Transitions + Micro-Animations (Framer Motion)
**Files:** `App.jsx`, all views
**What:**
- [ ] `AnimatePresence` wrapper around `ActiveView` in App.jsx
- [ ] Fade + slide transitions between views (150ms)
- [ ] Stagger-in animations for stat cards and table rows
- [ ] Pulse animation on live data updates (HarvestFeed, GasTicker)
- [ ] Scale-in animation for CommandPalette

### 7B — In-App Toast Notification System
**Files:** `components/Toast.jsx` (new), `lib/toast.js` (new), `stores/toastStore.js` (new)
**What:**
- [ ] Toast position: bottom-right, stacked
- [ ] Types: success (green), error (red), warning (amber), info (cyan)
- [ ] Auto-dismiss after 5s with progress bar
- [ ] Manual dismiss with X button
- [ ] Animate in from right, slide down on stack
- [ ] Wire into huntStore (claim results), walletStore (operations)

### 7C — Loading Skeletons
**Files:** `components/Skeleton.jsx` (new)
**What:**
- [ ] Shimmer animation skeleton component
- [ ] Apply to: Dashboard stat cards, WalletTree, AnalyticsBay charts
- [ ] Replace bare `Loading...` text with skeletons everywhere

### 7D — Enhanced Dashboard
**Files:** `views/Dashboard.jsx`, components
**What:**
- [ ] Animated counter for stat values (count-up on load)
- [ ] Pulsing green dot with "LIVE" label on HarvestFeed
- [ ] Mini progress ring showing overall claim success rate
- [ ] Gradient border glow on hovering stat cards

### 7E — Custom Window Title Bar
**Files:** `App.jsx`, `components/TitleBar.jsx` (new), `styles/globals.css`
**What:**
- [ ] Hide native title bar (Tauri decorations: false)
- [ ] Custom title bar with: ΣHARVEST logo, minimize/maximize/close buttons
- [ ] Draggable region for window movement
- [ ] System tray integration for background operation

### 7F — Enhanced AnalyticsBay Charts
**Files:** `views/AnalyticsBay.jsx`
**What:**
- [ ] Time-series line chart (cumulative profit over time)
- [ ] Interactive tooltips with formatted USD values
- [ ] Chain breakdown as horizontal bar chart with logos
- [ ] ROI percentage badge with trend arrow

### 7G — Glassmorphism + Visual Refinements
**Files:** `styles/globals.css`, all views
**What:**
- [ ] Subtle backdrop-blur on surface-raised elements
- [ ] Gradient mesh background on bg (animated, slow)
- [ ] Neon glow effects on primary/accent elements
- [ ] Refined scrollbar styling
- [ ] Selection highlight color matching theme
- [ ] Focus ring styling for accessibility

---

## Sprint 8: E2E Testing + Security Audit

**Goal:** Regression safety net + formal security review.

### 8A — Rust Integration Tests
**Files:** `tests/integration/` (new directory)
**What:**
- [ ] Full vault lifecycle test: create → derive → list → lock → unlock
- [ ] Discovery pipeline test with mock HTTP responses
- [ ] Claim execution pipeline test: create claim → simulate → process
- [ ] Analytics query tests with seeded data
- [ ] Property-based tests for crypto ops (`proptest` crate)

### 8B — Frontend E2E Tests
**Files:** `tests/e2e/` (new directory)
**What:**
- [ ] Tauri WebDriver test: vault create flow
- [ ] Hunt cycle simulation test
- [ ] Settings save/load round-trip test
- [ ] Navigation + keyboard shortcut tests

### 8C — Security Audit
**What:**
- [ ] `cargo audit` — check all Rust deps for CVEs
- [ ] `pnpm audit` — check all npm deps
- [ ] Verify: no private key material crosses IPC boundary
- [ ] Verify: all `zeroize` annotations on sensitive buffers
- [ ] Verify: no outbound telemetry/phone-home
- [ ] Verify: all user input validated at IPC boundary
- [ ] Document findings in `SECURITY-AUDIT.md`

---

## Sprint 9: Auto-Updater + Performance

**Goal:** Release-ready quality: auto-update, sub-2s startup, stable under load.

### 9A — Auto-Updater
**Files:** `src-tauri/Cargo.toml`, `tauri.conf.json`, `Settings.jsx`
**What:**
- [ ] Add `tauri-plugin-updater` to Cargo.toml
- [ ] Configure update endpoint: GitHub Releases
- [ ] "Check for Updates" button in Settings view
- [ ] Background update check on app launch (once per day)
- [ ] Update notification in header bar

### 9B — Performance Profiling
**What:**
- [ ] Startup time profiling (target: < 2s to first render)
- [ ] 24h soak test under continuous discovery (memory leak check)
- [ ] SQLite performance with 10K+ opportunity records
- [ ] RPC call efficiency audit (batch where possible)
- [ ] Frontend bundle code-splitting (dynamic imports for views)

---

## Sprint 10: Extended Chains + Phase 2

**Goal:** Expand beyond initial 6 chains, add quest automation.

### 10A — New EVM Chains
**Files:** `chain/registry.rs`, `src/lib/chains.js`
**What:**
- [ ] Avalanche C-Chain
- [ ] BNB Smart Chain
- [ ] Fantom / Sonic
- [ ] Linea
- [ ] Scroll
- [ ] Blast

### 10B — Quest Automation Templates
**Files:** `discovery/quests.rs` (new), `executor/quest_runner.rs` (new)
**What:**
- [ ] Galxe quest template engine
- [ ] Zealy task automation
- [ ] Layer3 quest support
- [ ] Template-based claim flow (multi-step)

---

## Success Metrics for v1.0.0

| Metric | Target |
|--------|--------|
| All tests pass | 800+ tests, 0 failures |
| Build | Clean (Vite + Cargo, 0 warnings) |
| Startup | < 2 seconds to first render |
| Memory | < 200MB under sustained operation |
| Security | 0 CVEs, 0 IPC key leaks |
| UI | 60fps animations, no layout shift |
| Coverage | 90%+ unit, all critical paths |

---

## Execution Order

**Immediate (this session):** Sprint 5A → 5B → 5C
**Next session:** Sprint 6
**Following sessions:** Sprint 7 → 8 → 9 → 10

Each sprint ends with: tests passing, commit, push.
