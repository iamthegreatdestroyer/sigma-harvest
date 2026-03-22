import { useState } from "react";
 
const sections = [
  {
    id: "VIS-001",
    title: "Project Vision & Codename",
    icon: "🎯",
    content: `
**Codename: ΣHARVEST (Sigma Harvest)**
*Sub-Linear Autonomous Resource Vacuum for Ethereum & Solana Tokens*

This isn't just a scraper — it's an **AI-augmented autonomous agent system** that continuously monitors the entire Web3 freebie landscape, evaluates opportunities using heuristic scoring, and executes collection operations across multiple chains and wallets simultaneously.

**Core Innovation**: Where existing tools (like DappRadar's airdrop dashboard or airdrops.io) require manual discovery and claim execution, ΣHARVEST operates as a **persistent daemon** with an intelligent pipeline:

\`\`\`
DISCOVER → EVALUATE → QUALIFY → CLAIM → STORE → REPORT
\`\`\`

Each stage is its own autonomous module with independent retry logic, risk scoring, and configurable thresholds.

**What Makes This Different**:
• **Hyperdimensional Opportunity Vectors** — Encodes airdrop characteristics (chain, gas cost, task complexity, estimated value, deadline) into HD vectors for ultra-fast similarity matching and prioritization
• **Phantom Wallet Array** — Generates and manages burner wallets programmatically with deterministic HD derivation from a single master seed
• **Gas Oracle Integration** — Only executes claims when gas is below configurable thresholds
• **Anti-Sybil Awareness** — Understands which airdrops have Sybil detection and routes accordingly
• **Zero-Click Collection** — For standard airdrops that only require wallet ownership, claims happen fully autonomously
`
  },
  {
    id: "ARC-002",
    title: "System Architecture",
    icon: "🏗️",
    content: `
**Stack Decision: Tauri 2.0 + React + Rust Backend**

Tauri is the perfect fit here — tiny binary (~3MB vs Electron's 100MB+), native OS webview, and critically: **Rust backend for cryptographic operations**. Private keys never touch the JS layer.

\`\`\`
┌─────────────────────────────────────────────────┐
│                  TAURI SHELL                     │
│  ┌───────────────────────────────────────────┐  │
│  │           REACT FRONTEND (UI)             │  │
│  │  ┌─────────┐ ┌──────────┐ ┌───────────┐  │  │
│  │  │Dashboard│ │ Wallet   │ │  Hunt      │  │  │
│  │  │  View   │ │ Manager  │ │  Console   │  │  │
│  │  └────┬────┘ └────┬─────┘ └─────┬─────┘  │  │
│  │       └───────────┼─────────────┘         │  │
│  │              Tauri IPC Bridge              │  │
│  └──────────────────┬────────────────────────┘  │
│  ┌──────────────────┴────────────────────────┐  │
│  │           RUST BACKEND (Core)             │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐  │  │
│  │  │ Crypto   │ │ Chain    │ │ Agent    │  │  │
│  │  │ Vault    │ │ Clients  │ │ Engine   │  │  │
│  │  └──────────┘ └──────────┘ └──────────┘  │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐  │  │
│  │  │ Scraper  │ │ Task     │ │ Gas      │  │  │
│  │  │ Pipeline │ │ Executor │ │ Oracle   │  │  │
│  │  └──────────┘ └──────────┘ └──────────┘  │  │
│  └───────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────┐  │
│  │         ENCRYPTED LOCAL STORAGE           │  │
│  │   SQLite + AES-256-GCM + Argon2id KDF    │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
\`\`\`

**Key Architectural Decisions**:
• **Viem over Ethers.js** — Tree-shakable, TypeScript-first, smaller bundle, faster encoding/parsing. Wagmi for React hooks layer.
• **Rust for all crypto ops** — HD wallet derivation (BIP-39/44), transaction signing, key storage via \`ring\` crate. JS never sees raw keys.
• **SQLite via \`rusqlite\`** — All persistent state (wallets, discovered opportunities, claim history, config) encrypted at rest.
• **Tokio async runtime** — Non-blocking concurrent scraping across multiple sources simultaneously.
`
  },
  {
    id: "MOD-003",
    title: "Module Breakdown",
    icon: "🧩",
    content: `
**6 Core Modules + 2 Support Modules**

**1. Discovery Engine** (\`src-tauri/src/discovery/\`)
Continuously scrapes and monitors:
• **API Sources**: DappRadar API, Galxe GraphQL, airdrops.io RSS, DeBank API
• **On-Chain Events**: ERC-20 Transfer events to known airdrop contracts, new token deployments with "claim" functions
• **Social Signals**: Twitter/X API for airdrop announcements (keyword matching + sentiment)
• **Faucet Registry**: Maintains list of active testnet/mainnet faucets with cooldown timers
• **Quest Platforms**: Galxe, Zealy, Layer3, Intract campaign monitoring

**2. Evaluation Engine** (\`src-tauri/src/evaluation/\`)
Every discovered opportunity gets a **Harvest Score (0-100)**:
• Gas cost vs estimated value ratio
• Contract verification status (verified on Etherscan = +20)
• Project funding/backing signals
• Community size indicators
• Sybil detection risk level
• Time-to-claim urgency

**3. Wallet Constellation** (\`src-tauri/src/vault/\`)
• Single master seed → deterministic HD wallet tree
• Auto-generates wallet "personas" for different chains
• Balance aggregation view across all wallets
• Auto-consolidation: sweeps collected tokens to a designated cold wallet on schedule

**4. Claim Executor** (\`src-tauri/src/executor/\`)
• Template-based claim strategies (simple transfer, contract interaction, multi-step quest)
• Gas price monitoring with configurable ceilings
• Transaction queue with retry logic and nonce management
• Dry-run simulation before real execution

**5. Scraper Pipeline** (\`src-tauri/src/scraper/\`)
• Headless browser integration (via \`headless_chrome\` Rust crate) for JS-heavy claim pages
• Anti-bot detection awareness (randomized delays, realistic user-agent rotation)
• Cookie/session management for platforms requiring login
• CAPTCHA detection (flags for manual intervention rather than solving)

**6. Reporting & Analytics** (\`src-tauri/src/analytics/\`)
• Total value collected over time
• Gas spent vs value received ROI tracking
• Chain-by-chain breakdown
• Best-performing discovery sources
`
  },
  {
    id: "UI-004",
    title: "UI Design Concept",
    icon: "🎨",
    content: `
**Aesthetic: "Cyberpunk Terminal Meets Bloomberg Terminal"**

Dark theme with phosphor green (#00FF41) primary accent, deep navy (#0A0E1A) background, and amber (#FFB800) for warnings/alerts. Monospace data displays mixed with clean sans-serif navigation.

**5 Primary Views**:

**① Command Center (Dashboard)**
• Live feed of discovered opportunities scrolling in real-time
• Wallet constellation health indicators (balances, pending claims)
• Gas price ticker across supported chains
• Harvest Score leaderboard of current best opportunities
• 24h/7d/30d collection summary with sparkline charts

**② Hunt Console**
• Active hunt configurations (which chains, min harvest score, gas ceiling)
• Start/stop/pause controls for each discovery source
• Real-time log stream showing agent decisions
• Manual opportunity submission field

**③ Wallet Manager**
• Visual tree of HD-derived wallets
• Per-wallet token inventory with USD estimates
• One-click consolidation to cold wallet
• Import/export wallet groups
• QR code display for any wallet address

**④ Opportunity Inspector**
• Detailed view of any discovered opportunity
• Contract source verification display
• Risk assessment breakdown
• One-click manual claim trigger
• Similar past opportunities comparison

**⑤ Analytics Bay**
• Time-series charts (Recharts) of collection value
• Gas efficiency metrics
• Source attribution (which scrapers found the best stuff)
• Token price tracking for collected assets
`
  },
  {
    id: "CHAIN-005",
    title: "Supported Chains & Protocols",
    icon: "⛓️",
    content: `
**Phase 1 — EVM Chains (Launch)**
• Ethereum Mainnet — Primary target, richest airdrop ecosystem
• Arbitrum — L2 with frequent retroactive airdrops
• Optimism — OP Stack ecosystem rewards
• Base — Coinbase L2, growing airdrop scene
• Polygon — Low gas, many faucet/airdrop programs
• zkSync Era — ZK rollup with anticipated token distributions

**Phase 2 — Extended EVM**
• Avalanche C-Chain
• BNB Chain
• Fantom / Sonic
• Linea, Scroll, Blast

**Phase 3 — Non-EVM (Future)**
• Solana (via \`solana-sdk\` Rust crate — native!)
• Cosmos ecosystem (via IBC)
• Sui / Aptos (Move-based chains)

**RPC Strategy**:
• Free tier RPC endpoints as default (Alchemy, Infura, QuickNode free plans)
• Configurable custom RPC URLs per chain
• Automatic failover between providers
• Rate limiting awareness to stay within free tiers

**Opportunity Types Supported**:
• Standard airdrops (wallet ownership based)
• Retroactive airdrops (based on historical on-chain activity)
• Faucet claims (testnet and mainnet micro-distributions)
• NFT mints (free mint detection and auto-execution)
• Quest completions (Galxe/Zealy/Layer3 task-based rewards)
• Liquidity mining signup bonuses
• Bridge incentive programs
`
  },
  {
    id: "SEC-006",
    title: "Security Architecture",
    icon: "🔐",
    content: `
**Non-Negotiable Security Principles**

Since this manages real crypto wallets, security is paramount even for personal use.

**Key Management**:
• Master seed encrypted with AES-256-GCM
• Key Derivation: Argon2id (memory-hard) from user-chosen passphrase
• Seed NEVER leaves Rust process — all signing happens in Rust
• JS frontend only ever sees public addresses
• Auto-lock after configurable idle timeout
• Optional hardware key (YubiKey) for master unlock via FIDO2

**Transaction Safety**:
• All transactions simulated via \`eth_call\` before execution
• Configurable max gas spend per transaction
• Daily/weekly spending caps
• Allowlist of known-safe contract addresses (community maintained)
• Flagging of approval transactions (\`approve()\`) with unlimited allowances

**Network Security**:
• All RPC calls over HTTPS/WSS
• No telemetry, no analytics, no phone-home
• Optional Tor/proxy routing for scraping
• DNS-over-HTTPS for name resolution

**Data at Rest**:
• SQLite database encrypted via SQLCipher
• Config files encrypted
• Log files auto-rotate and can be encrypted
• Secure memory wiping for sensitive data (\`zeroize\` crate)
`
  },
  {
    id: "TECH-007",
    title: "Technical Stack Details",
    icon: "🛠️",
    content: `
**Complete Dependency Map**

**Rust Backend (Cargo.toml)**:
\`\`\`toml
[dependencies]
tauri = { version = "2", features = ["shell-open"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ethers = "2"               # Rust-native Ethereum
alloy = "0.1"              # Next-gen Rust Ethereum
bip39 = "2"                # Mnemonic generation
coins-bip32 = "0.8"        # HD key derivation  
ring = "0.17"              # Cryptographic primitives
argon2 = "0.5"             # Password hashing
aes-gcm = "0.10"           # Symmetric encryption
zeroize = "1"              # Secure memory clearing
headless_chrome = "1"      # Browser automation
scraper = "0.19"           # HTML parsing
feed-rs = "1"              # RSS/Atom feed parsing
cron = "0.12"              # Scheduled tasks
tracing = "0.1"            # Structured logging
\`\`\`

**React Frontend (package.json)**:
\`\`\`json
{
  "dependencies": {
    "@tauri-apps/api": "^2",
    "react": "^19",
    "viem": "^2",
    "@tanstack/react-query": "^5",
    "recharts": "^2",
    "zustand": "^4",
    "lucide-react": "latest",
    "@radix-ui/react-*": "latest",
    "tailwindcss": "^4",
    "framer-motion": "^11",
    "cmdk": "^1"
  }
}
\`\`\`

**Why These Choices**:
• \`alloy\` — The Rust equivalent of viem, built by the same paradigm. Future of Rust-Ethereum.
• \`zustand\` — Minimal state management, perfect for Tauri IPC state sync.
• \`cmdk\` — Command palette (Ctrl+K) for power-user keyboard-driven operation.
• \`headless_chrome\` — Rust-native Chrome DevTools Protocol for claim pages needing JS execution.
`
  },
  {
    id: "ROAD-008",
    title: "Development Roadmap",
    icon: "🗺️",
    content: `
**5-Phase Build Plan**

**Phase 1: Foundation (Weeks 1-3)**
• Tauri 2.0 project scaffold with React + Tailwind
• Rust crypto vault (seed generation, HD derivation, AES encryption)
• SQLite schema design and migration system
• Basic dashboard shell with Tauri IPC communication working
• Single-chain (Ethereum) wallet creation and balance checking

**Phase 2: Discovery (Weeks 4-6)**
• RSS/API scraper modules for DappRadar, airdrops.io
• On-chain event listener for ERC-20 transfers to claim contracts
• Opportunity data model and Harvest Score algorithm v1
• Discovery feed in UI with filtering/sorting
• Galxe GraphQL integration

**Phase 3: Execution (Weeks 7-9)**
• Transaction builder and gas oracle integration
• Claim executor with dry-run simulation
• Headless browser module for JS-heavy claim pages
• Transaction queue with retry logic
• Multi-chain support (add Arbitrum, Optimism, Base, Polygon)

**Phase 4: Intelligence (Weeks 10-12)**
• Harvest Score v2 with ML-based valuation signals
• Pattern recognition for emerging airdrop campaigns
• Auto-consolidation sweep logic
• Advanced analytics and ROI tracking
• Command palette (Ctrl+K) and keyboard shortcuts

**Phase 5: Polish & Expand (Weeks 13-16)**
• zkSync, Linea, Scroll, Blast chain support
• Quest automation templates (Galxe/Zealy/Layer3)
• Notification system (desktop notifications for high-score opportunities)
• Settings/preferences panel
• Export functionality (CSV/JSON for tax reporting)
• Performance optimization and stress testing
`
  },
  {
    id: "REPO-009",
    title: "Repository Structure",
    icon: "📁",
    content: `
**Proposed repo: \`iamthegreatdestroyer/sigma-harvest\`**

\`\`\`
sigma-harvest/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── vault/
│   │   │   ├── mod.rs
│   │   │   ├── seed.rs          # BIP-39 mnemonic
│   │   │   ├── derivation.rs    # HD wallet tree
│   │   │   ├── encryption.rs    # AES-256-GCM + Argon2id
│   │   │   └── keystore.rs      # Encrypted storage
│   │   ├── discovery/
│   │   │   ├── mod.rs
│   │   │   ├── dappradar.rs
│   │   │   ├── galxe.rs
│   │   │   ├── onchain.rs       # Event listeners
│   │   │   ├── rss.rs           # Feed scraping
│   │   │   └── social.rs        # Twitter/X signals
│   │   ├── evaluation/
│   │   │   ├── mod.rs
│   │   │   ├── harvest_score.rs
│   │   │   └── risk.rs
│   │   ├── executor/
│   │   │   ├── mod.rs
│   │   │   ├── transaction.rs
│   │   │   ├── gas_oracle.rs
│   │   │   ├── queue.rs
│   │   │   └── browser.rs       # headless_chrome
│   │   ├── scraper/
│   │   │   ├── mod.rs
│   │   │   ├── pipeline.rs
│   │   │   └── parsers.rs
│   │   ├── analytics/
│   │   │   ├── mod.rs
│   │   │   └── reports.rs
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs
│   │   │   └── migrations/
│   │   └── ipc/
│   │       ├── mod.rs
│   │       └── commands.rs      # Tauri command handlers
│   └── icons/
├── src/
│   ├── App.jsx
│   ├── main.jsx
│   ├── stores/
│   │   ├── appStore.js
│   │   ├── walletStore.js
│   │   └── huntStore.js
│   ├── views/
│   │   ├── Dashboard.jsx
│   │   ├── HuntConsole.jsx
│   │   ├── WalletManager.jsx
│   │   ├── OpportunityInspector.jsx
│   │   └── AnalyticsBay.jsx
│   ├── components/
│   │   ├── CommandPalette.jsx
│   │   ├── GasTicker.jsx
│   │   ├── HarvestFeed.jsx
│   │   ├── WalletTree.jsx
│   │   └── ScoreGauge.jsx
│   ├── hooks/
│   │   ├── useTauriCommand.js
│   │   ├── useDiscovery.js
│   │   └── useWallets.js
│   ├── lib/
│   │   ├── chains.js
│   │   ├── formatters.js
│   │   └── constants.js
│   └── styles/
│       └── globals.css
├── SKILL.md                      # Copilot skill file
├── LICENSE                       # AGPL-3.0 + commercial
├── README.md
└── package.json
\`\`\`

This maps cleanly to your existing portfolio structure and dual-licensing model. The SKILL.md will follow your established 17+ project pattern for GitHub Copilot integration.
`
  },
  {
    id: "SIGMA-010",
    title: "ΣLANG Integration Potential",
    icon: "Σ",
    content: `
**Where ΣHARVEST Meets ΣLANG**

This is the sub-linear creativity angle. Your ΣLANG compression system can be applied here in two powerful ways:

**1. Opportunity Encoding**
Each discovered airdrop/freebie can be encoded as a Σ-vector using your 256 primitive system. This creates an ultra-compact representation that enables:
• Lightning-fast similarity search against historical opportunities
• Pattern matching: "this looks like the Arbitrum airdrop pattern from 2023"
• Anomaly detection: opportunities that don't match ANY known pattern = higher risk score
• Compressed storage: thousands of opportunity records in minimal space

**2. Communication Protocol**
If you ever expand to a multi-agent architecture (e.g., distributed scraping across machines), ΣLANG-compressed messages between agents would minimize bandwidth while maintaining semantic fidelity. A discovery agent could compress its findings into Σ-encoded packets that the evaluation agent decodes instantly.

**3. Log Compression**
ΣHARVEST will generate massive amounts of log data from continuous scraping. ΣLANG's 70%+ compression with >0.85 semantic fidelity is ideal for:
• Compressing historical claim records
• Archiving discovery logs
• Creating semantic indices of past opportunities

This could make ΣHARVEST a real-world proving ground for ΣLANG Phase 2.

**Torchhd Integration Note**: Since you're optimizing Torchhd for CPU/Ryzen (not GPU), the HD vector operations for opportunity encoding would run efficiently on your dev machine without needing CUDA.
`
  }
];

const phaseColors = {
  "VIS": "#00FF41",
  "ARC": "#00D4FF",
  "MOD": "#FF6B35",
  "UI": "#BD00FF",
  "CHAIN": "#FFB800",
  "SEC": "#FF0055",
  "TECH": "#00FFA3",
  "ROAD": "#4ECDC4",
  "REPO": "#A8E6CF",
  "SIGMA": "#FFD700",
};

function getRefColor(id) {
  const prefix = id.split("-")[0];
  return phaseColors[prefix] || "#00FF41";
}

export default function SigmaHarvestBlueprint() {
  const [activeSection, setActiveSection] = useState(0);
  const [expandedNav, setExpandedNav] = useState(true);

  const renderMarkdown = (text) => {
    return text
      .replace(/\*\*(.+?)\*\*/g, '<strong style="color:#00FF41">$1</strong>')
      .replace(/\*(.+?)\*/g, '<em style="color:#8892b0">$1</em>')
      .replace(/`([^`]+)`/g, '<code style="background:#1a2332;color:#FFB800;padding:2px 6px;border-radius:3px;font-size:0.85em">$1</code>')
      .replace(/```(\w*)\n([\s\S]*?)```/g, (_, lang, code) => 
        `<pre style="background:#0d1117;border:1px solid #1e2d3d;border-radius:8px;padding:16px;overflow-x:auto;font-size:0.82em;line-height:1.5;color:#c9d1d9;margin:12px 0"><code>${code.replace(/</g, '&lt;').replace(/>/g, '&gt;')}</code></pre>`
      )
      .replace(/^• (.+)$/gm, '<div style="padding-left:16px;margin:4px 0"><span style="color:#00FF41;margin-right:8px">›</span>$1</div>')
      .replace(/\n/g, '<br/>');
  };

  const section = sections[activeSection];

  return (
    <div style={{
      minHeight: "100vh",
      background: "#0A0E1A",
      color: "#c9d1d9",
      fontFamily: "'JetBrains Mono', 'Fira Code', 'SF Mono', monospace",
      display: "flex",
      flexDirection: "column",
    }}>
      {/* Header */}
      <div style={{
        background: "linear-gradient(135deg, #0A0E1A 0%, #0d1a2d 50%, #0A0E1A 100%)",
        borderBottom: "1px solid #1e2d3d",
        padding: "20px 24px",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
      }}>
        <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
          <div style={{
            width: 44,
            height: 44,
            borderRadius: 10,
            background: "linear-gradient(135deg, #00FF41, #00D4FF)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: 22,
            fontWeight: 900,
            color: "#0A0E1A",
          }}>Σ</div>
          <div>
            <div style={{
              fontSize: 20,
              fontWeight: 700,
              color: "#00FF41",
              letterSpacing: "0.05em",
            }}>ΣHARVEST</div>
            <div style={{
              fontSize: 11,
              color: "#4a5568",
              letterSpacing: "0.15em",
              textTransform: "uppercase",
            }}>Web3 Autonomous Freebie Collection System</div>
          </div>
        </div>
        <div style={{
          display: "flex",
          gap: 8,
          alignItems: "center",
        }}>
          <div style={{
            padding: "4px 10px",
            borderRadius: 4,
            background: "#1a2332",
            border: "1px solid #1e2d3d",
            fontSize: 11,
            color: "#FFB800",
          }}>PERSONAL USE ONLY</div>
          <div style={{
            padding: "4px 10px",
            borderRadius: 4,
            background: "#1a2332",
            border: "1px solid #1e2d3d",
            fontSize: 11,
            color: "#00FF41",
          }}>v0.1.0-blueprint</div>
        </div>
      </div>

      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        {/* Sidebar Nav */}
        <div style={{
          width: expandedNav ? 280 : 56,
          background: "#0d1117",
          borderRight: "1px solid #1e2d3d",
          transition: "width 0.2s ease",
          overflow: "hidden",
          flexShrink: 0,
          display: "flex",
          flexDirection: "column",
        }}>
          <button
            onClick={() => setExpandedNav(!expandedNav)}
            style={{
              background: "none",
              border: "none",
              borderBottom: "1px solid #1e2d3d",
              color: "#4a5568",
              padding: "12px",
              cursor: "pointer",
              fontSize: 12,
              textAlign: "left",
              whiteSpace: "nowrap",
            }}
          >
            {expandedNav ? "◀ Collapse" : "▶"}
          </button>
          <div style={{ flex: 1, overflowY: "auto", padding: "8px 0" }}>
            {sections.map((s, i) => (
              <button
                key={s.id}
                onClick={() => setActiveSection(i)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 10,
                  width: "100%",
                  padding: expandedNav ? "10px 16px" : "10px 16px",
                  background: i === activeSection ? "#1a2332" : "transparent",
                  border: "none",
                  borderLeft: i === activeSection ? `3px solid ${getRefColor(s.id)}` : "3px solid transparent",
                  color: i === activeSection ? "#e2e8f0" : "#4a5568",
                  cursor: "pointer",
                  fontSize: 12,
                  textAlign: "left",
                  whiteSpace: "nowrap",
                  transition: "all 0.15s ease",
                }}
              >
                <span style={{ fontSize: 16, flexShrink: 0 }}>{s.icon}</span>
                {expandedNav && (
                  <div style={{ overflow: "hidden" }}>
                    <div style={{
                      fontSize: 10,
                      color: getRefColor(s.id),
                      fontWeight: 600,
                      letterSpacing: "0.1em",
                    }}>[REF:{s.id}]</div>
                    <div style={{ fontSize: 12, marginTop: 2 }}>{s.title}</div>
                  </div>
                )}
              </button>
            ))}
          </div>
        </div>

        {/* Main Content */}
        <div style={{
          flex: 1,
          overflow: "auto",
          padding: "32px 40px",
        }}>
          {/* Section Header */}
          <div style={{
            display: "flex",
            alignItems: "center",
            gap: 16,
            marginBottom: 8,
          }}>
            <span style={{ fontSize: 32 }}>{section.icon}</span>
            <div>
              <div style={{
                fontSize: 10,
                color: getRefColor(section.id),
                fontWeight: 700,
                letterSpacing: "0.15em",
                marginBottom: 4,
              }}>[REF:{section.id}]</div>
              <h1 style={{
                fontSize: 24,
                fontWeight: 700,
                color: "#e2e8f0",
                margin: 0,
              }}>{section.title}</h1>
            </div>
          </div>

          <div style={{
            height: 1,
            background: `linear-gradient(90deg, ${getRefColor(section.id)}, transparent)`,
            margin: "16px 0 24px",
          }} />

          {/* Content */}
          <div
            style={{
              fontSize: 13.5,
              lineHeight: 1.75,
              color: "#a0aec0",
              maxWidth: 800,
            }}
            dangerouslySetInnerHTML={{ __html: renderMarkdown(section.content) }}
          />

          {/* Navigation */}
          <div style={{
            display: "flex",
            justifyContent: "space-between",
            marginTop: 40,
            paddingTop: 20,
            borderTop: "1px solid #1e2d3d",
          }}>
            <button
              onClick={() => setActiveSection(Math.max(0, activeSection - 1))}
              disabled={activeSection === 0}
              style={{
                background: activeSection === 0 ? "#0d1117" : "#1a2332",
                border: "1px solid #1e2d3d",
                color: activeSection === 0 ? "#2d3748" : "#00FF41",
                padding: "8px 20px",
                borderRadius: 6,
                cursor: activeSection === 0 ? "default" : "pointer",
                fontSize: 12,
                fontFamily: "inherit",
              }}
            >
              ← Previous
            </button>
            <span style={{
              fontSize: 11,
              color: "#4a5568",
              alignSelf: "center",
            }}>
              {activeSection + 1} / {sections.length}
            </span>
            <button
              onClick={() => setActiveSection(Math.min(sections.length - 1, activeSection + 1))}
              disabled={activeSection === sections.length - 1}
              style={{
                background: activeSection === sections.length - 1 ? "#0d1117" : "#1a2332",
                border: "1px solid #1e2d3d",
                color: activeSection === sections.length - 1 ? "#2d3748" : "#00FF41",
                padding: "8px 20px",
                borderRadius: 6,
                cursor: activeSection === sections.length - 1 ? "default" : "pointer",
                fontSize: 12,
                fontFamily: "inherit",
              }}
            >
              Next →
            </button>
          </div>
        </div>
      </div>

      {/* Footer */}
      <div style={{
        background: "#0d1117",
        borderTop: "1px solid #1e2d3d",
        padding: "10px 24px",
        display: "flex",
        justifyContent: "space-between",
        fontSize: 11,
        color: "#2d3748",
      }}>
        <span>ΣHARVEST Blueprint v0.1.0 — iamthegreatdestroyer</span>
        <span>Discuss any section: "Let's discuss [REF:XX-NNN]"</span>
      </div>
    </div>
  );
}
