# Staking & Yield Distribution Guide

## Overview

The ecosystem token uses a **weekly claimable yield model** where:
- Users can optionally stake tokens
- Tax (0.1% from transfers) accumulates weekly
- Stakers claim their pro-rata yield weekly
- Yield is paid in USDC (not auto-compounded)

---

## Account Hierarchy

```
LaunchpadState
├── token_mint: SPL Token
├── usdc_mint: USDC
├── vault: Collateral ATA (USDC)
├── tax_vault: Tax ATA (USDC)
├── mint_authority: Hardcoded to vault PDA
└── authority: Multisig

Per-User Accounts:
├── UserTierInfo (PDA: ["user-tier", user])
│   ├── is_tier2: bool
│   ├── vesting: Option<VestingSchedule>
│   └── total_tokens_minted: u64
│
├── StakingInfo (PDA: ["staking", user])
│   ├── staked_amount: u64
│   ├── staked_at: i64
│   ├── last_claim: i64
│   └── total_yield_claimed: u64
│
└── User Token Accounts
    ├── Unlocked balance (can trade/redeem)
    └── Staking vault (when staked)

Global Accounts:
├── YieldConfig (PDA: ["yield-config"])
│   ├── snapshot_frequency: i64 (604800 = 7 days)
│   ├── last_snapshot: i64
│   └── next_snapshot: i64
│
└── YieldSnapshot (PDA: ["yield-snapshot", timestamp])
    ├── total_staked: u64 (sum of all staked)
    ├── tax_collected: u64 (USDC in this period)
    └── is_distributed: bool
```

---

## Instruction Flows

### 1. Stake Tokens

**Instruction**: `stake_tokens(token_amount)`
**Who**: Any user
**Prerequisites**: User has tokens in wallet

```
User Account
  token_balance: 1000 tokens

call stake_tokens(500)
  ↓
Program:
  1. Validate user has 500 tokens
  2. Transfer 500: user_account → staking_vault
  3. Create/update StakingInfo PDA
     - staked_amount = 500
     - staked_at = now
  4. User can now earn yield on 500 tokens

Result:
  user_balance: 500 tokens
  staking_vault: 500 tokens
  StakingInfo.staked_amount: 500
```

### 2. Unstake Tokens

**Instruction**: `unstake_tokens(token_amount)`
**Who**: Any user
**Prerequisites**: User has staked tokens

```
StakingInfo.staked_amount: 500 tokens

call unstake_tokens(300)
  ↓
Program:
  1. Validate staked_amount >= 300
  2. Transfer 300: staking_vault → user_account
  3. Update StakingInfo
     - staked_amount = 200 (500 - 300)
  4. User loses yield on unstaked amount

Result:
  user_balance: 800 tokens
  staking_vault: 200 tokens
  StakingInfo.staked_amount: 200
```

### 3. Create Weekly Snapshot

**Instruction**: `create_yield_snapshot()`
**Who**: Keeper (off-chain) or manual call
**When**: Every 7 days (604800 seconds)
**Prerequisites**: 7+ days since last snapshot

```
Current State:
  tax_vault: 100K USDC (accumulated from transfers)
  total_staked (summed): 10M tokens
  YieldConfig.last_snapshot: 1000 (old time)
  YieldConfig.next_snapshot: 604800 + 1000 = 605800

call create_yield_snapshot()
  ↓
Program:
  1. Validate: now >= next_snapshot
  2. Create YieldSnapshot PDA
     - snapshot_time = now
     - total_staked = 10M (sum of all StakingInfo.staked_amount)
     - tax_collected = 100K USDC (tax_vault balance)
     - is_distributed = true (ready for claims)
  3. Update YieldConfig
     - last_snapshot = now
     - next_snapshot = now + 604800

Result:
  YieldSnapshot["yield-snapshot", now] created
  YieldConfig.next_snapshot = now + 7 days
  Tax vault still holds 100K (will be drained as users claim)
```

### 4. Claim Yield

**Instruction**: `claim_yield()`
**Who**: Any user who is staked
**Prerequisites**: 
  - User has staked tokens (StakingInfo.staked_amount > 0)
  - Latest YieldSnapshot exists and is distributed

```
Setup:
  User A staked: 500K tokens
  YieldSnapshot.total_staked: 10M tokens
  YieldSnapshot.tax_collected: 100K USDC
  tax_vault: 100K USDC

call claim_yield()
  ↓
Program:
  1. Validate: staked_amount > 0
  2. Validate: latest snapshot is_distributed = true
  3. Calculate pro-rata:
     user_share = (100K * 500K) / 10M = 5K USDC
  4. Transfer 5K USDC: tax_vault → user_usdc_account
  5. Update StakingInfo
     - last_claim = now
     - total_yield_claimed += 5K

Result:
  User A wallet: +5K USDC
  tax_vault: -5K USDC (95K remaining)
  StakingInfo.total_yield_claimed: 5K
```

---

## Weekly Yield Timeline Example

### Week 1
**Monday (Day 0)**:
- Launchpad opens
- Users mint tokens and start staking
- Tax vault: empty (no transfers yet)

**Tuesday - Thursday**:
- Users trade tokens among themselves
- 0.1% tax on every transfer
- Tax accumulates: 50K USDC by Friday

**Friday (Day 5)**:
- Tax vault balance: 50K USDC
- Total staked: 5M tokens
- Keeper calls `create_yield_snapshot()`
  - YieldSnapshot created
  - total_staked = 5M
  - tax_collected = 50K
  - is_distributed = true

**Saturday - Sunday**:
- Users claim yield (if staked)
- User claiming with 500K staked (10% of total):
  - Yield = 50K * (500K / 5M) = 5K USDC
  - Calls `claim_yield()` → receives 5K
- Tax vault drains as users claim

### Week 2
**Monday (Day 7)**:
- New snapshot period begins
- Previous snapshot's remaining tax (unclaimed) stays in vault
- New tax accumulation starts
- Keeper calls `create_yield_snapshot()` again

---

## Key Design Features

### 1. Pro-Rata Distribution
```
Each staker gets: (tax_collected * staker_amount) / total_staked

Example:
  Total tax: 100K USDC
  Total staked: 10M tokens
  You staked: 1M tokens
  Your yield: 100K * (1M / 10M) = 10K USDC
```

### 2. No Lock-In Period
- Can stake/unstake anytime
- No minimum lock duration
- But: Unstaked = no yield that period

### 3. No Auto-Compounding
- Yield paid in USDC (not tokens)
- User receives USDC directly to wallet
- User could manually re-buy/stake tokens if desired

### 4. Claimable (Not Pushed)
- User must call `claim_yield()` manually
- No automatic payments
- Unclaimed yield remains in tax vault (accrues)

### 5. Hardcoded Supply Cap
- Initial supply: 100M tokens (never changes)
- Minting only happens on USDC deposit
- Mint authority: Vault PDA (no human control)

---

## Tax Vault Flow

```
Transfer Hook (Phase 2):
  User A sends 1000 tokens to User B
  → 1 token tax deducted (0.1%)
  → 999 tokens received by User B
  → 1 token value in USDC ≈ 1 USDC to tax_vault

Weekly Accumulation:
  Day 1: tax_vault = 50K USDC
  Day 2: tax_vault = 75K USDC
  ...
  Day 7: tax_vault = 100K USDC

Snapshot + Distribution:
  YieldSnapshot created: 100K tax, 10M staked
  Users claim:
    User 1 claims: -5K USDC
    User 2 claims: -8K USDC
    User 3 claims: -3K USDC
  tax_vault = 84K USDC (unclaimed yield accrues)
```

---

## Yield Scenarios

### Scenario A: Small Staker
```
You stake: 10K tokens
Total staked: 10M tokens (0.1% of pool)
Weekly tax collected: 100K USDC

Your weekly yield: 100K * (10K / 10M) = 0.1 USDC

Annual (52 weeks): 0.1 * 52 = 5.2 USDC
APY (relative to token price): 5.2 USDC / (10K tokens at 1 USDC/token) = 0.052% APY
```

### Scenario B: Major Holder
```
You stake: 100M tokens
Total staked: 500M tokens (20% of pool)
Weekly tax collected: 100K USDC

Your weekly yield: 100K * (100M / 500M) = 20K USDC

Annual (52 weeks): 20K * 52 = 1.04M USDC
APY (relative to token price): 1.04M / (100M tokens) = 1.04% APY
```

**Note**: APY scales with transfer volume. Higher trading = more tax = more yield.

---

## Governance (Multisig Authority)

The multisig controls:
- ✅ Update discount tiers
- ✅ Pause/resume launchpad
- ✅ Pause/resume transfers
- ✅ Withdraw unclaimed tax (for treasury)
- ✅ Update snapshot frequency (e.g., change from weekly to daily)

The multisig CANNOT:
- ❌ Mint arbitrary tokens (mint authority is vault PDA only)
- ❌ Withdraw collateral USDC (vault is 1:1 backed)
- ❌ Modify vesting schedules (immutable per-user)
- ❌ Change user tier (immutable per-user)

---

## Implementation Checklist

### Phase 1 ✅ (Done)
- [x] StakingInfo account structure
- [x] YieldSnapshot account structure
- [x] YieldConfig account structure
- [x] `stake_tokens` instruction
- [x] `unstake_tokens` instruction
- [x] `claim_yield` instruction
- [x] Mint authority hardcoded to vault
- [x] Initial supply cap (100M)

### Phase 2 (Next)
- [ ] Transfer hook (0.1% tax deduction)
- [ ] Vesting enforcement on transfers
- [ ] Tier 2 restriction mechanism
- [ ] Yield snapshot creation (off-chain keeper or manual)
- [ ] Emergency pause mechanism

### Phase 3 (Testing)
- [ ] Unit tests (yield calculation)
- [ ] Integration tests (stake → claim flow)
- [ ] Stress tests (1000+ stakers)

---

## FAQ

**Q: Can I stake tokens and still trade them?**
A: No, staked tokens are in a separate vault. Unstake first if you want to trade.

**Q: What if I don't stake?**
A: You can hold/trade tokens, but you won't earn yield. Only stakers get yield.

**Q: What happens to unclaimed yield?**
A: It stays in the tax vault and accrues. Users can claim it anytime after snapshot is distributed (no time limit).

**Q: Can yield be negative?**
A: No, minimum yield is 0. If no transfers happened that week, tax collected = 0, yield = 0.

**Q: Is there a minimum staking amount?**
A: No minimum. You can stake 1 token if you want.

**Q: Can I stake/unstake multiple times per week?**
A: Yes, anytime. But only the balance staked during snapshot period counts for that week's yield.

**Q: Who triggers the weekly snapshot?**
A: Off-chain keeper (bot), or anyone manually calling `create_yield_snapshot()` after 7 days pass.

---

**Status**: Staking & yield framework implemented in Phase 1. Ready for transfer hook in Phase 2.
