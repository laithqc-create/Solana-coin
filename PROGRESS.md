# Solana Ecosystem Token — PROGRESS.md
# Last updated: Session 6 | Commit: 8e090ed
# Repo: https://github.com/laithqc-create/Solana-coin

---

## 🔴 SESSION RULES (ENFORCED EVERY SESSION)

1. **Token usage** — Warn proactively when context is getting heavy.
2. **Before context limit** — Save state to PROGRESS.md + write "RESUME FROM HERE".
3. **Start of every session** — Read PROGRESS.md first, summarize, ask to confirm.
4. **PROGRESS.md structure** — Completed / Current file / Next steps / Blockers.
5. **License checking** — Verify MIT/Apache 2.0/BSD before using any open-source code.
6. **Clean-room development** — GPL/AGPL/CC-BY-NC: learn logic only, never copy.
7. **Architectural integration** — No copy-paste-modify. Analyze → write clean → wire in.
8. **Production ready** — Robust error handling, input validation, env vars for secrets, no redundant loops.

---

## ✅ COMPLETED

### Infrastructure
- GitHub repo: https://github.com/laithqc-create/Solana-coin
- GitHub Actions CI/CD (cargo build --release, produces x86 .so for testing)
- Note: **Real SBF binary requires Windows local build with Solana toolchain**

### Smart Contract (`ecosystem-token/programs/ecosystem-token/src/`)

#### Architecture decisions (LOCKED):
| Decision | Value |
|----------|-------|
| Peg | Fixed 1 USDT = 1 token ALWAYS (no market pricing) |
| Mint fee | 0.1% from pool (user gets full 1:1, pool absorbs fee) |
| Burn fee | 0.1% from pool (user gets full 1:1 USDC back, pool absorbs fee) |
| Transfer tax | REMOVED — no tax on wallet-to-wallet transfers |
| Fee split | 70% → treasury (RUSDY via Jupiter DCA) / 30% → yield pool (stakers) |
| Supply cap | REMOVED — unlimited minting |
| Yield source | RUSDY (real world asset) via Jupiter DCA — replaced Aave |
| Unstaking | 3-path flow (see below) |

#### Files:
- `lib.rs` — 20 instruction handlers registered
- `state.rs` — 10 account structs with all fields synced
- `instructions.rs` — All business logic (~1,100 lines)
- `rwa.rs` — RUSDY/Jupiter DCA module (replaces aave.rs)
- `errors.rs` — EcosystemError enum (~22 variants)

#### Instructions implemented:
| Instruction | Status |
|-------------|--------|
| initialize_launchpad | ✅ |
| initialize_treasury | ✅ |
| mint_tokens | ✅ Fixed 1:1 peg, 0.1% fee from pool |
| burn_tokens | ✅ Fixed 1:1 peg, 0.1% fee from pool, burns tokens |
| stake_tokens | ✅ With vesting lock check |
| request_unstake | ✅ Starts 48hr business-hour countdown |
| complete_unstake | ✅ Completes after 48 business hours |
| emergency_redeem_defi | ✅ Immediate exit with slippage warning |
| create_yield_snapshot | ✅ |
| claim_yield | ✅ |
| set_tier2_whitelist | ✅ |
| invest_in_rwa | ✅ RUSDY via Jupiter DCA (keeper executes) |
| claim_rwa_yields | ✅ ~5% APY simulation |
| distribute_revenue | ✅ 40/20/20/20 split |
| update_allocation_percentages | ✅ |
| pause_launchpad | ✅ |
| resume_launchpad | ✅ |
| request_unstake | ✅ |
| complete_unstake | ✅ |
| emergency_redeem_defi | ✅ |

#### Unstaking flow (3 paths):
```
Path 1 (normal): request_unstake → wait 48 business hrs → complete_unstake
  - Timer pauses on weekends (Sat + Sun excluded from 48hr count)
  - Dashboard shows countdown + "funds routing from RUSDY to USDC"
  - Message: "Your money is being routed from real world assets to USDC.
              Please wait — this can take up to 48 business hours due to regulation."

Path 2 (emergency): request_unstake → emergency_redeem_defi
  - Immediate exit
  - Warning: "Your transaction may face high slippage due to AMM market conditions."

Path 3: Never — cannot bypass without a pending request
```

#### RUSDY/RWA Module (`rwa.rs`):
- `business_seconds_elapsed()` — calculates Mon-Fri only elapsed time
- `is_weekend()` — day_of_week check (0=Sun, 6=Sat excluded)
- `calculate_rwa_yield()` — 5% APY simple interest
- `record_rwa_investment()` — 70% treasury → RUSDY
- `calculate_emergency_redemption()` — slippage-adjusted amount
- UNSTAKE_BUSINESS_SECONDS = 172,800 (48 × 3600)

### Dashboard (`ecosystem-token/dashboard/`)
- Next.js 14, TypeScript, React 18, TailwindCSS
- 4 pages: User / Treasury / Analytics / Admin
- Hook: `useEcosystemToken.ts` (mock data — needs on-chain wiring after deploy)

---

## 🔧 CURRENT BLOCKERS

### BLOCKER 1: Compilation requires Solana toolchain
- `cargo build --release` (standard Rust) produces x86 .so — not deployable
- Need `cargo build-sbf` with Solana platform tools
- **Solution: Compile on Windows machine**

### BLOCKER 2: Program ID is placeholder
- Current: `11111111111111111111111111111112` (system program — wrong!)
- After Windows compile: run `solana-keygen new` to generate real program ID
- Update `declare_id!()` before deployment

### BLOCKER 3: RUSDY mint address is placeholder
- Current: `"RuSDyTokenMintAddress1111111111111111111111"` in `rwa.rs`
- Need: actual RUSDY token mint address on Solana mainnet/devnet

### BLOCKER 4: Dashboard not wired to contract
- `useEcosystemToken.ts` uses mock data
- Needs: real Program ID + RPC connection after deployment

---

## 📋 EXACT NEXT STEPS (RESUME FROM HERE)

### Step 1: Windows local compile
```powershell
# In Windows PowerShell (as Administrator):

# Install Rust
winget install Rustlang.Rust

# Install Solana CLI (from Anza/Agave)
iwr https://github.com/anza-xyz/agave/releases/download/v1.18.26/solana-release-x86_64-pc-windows-msvc.tar.bz2 -OutFile solana.tar.bz2
tar -xjf solana.tar.bz2

# Add to PATH
$env:PATH += ";$(pwd)\solana-release\bin"

# Clone repo
git clone https://github.com/laithqc-create/Solana-coin
cd Solana-coin/ecosystem-token

# Build (produces real SBF .so file)
cargo build-sbf --manifest-path programs/ecosystem-token/Cargo.toml

# Find output
ls target/deploy/ecosystem_token.so
```

### Step 2: Generate real Program ID
```powershell
solana-keygen new -o program-keypair.json
solana address -k program-keypair.json
# Copy the output address
```

### Step 3: Update declare_id! in lib.rs
```rust
// Replace: 11111111111111111111111111111112
// With your real program ID from step 2
declare_id!("YourRealProgramID...");
```

### Step 4: Deploy to devnet
```powershell
solana config set --url devnet
solana airdrop 2
solana program deploy target/deploy/ecosystem_token.so --keypair program-keypair.json
```

### Step 5: Update RUSDY mint address in rwa.rs
- Find RUSDY token on Solana devnet/mainnet
- Replace `RUSDY_MINT` constant

### Step 6: Wire dashboard
- Set `NEXT_PUBLIC_PROGRAM_ID=<your-program-id>` in dashboard/.env.local
- Update `useEcosystemToken.ts` to use real RPC calls

### Step 7: End-to-end testing
- Mint tokens (verify 1:1 peg + pool fee)
- Burn tokens (verify 1:1 return + pool fee)
- Stake → request unstake → wait 48hrs → complete
- Emergency redeem with slippage warning
- Verify RUSDY yield accumulation

---

## 🏗️ RULE COMPLIANCE AUDIT

### Rule 5 (License checking):
- anchor-lang 0.29.0 → Apache 2.0 ✅ commercial OK
- anchor-spl 0.29.0 → Apache 2.0 ✅ commercial OK
- spl-token 3.5.0 → Apache 2.0 ✅ commercial OK
- rust_decimal 1.27 → MIT ✅ commercial OK
- Next.js → MIT ✅ commercial OK

### Rule 6 (Clean-room):
- Jupiter DCA integration: off-chain keeper pattern written from scratch ✅
- No copied Anchor examples — all logic independently written ✅

### Rule 7 (Architecture):
- Each instruction is a separate function, not copy-pasted ✅
- rwa.rs is a new module, not a modified aave.rs ✅
- Error types in dedicated errors.rs ✅

### Rule 8 (Production ready):
- All math uses checked_mul/checked_add/checked_div → no silent overflow ✅
- require!() validates all inputs before state changes ✅
- No API keys in code (all Solana RPCs are public endpoints) ✅
- No redundant loops in hot paths ✅
- TODO: audit dashboard for env var secrets before mainnet ⚠️

---

## 📁 FILE LOCATIONS

```
/home/claude/
├── PROGRESS.md                          ← THIS FILE
├── ecosystem-token/
│   ├── Cargo.toml                       ← workspace deps
│   ├── .gitignore
│   ├── programs/ecosystem-token/
│   │   ├── Cargo.toml                   ← program deps
│   │   └── src/
│   │       ├── lib.rs                   ← instruction router
│   │       ├── state.rs                 ← account structs
│   │       ├── instructions.rs          ← all business logic
│   │       ├── rwa.rs                   ← RUSDY/Jupiter module
│   │       └── errors.rs               ← error codes
│   └── dashboard/
│       ├── package.json
│       ├── src/app/
│       │   ├── page.tsx                 ← User dashboard
│       │   ├── treasury/page.tsx        ← Treasury
│       │   ├── analytics/page.tsx       ← Analytics
│       │   └── admin/page.tsx           ← Admin
│       └── src/hooks/useEcosystemToken.ts
└── .github/workflows/build.yml          ← CI/CD (x86 test build)
```

---

## ⚠️ CONTEXT WARNING THRESHOLD
When this conversation exceeds ~80% context, I will:
1. Save final state to this PROGRESS.md
2. Commit and push to GitHub
3. Write "RESUME FROM HERE" section
4. Warn you to start a new session

