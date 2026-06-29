# Solana Ecosystem Token — PROGRESS.md
# Last updated: Session 7 | Commit: f4e48c3
# Repo: https://github.com/laithqc-create/Solana-coin

---

## 🔴 SESSION RULES (ENFORCED EVERY SESSION)
## ⚠️ READ THIS FIRST — BEFORE DOING ANYTHING ELSE

**RULE: On session start, Claude MUST:**
1. Read this file IMMEDIATELY
2. Summarize what was done and what's next
3. Ask user to confirm before continuing
4. If context already feels heavy → warn immediately

1. **Token usage** — Warn proactively when context is getting heavy.
2. **Before context limit** — Save state to PROGRESS.md + push to GitHub.
3. **Start of every session** — Read PROGRESS.md first, summarize, confirm.
4. **PROGRESS.md structure** — Completed / Current file / Next steps / Blockers.
5. **License checking** — Verify MIT/Apache 2.0/BSD before using any open-source.
6. **Clean-room** — GPL/AGPL: learn logic only, never copy.
7. **Architecture** — Analyze → write clean → wire in. No copy-paste-modify.
8. **Production ready** — checked_* math, require!() validation, env vars, no loops.

---

## ✅ COMPLETED THIS SESSION (Session 7)

### Architecture decisions changed (ALL LOCKED):

| Decision | Old | New |
|----------|-----|-----|
| Yield model | Staking required | **Auto-yield on hold** (no staking needed) |
| Normal APY | 5% (RUSDY) | **45% APY** |
| Campaign APY | N/A | **70% APY for 6 months** from purchase |
| Campaign incentive | Discount % | **Higher yield** only |
| Investment partner | RUSDY/Jupiter | **Morpho Protocol** |
| Investment % | 70% | **80% of pool** |
| Mint/burn fee - launchpad | 0.1% split | **0.1% on ≤ purchase, 0.5% on excess → 100% protocol** |
| Mint/burn fee - standard | 0.5% | **0.5% → 100% protocol treasury** |
| Revenue split | Fixed 70/30 | **4-way split, admin-configurable in dashboard** |
| Morpho pools | Auto | **Admin approval required before investment** |

### New files:
- `morpho.rs` — Morpho investment module (replaces rwa.rs for investment logic)
  - `accrue_yield_index()` — global index model (like Aave's liquidity index)
  - `calculate_holder_yield()` — per-holder yield from index delta
  - `holder_apy_bps()` — 70% campaign / 45% normal
  - `calculate_fee()` — 3-tier fee logic (launchpad / standard / bot)
  - `is_eligible_pool_type()` — Morpho pool eligibility check
  - `calculate_morpho_yield()` — pool yield calculation
  - All constants: YIELD_INDEX_PRECISION, NORMAL/CAMPAIGN_YIELD_BPS, etc.

### Updated files:
- `state.rs` — New: HolderInfo, MorphoPool, MorphoPoolType enum. Updated: YieldConfig, TreasuryVault, RevenueDistribution
- `instructions.rs` — Complete rewrite (1,033 lines). New: propose_morpho_pool, approve_morpho_pool, invest_in_morpho, report_morpho_yield, update_revenue_split, claim_yield (auto)
- `errors.rs` — New errors: InvestmentPendingApproval, IneligiblePoolType, PoolNotApproved, etc.
- `lib.rs` — All 15 instructions registered

### Key logic implemented:

**Auto-yield (no staking):**
```
On mint/burn/claim: accrue global_yield_index
holder_yield = (current_index - snapshot) * balance / PRECISION
Campaign buyers (is_campaign=true): 70% APY for 6 months
All others: 45% APY
```

**Fee tiers (100% → protocol treasury):**
```
Launchpad investor, amount ≤ original: 0.1%
Launchpad investor, amount > original: (original × 0.1%) + (excess × 0.5%)
Standard trader / bot: 0.5%
```

**Morpho pool approval flow:**
```
1. Admin calls propose_morpho_pool (must pass type check)
2. Pool sits in "pending" state
3. Admin calls approve_morpho_pool in dashboard
4. investment_pending_approval = false → investment enabled
5. invest_in_morpho routes 80% of liquid USDC
6. Off-chain keeper executes cross-chain transfer
7. Keeper calls report_morpho_yield on-chain with earned amount
```

**Revenue split (admin-configurable):**
```
update_revenue_split(holder_bps, marketing_bps, manager_bps, protocol_bps, investment_ratio_bps)
Must sum to 10,000 bps (100%)
Default: 25% / 25% / 25% / 25%
```

---

## 🔧 CURRENT BLOCKERS

### BLOCKER 1: Compilation (unchanged)
- Need `cargo build-sbf` with Solana toolchain on Windows
- Standard `cargo build` = x86, not deployable to Solana

### BLOCKER 2: Program ID (unchanged)
- Current: `11111111111111111111111111111112` (placeholder)
- Generate real ID after Windows compile

### BLOCKER 3: Morpho is on Ethereum/Base (architecture note)
- Morpho Protocol lives on Ethereum/Base, not Solana
- Investment execution: off-chain keeper bridges USDC via Wormhole/LayerZero
- On-chain contract tracks intent + reports results
- **Decision needed**: Do you want to keep Morpho cross-chain, or find a Solana-native equivalent?
  - Solana native alternatives with same criteria: MarginFi, Kamino Finance, Solend
  - These support isolated vaults + stablecoin collateral on Solana directly

### BLOCKER 4: rwa.rs still referenced
- `rwa.rs` still exists for `business_seconds_elapsed()` used in unstake
- `morpho.rs` is the new investment module
- Both coexist — `rwa.rs` handles time logic, `morpho.rs` handles investment+fees

### BLOCKER 5: Dashboard not wired
- `useEcosystemToken.ts` still uses mock data
- Needs real Program ID after deployment

---

## 📋 RESUME FROM HERE (Session 8)

### Priority 1: Confirm Morpho cross-chain vs Solana-native
**Ask user:** "Morpho is on Ethereum/Base. For Solana-native, we can use MarginFi or Kamino which have identical isolated vault criteria. Which do you prefer?"

### Priority 2: Windows compile
```powershell
# Install Solana CLI
iwr https://github.com/anza-xyz/agave/releases/download/v1.18.26/solana-release-x86_64-pc-windows-msvc.tar.bz2 -OutFile solana.tar.bz2
tar -xjf solana.tar.bz2
$env:PATH += ";$(pwd)\solana-release\bin"

# Clone and build
git clone https://github.com/laithqc-create/Solana-coin
cd Solana-coin/ecosystem-token
cargo build-sbf --manifest-path programs/ecosystem-token/Cargo.toml
```

### Priority 3: Update PROGRESS.md with Program ID after compile

### Priority 4: Wire dashboard to real on-chain data

---

## 📁 CURRENT FILE STATE

```
ecosystem-token/programs/ecosystem-token/src/
├── lib.rs          — 15 instructions registered (commit f4e48c3)
├── state.rs        — HolderInfo, MorphoPool, YieldConfig, TreasuryVault, RevenueDistribution
├── instructions.rs — 1,033 lines, complete rewrite
├── morpho.rs       — NEW: investment + fee + yield index module
├── rwa.rs          — KEPT: business_seconds_elapsed() for unstake timer
└── errors.rs       — 27 error variants
```

---

## ⚠️ CONTEXT WARNING
**This session hit context limits quickly because PROGRESS.md was not read at start.**
**Next session: Claude MUST read this file as the VERY FIRST action.**
