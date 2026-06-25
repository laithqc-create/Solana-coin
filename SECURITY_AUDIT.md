# Session 4: Security Audit Checklist

## STEP 4: Code Audit & Security Review

### A. Smart Contract Security

#### A1. Account Validation
- [ ] LaunchpadState PDA validates seeds correctly
- [ ] Vault PDA is canonical (only one per program)
- [ ] All accounts marked with `#[account]` derive constraints
- [ ] Owner validation on all mutable accounts
- [ ] Signer validation on sensitive operations

**Review Code**:
```rust
// ✓ GOOD: Proper PDA validation
#[account(
    mut,
    seeds = [b"launchpad"],
    bump = launchpad_state.bump,
    constraint = launchpad_state.mint_authority == vault_pda.key()
)]
pub launchpad_state: Account<'info, LaunchpadState>,

// ✗ BAD: Missing bump validation
#[account(mut)]
pub launchpad_state: Account<'info, LaunchpadState>,
```

#### A2. Math & Overflow
- [ ] All arithmetic uses checked operations
- [ ] No silent truncation in divisions
- [ ] Conversion from u64 to u128 before multiplication
- [ ] Overflow protection on all BN operations

**Verify**:
```rust
// ✓ Safe math
let tax = amount
    .checked_mul(1)
    .ok_or(MathOverflow)?
    .checked_div(1000)
    .ok_or(MathOverflow)?;

// ✗ Unsafe
let tax = (amount * 1) / 1000; // Can overflow
```

#### A3. Vesting Enforcement
- [ ] Tier 2 tokens locked until end_time
- [ ] vesting.vested_amount() linear interpolation correct
- [ ] Cannot bypass vesting with transfer_with_tax
- [ ] Vesting progress cannot go backward
- [ ] Vesting math handles edge cases (now < start, now > end)

**Test Cases**:
```
Time = start - 1 day → vested = 0 ✓
Time = start → vested = 0 ✓
Time = start + 182.5 days → vested ≈ 50% ✓
Time = end → vested = 100% ✓
Time = end + 1 year → vested = 100% ✓
```

#### A4. Transfer Hook Tax
- [ ] Tax = 0.1% (amount / 1000)
- [ ] Treasury tax = 70% of tax ✓
- [ ] Yield tax = 30% of tax ✓
- [ ] No rounding loss (0.07 + 0.03 = 0.10 exactly)
- [ ] Tier 2 vesting enforced before tax calculation

**Verify**:
```rust
// 0.1% tax calculation
pub fn calculate_tax(amount: u64) -> Result<(u64, u64, u64)> {
    let tax = amount
        .checked_mul(1)
        .ok_or(MathOverflow)?
        .checked_div(1000)
        .ok_or(MathOverflow)?;

    let treasury = tax
        .checked_mul(70)
        .ok_or(MathOverflow)?
        .checked_div(100)
        .ok_or(MathOverflow)?;

    let yield_tax = tax.checked_sub(treasury).ok_or(MathOverflow)?;

    assert_eq!(treasury + yield_tax, tax); // No loss
    Ok((tax, treasury, yield_tax))
}
```

#### A5. Supply Cap
- [ ] Total supply capped at 100M tokens
- [ ] Check before minting: new_total <= 100M
- [ ] Cannot mint past cap via any path
- [ ] Cap includes all tiers and multipliers

**Verify**:
```rust
let new_total = launchpad
    .total_tokens_minted
    .checked_add(tokens_to_mint)
    .ok_or(MathOverflow)?;

require!(
    new_total <= 100_000_000_000_000, // 100M * 1e6
    SupplyCapExceeded
);
```

#### A6. Whitelist Enforcement
- [ ] Tier 2Whitelist PDA required for Tier 2 minting
- [ ] Whitelist can be toggled (remove + re-add)
- [ ] Only authority can modify whitelist
- [ ] Non-whitelisted users get NotWhitelisted error

**Test**:
```
Non-whitelisted user → mint Tier 2 → NotWhitelisted ✓
Admin whitelist user → mint Tier 2 → Success ✓
Admin remove whitelist → mint Tier 2 → NotWhitelisted ✓
```

#### A7. Yield Distribution
- [ ] Pro-rata calculation correct: (user_stake / total_stake) * yield
- [ ] No precision loss in division
- [ ] YieldSnapshot created weekly (604,800 seconds)
- [ ] Multiple claims don't double-pay
- [ ] Only distributes available yield

**Example**:
```
Total staked: 1000 tokens
User staked: 200 tokens
User share: 200/1000 = 20%

Tax collected: 500 USDC
User yield: 500 * 20% = 100 USDC ✓
```

#### A8. Admin Controls
- [ ] Pause/resume requires authority signature
- [ ] Paused state actually prevents minting
- [ ] Paused state doesn't prevent staking
- [ ] Emergency procedures require multisig
- [ ] Allocation percentages sum to 100

**Verify**:
```rust
require!(!ctx.accounts.launchpad_state.paused, LaunchpadPaused);

pub fn update_allocation_percentages(
    user: u8, marketing: u8, manager: u8, owner: u8
) -> Result<()> {
    require!(
        (user + marketing + manager + owner) == 100,
        InvalidDistributionPercentages
    );
    Ok(())
}
```

#### A9. CPI Safety
- [ ] All token transfers via CPI
- [ ] No hardcoded program IDs (use constants)
- [ ] AssociatedTokenAccount validated
- [ ] SPL Token Program ID verified
- [ ] No unsafe CPI calls

**Verify**:
```rust
// ✓ Safe CPI
let transfer_ctx = CpiContext::new(
    ctx.accounts.token_program.to_account_info(),
    Transfer {
        from: ctx.accounts.user_token_ata.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    },
);
transfer(transfer_ctx, amount)?;

// ✗ Unsafe
invoke(
    &Token::transfer(...).unwrap(),
    &[], // Missing signers
)?;
```

---

### B. Frontend Security

#### B1. Input Validation
- [ ] Amount inputs > 0
- [ ] Wallet address validation (58 chars, valid base58)
- [ ] No XSS via unescaped user input
- [ ] Rate limiting on transaction submission
- [ ] Maximum reasonable amounts (e.g., > 1B tokens fails)

**Verify**:
```typescript
// ✓ Safe input validation
const validateAmount = (amount: string): boolean => {
  const num = parseFloat(amount);
  return !isNaN(num) && num > 0 && num < 1_000_000_000; // Max 1B
};

// ✗ Unsafe
const handleMint = (amount) => {
  program.methods.mintTokens(amount); // No validation
};
```

#### B2. Private Key Security
- [ ] No private keys stored in localStorage
- [ ] No private keys logged to console
- [ ] Wallet connects via WalletAdapter only
- [ ] All transactions signed by wallet extension
- [ ] Environment variables never exposed to frontend

**Verify**:
```typescript
// ✓ Safe: Use wallet signer
const tx = await program.methods
  .mintTokens(amount, isTier2)
  .signers([]) // Wallet signs
  .rpc();

// ✗ Unsafe: Never do this
const wallet = new Keypair.fromSecret(Buffer.from(SECRET));
```

#### B3. CORS & RPC
- [ ] RPC endpoint is public/official
- [ ] No custom RPC proxies (avoids MITM)
- [ ] Environment-specific endpoints (devnet, mainnet-beta)
- [ ] Fallback RPC if primary fails
- [ ] Rate limit aware

**Config**:
```typescript
// ✓ Good
const endpoints = {
  devnet: "https://api.devnet.solana.com",
  mainnet: "https://api.mainnet-beta.solana.com",
};

// ✗ Risky
const endpoint = "http://private-rpc.example.com"; // Unknown
```

#### B4. Transaction Signing
- [ ] All transactions shown to user before signing
- [ ] Clear description of what's being signed
- [ ] No silent/background transactions
- [ ] Reasonable gas/instruction limits
- [ ] Timeout on pending transactions (5 min)

**Verify**:
```typescript
// ✓ Clear intent
const tx = new Transaction()
  .add(
    program.instruction.mintTokens(amount, isTier2)
  );

// Show to user before:
const { blockhash } = await connection.getLatestBlockhash();
tx.recentBlockhash = blockhash;
tx.feePayer = publicKey;

const signed = await signTransaction(tx);
const txSig = await sendTransaction(signed, connection);
```

#### B5. Error Messages
- [ ] User-friendly error text
- [ ] No sensitive data in errors (account keys, amounts)
- [ ] Helpful troubleshooting suggestions
- [ ] Distinguish user errors from system errors

**Examples**:
```typescript
// ✓ User-friendly
throw new Error("Insufficient USDC balance. Request more from faucet.");

// ✗ Confusing
throw new Error("0x1234567... account constraint: owner mismatch");
```

---

### C. Deployment Security

#### C1. Devnet Safety
- [ ] Use only devnet during testing
- [ ] Devnet keypair ≠ mainnet keypair
- [ ] Test tokens/SOL only on devnet
- [ ] No real user data on devnet

**Check**:
```bash
# Verify devnet cluster
solana config get
# Output should show: RPC URL: https://api.devnet.solana.com
```

#### C2. Program Upgrade Authority
- [ ] Upgrade authority is multisig (not single keypair)
- [ ] Upgrade procedure documented
- [ ] No accidental upgrades possible
- [ ] Old program versions archived

**Multisig Setup**:
```bash
# Create 2-of-3 multisig
anchor multisig create-multisig \
  --signers <key1> <key2> <key3> \
  --threshold 2

# Result: Multisig Authority = <MULTISIG_ADDRESS>
```

#### C3. Mint Authority
- [ ] Hardcoded to vault PDA (no human control)
- [ ] Cannot change mint authority post-deployment
- [ ] Only mint instruction can create tokens
- [ ] No emergency mint capability

**Verify**:
```bash
# Check mint authority
spl-token display <TOKEN_MINT>
# Should show: Mint authority: <VAULT_PDA>
```

#### C4. Treasury Security
- [ ] Treasury keys are multisig
- [ ] Aave deposits require multisig approval
- [ ] Distribution requires multisig approval
- [ ] Regular audits of treasury balance
- [ ] Withdrawal limits (e.g., max 10% per week)

**Emergency Procedures**:
- Loss of keys → pause launchpad
- Aave hack → withdraw via multisig
- Smart contract bug → upgrade via multisig

---

### D. Testing Security

#### D1. Invariant Tests
- [ ] Total tokens minted ≥ total supply
- [ ] Tax collected ≤ total transfers
- [ ] Vesting end_time ≥ start_time
- [ ] Supply never exceeds 100M

#### D2. Edge Cases
- [ ] Transfer amount = 1 (minimum)
- [ ] Transfer amount = u64::MAX (should fail)
- [ ] Vesting time = 0 (should work)
- [ ] Vesting time = u64::MAX (should work)
- [ ] Zero stakers claiming yield (no division by zero)

#### D3. Fuzz Testing
```bash
# Random inputs to find edge cases
cargo fuzz --crate ecosystem_token 100000
```

---

### E. Audit Sign-Off

#### Security Review Checklist
- [ ] All accounts properly validated
- [ ] Math is safe (no overflow, precision loss)
- [ ] Vesting correctly enforced
- [ ] Whitelist prevents unauthorized Tier 2 minting
- [ ] Transfer tax correctly split
- [ ] Supply cap enforced
- [ ] Admin controls work as designed
- [ ] CPI is safe
- [ ] Frontens properly validates inputs
- [ ] No private keys exposed
- [ ] Deployment is secure (multisig, vault PDA)

#### Code Review Completed By
```
Reviewer: ________________
Date: ________________
Audit Status: ✓ PASSED / ✗ ISSUES FOUND

Issues Found:
1. ...
2. ...

Recommendations:
1. ...
2. ...

Approved for Devnet Testing: ✓ YES / ✗ NO
Approved for Mainnet: ✓ YES / ✗ NO (after further review)
```

#### Critical Issues Template
```markdown
## Issue #1: [Title]

**Severity**: CRITICAL / HIGH / MEDIUM / LOW

**Description**: 
[Detailed explanation]

**Location**: 
ecosystem-token/programs/ecosystem-token/src/[file].rs::[function]

**Proof of Concept**:
[Code demonstrating the issue]

**Fix**:
[Recommended fix]

**Status**: OPEN / IN REVIEW / FIXED / CLOSED
```

---

## Next: STEP 5 - Mainnet Readiness

Once audit is complete and all issues resolved, proceed to mainnet documentation.
