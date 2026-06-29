# Solana Ecosystem Token — PROGRESS.md
# Last updated: Session 8 | LATEST COMMIT: 42b6e4c
# Repo: https://github.com/laithqc-create/Solana-coin
# Branch: main

---

## 🔴 SESSION RULES — CLAUDE MUST READ THIS FIRST, EVERY SESSION

1. **Token monitoring** — Warn user at 70% context. Hard stop at 85%. Push + update PROGRESS.md.
2. **Before limit** — Save PROGRESS.md → `git add . && git commit && git push` → write RESUME section.
3. **Session start** — Read PROGRESS.md FIRST. Summarize to user. Wait for confirm. Then code.
4. **Structure** — Always: Completed / Current file / Next steps / Blockers.
5. **License check** — MIT/Apache 2.0/BSD only. Verify before using any library.
6. **Clean-room** — GPL/AGPL/CC-BY-NC: learn logic only, never copy code.
7. **Architecture** — Analyze codebase first → write clean new code → wire in. No copy-paste-modify.
8. **Production ready** — checked_* math, require!() input validation, env vars for secrets, no redundant loops.

---

## ✅ FULLY COMPLETED (All Sessions)

### Smart Contract — Rust/Anchor
**Location:** `ecosystem-token/programs/ecosystem-token/src/`
**Total instructions:** 27
**Latest commit:** `42b6e4c`

#### All 8 source files:

| File | Purpose | Lines |
|------|---------|-------|
| `lib.rs` | Instruction router — registers all 27 instructions | ~120 |
| `state.rs` | All account structs | ~250 |
| `instructions.rs` | Core: mint, burn, auto-yield, unstake, admin | ~500 |
| `investment_instructions.rs` | sUSDS + sUSDe invest/report/withdraw | ~370 |
| `yield_strategy.rs` | split_investment, APY calc, keeper validation | ~270 |
| `morpho.rs` | Fee calculation + yield index model | ~285 |
| `rwa.rs` | business_seconds_elapsed() for unstake timer | ~120 |
| `errors.rs` | 27 error variants | ~80 |

#### All 27 instructions:

**Core (instructions.rs):**
- `initialize_launchpad` — sets up launchpad, blocks investment until admin approves pool
- `initialize_treasury` — sets up treasury, revenue dist, yield config (25/25/25/25 default)
- `mint_tokens(usdc_amount, is_campaign)` — fixed 1:1 peg, fee from pool, auto-enrols in yield
- `burn_tokens(token_amount)` — fixed 1:1 peg, fee to protocol treasury, burns tokens
- `claim_yield` — claim auto-accrued yield (no staking needed, just hold)
- `request_unstake(amount)` — starts 48 business-hour countdown
- `complete_unstake` — after 48 business hours (weekends excluded)
- `emergency_redeem_defi` — immediate exit with slippage warning
- `update_revenue_split` — admin changes 4-way split ratios
- `distribute_revenue` — distribute yield to 4 parties
- `pause_launchpad` / `resume_launchpad`

**Strategy 1 — USDC → sUSDS (Sky Protocol) (investment_instructions.rs):**
- `initialize_sky_position` — admin sets up Sky position account
- `invest_usdc_to_susds` — locks 80% USDC, 20% stays liquid, keeper executes bridge
- `confirm_sky_investment(susds_balance, report_ts)` — keeper confirms position live
- `report_sky_yield(new_balance, new_apy_bps, report_ts)` — keeper reports yield
- `withdraw_from_susds` — initiates unwinding back to USDC

**Strategy 2 — USDT → sUSDe (Ethena via Meteora) (investment_instructions.rs):**
- `initialize_ethena_position` — admin sets up Ethena position account
- `invest_usdt_to_susde` — locks 80% USDT, 20% stays liquid, keeper routes via Meteora
- `confirm_ethena_investment(susde_balance, report_ts)` — keeper confirms
- `report_ethena_yield(new_balance, new_apy_bps, report_ts)` — keeper reports
- `withdraw_from_susde` — initiates unwinding back to USDT

#### All locked architecture decisions:

| Decision | Value |
|----------|-------|
| Token peg | Fixed 1:1 USDT — NEVER changes with market |
| Mint fee — launchpad investor | 0.1% on ≤ original purchase; 0.5% on excess |
| Mint fee — standard/bot trader | 0.5% always |
| Burn fee | Same tiers as mint |
| Fee destination | 100% → protocol treasury (no split) |
| Transfer tax | REMOVED — no tax on wallet-to-wallet transfers |
| Supply cap | REMOVED — unlimited minting |
| Yield model | Auto-yield on hold (no staking required) |
| Normal holder APY | 45% |
| Campaign buyer APY | 70% for first 6 months after purchase |
| Campaign incentive | Higher yield only (no discounts) |
| USDC investment | 80% → sUSDS via Sky Protocol native bridge |
| USDT investment | 80% → sUSDe via Ethena through Meteora |
| Liquid reserve | 20% both pools (for immediate redemptions) |
| Revenue split | 4-way: holder/marketing/manager/protocol — admin configurable |
| Fees split | NOT split — 100% to protocol treasury |
| Investment execution | Off-chain keeper pattern (cross-chain) |
| Keeper validation | 10% slippage tolerance + future-timestamp guard |
| Unstaking | 3 paths: normal (48hr) / emergency DeFi / cannot bypass |
| Unstake timer | 48 business hours, Mon-Fri only, pauses Sat-Sun |
| Unstake message | "Your money is being routed from real world assets to USDC. This can take 48 business hours due to regulation. Timer pauses on weekends." |
| Emergency message | "Your transaction faces potential high slippage due to AMM market conditions." |

#### License audit (Rule 5) ✅:
- anchor-lang 0.29.0 → Apache 2.0 ✅
- anchor-spl 0.29.0 → Apache 2.0 ✅
- spl-token 3.5.0 → Apache 2.0 ✅
- rust_decimal 1.27 → MIT ✅
- Sky Protocol → Apache 2.0 ✅
- Ethena → MIT ✅
- Wormhole bridge SDK → Apache 2.0 ✅

### Dashboard — Next.js 14
**Location:** `ecosystem-token/dashboard/`
- 4 pages: User / Treasury / Analytics / Admin
- Hook: `useEcosystemToken.ts` — currently mock data
- Status: NOT yet wired to on-chain data (blocked on deploy)

---

## 🔧 BLOCKERS (must resolve in order)

### BLOCKER 1: Need Windows compile with Solana toolchain
Standard `cargo build --release` produces x86 — NOT deployable to Solana.
Need `cargo build-sbf` which cross-compiles to SBF bytecode.

```powershell
# Step 1: Download Solana tools (in PowerShell as Administrator)
iwr https://github.com/anza-xyz/agave/releases/download/v1.18.26/solana-release-x86_64-pc-windows-msvc.tar.bz2 -OutFile solana.tar.bz2
tar -xjf solana.tar.bz2
$env:PATH += ";$(pwd)\solana-release\bin"

# Step 2: Install Rust (if not already installed)
# Download from: https://rustup.rs

# Step 3: Clone and build
git clone https://github.com/laithqc-create/Solana-coin
cd Solana-coin\ecosystem-token
cargo build-sbf --manifest-path programs/ecosystem-token/Cargo.toml

# Output: target/deploy/ecosystem_token.so  ← SBF bytecode, deployable
```

### BLOCKER 2: Program ID is placeholder
Current `declare_id!("11111111111111111111111111111112")` is wrong.

```powershell
# After building:
solana-keygen new -o program-keypair.json
# Copy the address shown, update lib.rs declare_id!("YOUR_REAL_ID")
# Commit + push before deploying
```

### BLOCKER 3: Dashboard not wired
After deploy, set in `dashboard/.env.local`:
```
NEXT_PUBLIC_PROGRAM_ID=<your-program-id>
NEXT_PUBLIC_SOLANA_NETWORK=devnet
```
Then update `useEcosystemToken.ts` with real RPC calls.

---

## 📋 RESUME FROM HERE — Session 9

**Step 1** — Ask user: "Any more smart contract changes before we compile?"
**Step 2** — Walk through Windows Solana install (BLOCKER 1 above)
**Step 3** — Generate real Program ID (BLOCKER 2)
**Step 4** — Update `declare_id!()` in lib.rs, commit, push
**Step 5** — Deploy to Solana devnet
**Step 6** — Wire dashboard to on-chain data (BLOCKER 3)
**Step 7** — End-to-end testing:
  - Mint (verify 1:1 peg + correct fee tier)
  - Burn (verify 1:1 return + fee to treasury)
  - Yield claim (verify auto-accrual)
  - Request unstake → wait → complete (verify 48hr timer)
  - Emergency redeem (verify slippage warning)
  - Revenue split update from admin dashboard
  - sUSDS investment flow
  - sUSDe investment flow

---

## 📁 FULL FILE MAP

```
/home/claude/ (or clone on Windows)
├── PROGRESS.md                              ← THIS FILE
├── ecosystem-token/
│   ├── Cargo.toml                           ← workspace (anchor 0.29, spl-token 3.5)
│   ├── .gitignore                           ← excludes Cargo.lock, target/
│   ├── programs/ecosystem-token/
│   │   ├── Cargo.toml                       ← program deps + init-if-needed feature
│   │   └── src/
│   │       ├── lib.rs                       ← 27 instructions registered
│   │       ├── state.rs                     ← all account structs
│   │       ├── instructions.rs              ← core logic
│   │       ├── investment_instructions.rs   ← sUSDS + sUSDe strategies
│   │       ├── yield_strategy.rs            ← shared math + validation
│   │       ├── morpho.rs                    ← fee tiers + yield index
│   │       ├── rwa.rs                       ← business hours timer
│   │       └── errors.rs                    ← 27 error codes
│   └── dashboard/
│       ├── package.json
│       └── src/
│           ├── app/page.tsx                 ← User dashboard
│           ├── app/treasury/page.tsx        ← Treasury
│           ├── app/analytics/page.tsx       ← Analytics
│           ├── app/admin/page.tsx           ← Admin
│           └── hooks/useEcosystemToken.ts   ← Mock data (needs wiring)
└── .github/workflows/build.yml             ← CI/CD (x86 test only)
```

