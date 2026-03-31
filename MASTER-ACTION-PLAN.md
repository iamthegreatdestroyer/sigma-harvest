# ΣHARVEST — Next Steps Master Action Plan v3

> **Updated:** 2026-03-31
> **Objective:** Ship v1.0.0 — fully functioning, polished, audited desktop application
> **Current state:** 699 tests passing (202 FE + 497 Rust), 42 Rust + 44 JS/JSX files
> **Sprints completed:** 1-5 (delivered), Sprint 6 (90% — uncommitted)
> **Target:** v1.0.0 release-ready
> **Autonomy model:** Maximum automation — each sprint designed for uninterrupted agent execution

---

## Plan Overview

```
COMPLETED ────────────────────────────────────────────────────────────
Sprint 1  │ Transaction Simulation + Env Loading    │ DONE ✓  614979c
Sprint 2  │ Settings View + Shortcuts + Auto-Lock   │ DONE ✓  4985b52
Sprint 3  │ Consolidation Backend + Price Fetching   │ DONE ✓  5b826ee
Sprint 4  │ Notifications + Sparkline Charts         │ DONE ✓  5fd4110
Sprint 5  │ API Key Wiring + USD + Sweep Logging     │ DONE ✓  0749ae0

IN PROGRESS ─────────────────────────────────────────────────────────
Sprint 6  │ Commit + Final Backend Gaps              │ ~90% (code written, tests pass)

REMAINING ────────────────────────────────────────────────────────────
Sprint 7  │ Headless Browser Module                  │ 1-2 sessions
Sprint 8  │ State-of-the-Art UI Polish               │ 1-2 sessions
Sprint 9  │ E2E Testing + Security Audit             │ 1-2 sessions
Sprint 10 │ Auto-Updater + Performance + Release     │ 1 session
Sprint 11 │ Extended Chains + Phase 2                │ Post-v1.0
```

---

## Autonomy & Automation Principles

1. **Zero human input required** for all remaining sprints (code, tests, commits)
2. **Each sprint is self-contained** — read, implement, test, verify, commit
3. **Dependency chain is strict** — no sprint starts until predecessor tests pass
4. **Automated gates** — `cargo test`, `vitest run`, `cargo clippy` must all pass before commit
5. **Parallel execution** — independent tasks within a sprint run concurrently via subagents
6. **Rollback safety** — each sprint is a single commit; `git revert` undoes entire sprint
7. **Incremental value** — every sprint produces a usable improvement, never leaves the app broken

---

## Sprint 6: Commit Existing Work + Final Cleanup

**Goal:** Commit the 11 uncommitted files (simulation gate, env constructors, USD display, RPC overrides). Close any remaining gaps.

**Autonomy:** [A] Fully autonomous — code is written, tests pass.

### 6A — Verify & Commit Existing Work
**Status:** Code complete, 699 tests passing.
**Files (uncommitted):**
- [x] `src-tauri/src/executor/mod.rs` — simulation gate in ClaimPipeline
- [x] `src-tauri/src/executor/queue.rs` — queue updates
- [x] `src-tauri/src/discovery/dappradar.rs` — `from_env()` constructor
- [x] `src-tauri/src/discovery/social.rs` — `from_env()` constructor
- [x] `src-tauri/src/chain/provider.rs` — RPC override from env vars
- [x] `src-tauri/src/chain/coingecko.rs` — env var support
- [x] `src-tauri/src/analytics/mod.rs` — USD in analytics
- [x] `src/views/WalletManager.jsx` — portfolio USD total + per-wallet USD
- [x] `src/views/AnalyticsBay.jsx` — chain breakdown USD table
- [x] `src/lib/formatters.js` — `formatUsd`, `formatUsdLarge`
- [x] `MASTER-ACTION-PLAN.md` — this document

**Action:** Run tests → commit → update REMAINING-WORK.md checkboxes.

**Sprint 6 Exit Criteria:** All 699+ tests pass, clean commit on main.

---

## Sprint 7: Headless Browser Module

**Goal:** Enable autonomous claiming on platforms requiring JavaScript execution.

**Autonomy:** [A] Fully autonomous.

### 7A — BrowserSession Implementation
**Files:** `src-tauri/src/executor/browser.rs` (new)
**What:**
- [ ] `BrowserSession` struct wrapping `headless_chrome::Browser`
- [ ] `navigate_and_claim(url, claim_config)` — page interaction via CDP
- [ ] Cookie/session persistence between claims on same platform
- [ ] MetaMask-like wallet provider injection (inject `window.ethereum` via CDP)
- [ ] Screenshot on error → save to `{app_data_dir}/error-screenshots/`
- [ ] CAPTCHA detection (heuristic: detect common CAPTCHA iframe patterns) → flag `ManualIntervention`
- [ ] Configurable timeout (default 30s, max 120s)
- [ ] 10+ unit tests with mock browser behavior

### 7B — BrowserClaim Pipeline Integration
**Files:** `src-tauri/src/executor/mod.rs`, `src-tauri/src/ipc/commands.rs`
**What:**
- [ ] Route opportunities with `requires_browser: true` through browser path
- [ ] `execute_browser_claim` IPC command
- [ ] Opportunity scoring: `requires_browser` adds risk penalty (slower, less reliable)
- [ ] Tests: mock browser claim → verify pipeline routing

**Sprint 7 Exit Criteria:** Tests pass, browser module compiles, commit.

---

## Sprint 8: State-of-the-Art UI Polish

**Goal:** Transform from functional to stunning. Cyberpunk terminal aesthetic with modern UX.

**Autonomy:** [A] Fully autonomous.

### 8A — View Transitions + Micro-Animations
**Files:** `src/App.jsx`, all views
- [ ] `AnimatePresence` wrapper around active view in App.jsx
- [ ] Fade + slide transitions between views (150ms ease-out)
- [ ] Stagger-in animations for stat cards (50ms delay per item)
- [ ] Pulse animation on live data updates (HarvestFeed, GasTicker)
- [ ] Scale-in animation for CommandPalette dialog

### 8B — Toast Notification System
**Files:** `src/components/Toast.jsx`, `src/lib/toast.js`, `src/stores/toastStore.js` (all new)
- [ ] Toast container: bottom-right, stacked (max 5 visible)
- [ ] Types: success (green), error (red), warning (amber), info (cyan)
- [ ] Auto-dismiss after 5s with animated progress bar
- [ ] Manual dismiss with X button
- [ ] Framer Motion slide-in from right
- [ ] Wire into: huntStore (claim results), walletStore (derivation, consolidation)
- [ ] Tests: toastStore add/remove/auto-dismiss

### 8C — Loading Skeletons
**Files:** `src/components/Skeleton.jsx` (new)
- [ ] Shimmer animation skeleton component (configurable width/height/rounded)
- [ ] Apply to: Dashboard stat cards, WalletTree rows, AnalyticsBay chart areas
- [ ] Replace all bare `Loading...` text throughout app

### 8D — Enhanced Dashboard Widgets
**Files:** `src/views/Dashboard.jsx`
- [ ] Animated counter (count-up from 0 on load using requestAnimationFrame)
- [ ] Pulsing green dot + "LIVE" label on HarvestFeed header
- [ ] Mini progress ring (SVG) showing overall claim success rate
- [ ] Gradient border glow on stat card hover (CSS transition)

### 8E — Custom Window Title Bar
**Files:** `src/App.jsx`, `src/components/TitleBar.jsx` (new), `src-tauri/tauri.conf.json`
- [ ] Set `"decorations": false` in tauri.conf.json
- [ ] Custom title bar: ΣHARVEST logo + version, minimize/maximize/close buttons
- [ ] `data-tauri-drag-region` for window dragging
- [ ] Window control buttons using `@tauri-apps/api/window`

### 8F — Enhanced Charts + Glassmorphism
**Files:** `src/views/AnalyticsBay.jsx`, `src/styles/globals.css`
- [ ] Time-series line chart (cumulative profit over time) with interactive tooltips
- [ ] Chain breakdown horizontal bar chart
- [ ] ROI percentage badge with trend arrow (up green / down red)
- [ ] Backdrop-blur on elevated surfaces (`backdrop-filter: blur(12px)`)
- [ ] Subtle animated gradient mesh background
- [ ] Neon glow on primary/accent interactive elements
- [ ] Custom scrollbar styling matching theme
- [ ] Focus ring and selection highlight in accent color

**Sprint 8 Exit Criteria:** All tests pass, UI polished, 60fps animations, commit.

---

## Sprint 9: E2E Testing + Security Audit

**Goal:** Regression safety net + formal security review.

**Autonomy:** [A] Fully autonomous.

### 9A — Rust Integration Tests
**Files:** `src-tauri/tests/` (new directory)
- [ ] Full vault lifecycle: create → derive → list → lock → unlock → re-derive
- [ ] Discovery pipeline: mock HTTP → parse → evaluate → score
- [ ] Claim pipeline: create → simulate → process → record in DB
- [ ] Analytics queries: seed 100 records → verify aggregations
- [ ] Property-based tests (proptest): BIP-39 seeds, AES encrypt/decrypt roundtrip

### 9B — Frontend Component Tests
**Files:** `src/__tests__/` (expand existing)
- [ ] Dashboard render test with mocked store data
- [ ] Settings save/load round-trip test
- [ ] Toast lifecycle test (add → auto-dismiss → remove)
- [ ] CommandPalette search + action test
- [ ] Navigation keyboard shortcut tests

### 9C — Security Audit
- [ ] Run `cargo audit` — document all findings
- [ ] Run `pnpm audit` — document all findings
- [ ] Manual IPC boundary review: grep for private key patterns crossing IPC
- [ ] Verify all `zeroize` annotations on `Seed`, `EncryptionKey`, `PrivateKey` types
- [ ] Verify zero outbound telemetry/analytics/phone-home
- [ ] Verify all user input validated at IPC boundary (string lengths, address formats)
- [ ] Write `SECURITY-AUDIT.md` with findings and attestations

**Sprint 9 Exit Criteria:** 900+ tests pass, 0 audit CVEs (or documented mitigations), commit.

---

## Sprint 10: Auto-Updater + Performance + Release

**Goal:** Release-ready quality and v1.0.0 tag.

**Autonomy:** [A] Fully autonomous (except tag push authorization).

### 10A — Auto-Updater
- [ ] Add `tauri-plugin-updater` to Cargo.toml and capabilities
- [ ] Configure update endpoint: GitHub Releases API
- [ ] "Check for Updates" button in Settings view
- [ ] Background update check on app launch (once per 24h, stored in config table)
- [ ] Update available toast notification

### 10B — Performance Optimization
- [ ] Startup time profiling with `tracing` spans (target: <2s to first render)
- [ ] Frontend code-splitting: dynamic `import()` for each view
- [ ] SQLite query optimization: EXPLAIN ANALYZE on analytics queries
- [ ] RPC call batching: consolidate multiple balance checks into multicall
- [ ] Memory profiling: verify <200MB under sustained 24h operation

### 10C — Release Prep
- [ ] Update CHANGELOG.md with all sprint deliverables
- [ ] Update README.md with current feature list and screenshots
- [ ] Verify `pnpm tauri build` produces clean Windows installer
- [ ] Create GitHub Release draft with release notes
- [ ] Tag `v1.0.0`

**Sprint 10 Exit Criteria:** 1000+ tests, clean build, sub-2s startup, v1.0.0 tag ready.

---

## Sprint 11: Extended Chains + Phase 2 (Post v1.0)

**Goal:** Expand beyond initial 6 chains.

### 11A — New EVM Chains
- [ ] Avalanche C-Chain (43114)
- [ ] BNB Smart Chain (56)
- [ ] Fantom / Sonic (250)
- [ ] Linea (59144)
- [ ] Scroll (534352)
- [ ] Blast (81457)
- [ ] Update both `chain/registry.rs` and `src/lib/chains.js`
- [ ] Verify discovery + claiming works per chain

### 11B — Quest Automation Templates
- [ ] `discovery/quests.rs` — quest platform detection
- [ ] `executor/quest_runner.rs` — multi-step quest execution
- [ ] Galxe quest template engine
- [ ] Zealy task automation (non-CAPTCHA tasks)
- [ ] Layer3 quest support

---

## Success Metrics for v1.0.0

| Metric | Current | Target | Gap |
|--------|---------|--------|-----|
| Tests passing | 699 | 1000+ | +301 |
| Build warnings | 0 | 0 | MET |
| Clippy warnings | 0 | 0 | MET |
| Sprints complete | 5.9/10 | 10/10 | +4.1 |
| Startup time | unmeasured | <2s | TBD |
| Memory (sustained) | unmeasured | <200MB | TBD |
| Security CVEs | unaudited | 0 | Sprint 9 |
| UI animations | none | 60fps | Sprint 8 |

---

## Execution Strategy for Maximum Autonomy

### Per-Sprint Workflow (Automated)
```
1. Read all target files
2. Implement changes (parallel where independent)
3. Run cargo test + vitest run + cargo clippy
4. Fix any failures
5. Commit with conventional commit message
6. Update REMAINING-WORK.md checkboxes
7. Report completion summary
```

### Parallelization Map
```
Sprint 6:  Commit existing work (single atomic action)

Sprint 7:  7A (browser impl) ──────┐
           7B (pipeline route) ─────┘── 7B depends on 7A

Sprint 8:  8A (animations) ───┐
           8B (toasts) ───────┤
           8C (skeletons) ────┤── All independent, run parallel
           8D (dashboard) ────┤
           8E (titlebar) ─────┤
           8F (charts+glass) ─┘

Sprint 9:  9A (rust tests) ───┐
           9B (FE tests) ─────┤── All independent, run parallel
           9C (security) ─────┘

Sprint 10: 10A (updater) ─────┐
           10B (perf) ────────┤── Mostly independent
           10C (release) ─────┘── 10C depends on 10A+10B
```

### Human Touchpoints (Minimal)
| When | What | Why |
|------|------|-----|
| After Sprint 9C | Review SECURITY-AUDIT.md | Security-critical attestation |
| After Sprint 10C | Approve v1.0.0 tag push | Release authorization |
| Post v1.0 | Provide API keys in .env.local | External service credentials |

---

## Immediate Next Action

**Execute Sprint 6 NOW** — commit the existing 11 uncommitted files, verify all 699+ tests pass.
Then proceed to Sprint 7 (headless browser module).
