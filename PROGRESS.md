# Solana Ecosystem Token — PROGRESS.md
# Last updated: Session 8 | Commit: bee8145
# Repo: https://github.com/laithqc-create/Solana-coin

---

## 🔴 SESSION RULES — READ FIRST, EVERY SESSION

1. **Token monitoring** — Warn at 70% context. Stop at 85%.
2. **Before limit** — Save PROGRESS.md → git commit → git push.
3. **Session start** — Read this file FIRST. Summarize. Confirm. Then code.
4. **Structure** — Completed / Current file / Next steps / Blockers.
5. **License check** — MIT/Apache 2.0/BSD only. Verify before using libs.
6. **Clean-room** — GPL/AGPL: learn only, never copy.
7. **Architecture** — Analyze → clean new code → wire in. No copy-paste-modify.
8. **Production ready** — checked_* math, require!() validation, env vars.

---

## ✅ COMPLETED

### Session 8 (current):
- Replaced Morpho with dual investment strategy:
  - USDC pool: 80% → sUSDS via Sky Protocol native bridge
  - USDT pool: 80% → sUSDe via Ethena through Meteora
  - 20% stays liquid in both pools
- New files: `yield_strategy.rs`, `investment_instructions.rs`
- License verified: Sky (Apache 2.0) ✅, Ethena (MIT) ✅, Wormhole (Apache 2.0) ✅
- 27 total instructions in lib.rs
- All math: checked_* / saturating_* — no silent overflows
- Keeper reports validated: 10% slippage tolerance, future timestamp guard

### Architecture (ALL LOCKED):

| Component | Decision |
|-----------|----------|
| Token peg | Fixed 1:1 USDT always |
| Mint/burn fee | Launchpad: 0.1% (≤ purchase) / 0.5% (excess). Standard: 0.5% |
| Fee destination | 100% → protocol treasury |
| Yield model | Auto-yield on hold (no staking) |
| Normal APY | 45% |
| Campaign APY | 70% for 6 months |
| USDC investment | 80% → sUSDS (Sky Protocol) |
| USDT investment | 80% → sUSDe (Ethena via Meteora) |
| Liquid reserve | 20% both pools |
| Revenue split | 4-way, admin-configurable (must sum to 10,000 bps) |
| Execution model | Off-chain keeper, on-chain state + validation |
| Unstaking | 48 business hours (weekends excluded) |

### Files (current state):
```
src/
├── lib.rs                      — 27 instructions (bee8145)
├── state.rs                    — All account structs incl. SkyPosition, EthenaPosition, PoolState
├── instructions.rs             — Core: mint, burn, yield, unstake, admin
├── investment_instructions.rs  — NEW: sUSDS + sUSDe invest/report/withdraw
├── yield_strategy.rs           — NEW: split_investment, validate_keeper_report, APY calc
├── morpho.rs                   — KEPT: fee calculation + yield index model
├── rwa.rs                      — KEPT: business_seconds_elapsed for unstake timer
└── errors.rs                   — 27 error variants
```

---

## 🔧 BLOCKERS

### BLOCKER 1: Compilation needs Windows Solana toolchain
```powershell
iwr https://github.com/anza-xyz/agave/releases/download/v1.18.26/solana-release-x86_64-pc-windows-msvc.tar.bz2 -OutFile solana.tar.bz2
tar -xjf solana.tar.bz2
$env:PATH += ";$(pwd)\solana-release\bin"
cd Solana-coin/ecosystem-token
cargo build-sbf --manifest-path programs/ecosystem-token/Cargo.toml
```

### BLOCKER 2: Program ID is placeholder
- After Windows compile: `solana-keygen new -o program-keypair.json`
- Update `declare_id!()` in lib.rs

### BLOCKER 3: Dashboard not wired
- `useEcosystemToken.ts` uses mock data
- Wire after deployment with real Program ID

---

## 📋 RESUME FROM HERE (Session 9)

**Step 1** — Confirm: Any more smart contract changes before compiling?
**Step 2** — Windows compile walkthrough
**Step 3** — Generate real program ID
**Step 4** — Update lib.rs declare_id!()
**Step 5** — Deploy to devnet
**Step 6** — Wire dashboard to on-chain data
**Step 7** — End-to-end test all flows

---

⚠️ Context warning: Start fresh session for Step 2+
