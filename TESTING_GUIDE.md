# Session 4: Testing Plan & Devnet Deployment

## STEP 3: Testing - End-to-End Flows

### A. Devnet Setup

#### 1. Solana CLI Setup
```bash
# Check Solana version
solana --version  # Should be 1.17+

# Set to devnet
solana config set --url devnet

# Create keypair if needed
solana-keygen new -o ~/.config/solana/id.json

# Check balance
solana balance

# Request airdrop (up to 2 SOL per 24h)
solana airdrop 2
```

#### 2. Devnet USDC Token
```bash
# Option A: Use existing devnet USDC token
# Token address: EPjFWaKUvTqiQS5aaRrASJeFdbMqnRBcifH2 nielsen24

# Option B: Create test USDC token
spl-token create-token --decimals 6
# Save output as USDC_MINT

# Create token account
spl-token create-account <USDC_MINT>

# Mint some USDC for testing
spl-token mint <USDC_MINT> 10000000
```

#### 3. Create Ecosystem Token Mint
```bash
# Create ecosystem token mint (6 decimals)
spl-token create-token --decimals 6
# Save output as ECOSYSTEM_MINT

# Create token account for ecosystem token
spl-token create-account <ECOSYSTEM_MINT>
```

### B. Smart Contract Deployment

#### 1. Build Smart Contract
```bash
cd ecosystem-token

# Option A: Fix Rust environment
rustup update stable

# Option B: Use Docker
docker run --rm -v $(pwd):/workspace \
  projectserum/anchor:v0.28.0 \
  bash -c "cd /workspace && anchor build"

# Build command
anchor build --provider.cluster devnet
```

#### 2. Deploy to Devnet
```bash
# Deploy
anchor deploy --provider.cluster devnet

# Output will show:
# Deploying cluster: https://api.devnet.solana.com
# Upgrade authority: /path/to/keypair.json
# Deployed program: <PROGRAM_ID>

# Save PROGRAM_ID for later
```

#### 3. Generate IDL
```bash
# Fetch IDL from deployed program
anchor idl fetch <PROGRAM_ID> -o idl/ecosystem_token.json

# Output:
# IDL saved to ./idl/ecosystem_token.json
```

### C. Initialize Protocol (Admin)

#### 1. Get Account Addresses
```bash
# Derive PDA addresses
solana address -k ecosystem-token/target/deploy/ecosystem_token-keypair.json

# Launchpad state PDA:
# seeds: [b"launchpad"]
# bump: (derive with anchor)

# Vault PDA:
# seeds: [b"vault"]
# bump: (derive with anchor)
```

#### 2. Create Initialization Script
Create `scripts/init.ts`:

```typescript
import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import IDL from "../idl/ecosystem_token.json";

const PROGRAM_ID = new PublicKey("<PROGRAM_ID>");
const USDC_MINT = new PublicKey("<USDC_MINT>");
const ECOSYSTEM_MINT = new PublicKey("<ECOSYSTEM_MINT>");

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = new anchor.Program(IDL as any, PROGRAM_ID, provider);

  // 1. Initialize Launchpad
  const launchpadTx = await program.methods
    .initializeLaunchpad(255, 255) // token_bump, vault_bump
    .accounts({
      launchpadState: launchpadPDA,
      yieldConfig: yieldConfigPDA,
      tokenMint: ECOSYSTEM_MINT,
      usdcMint: USDC_MINT,
      vault: vaultATA,
      taxVault: taxVaultATA,
      vaultPda: vaultPDA,
      authority: provider.wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log("✓ Launchpad initialized:", launchpadTx);

  // 2. Initialize Treasury
  const treasuryTx = await program.methods
    .initializeTreasury(
      new PublicKey("<MARKETING_ADDRESS>"),
      new PublicKey("<MANAGER_ADDRESS>"),
      new PublicKey("<OWNER_ADDRESS>")
    )
    .accounts({
      treasuryVault: treasuryPDA,
      revenueDistribution: revenuePDA,
      authority: provider.wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log("✓ Treasury initialized:", treasuryTx);
}

main().catch(console.error);
```

#### 3. Run Initialization
```bash
ts-node scripts/init.ts
```

### D. Test User Flows

#### Test 1: Tier 1 Minting
```
User Flow:
1. Connect wallet (Phantom)
2. Approve USDC spend (500 USDC)
3. Click "Mint Tokens" → 500 USDC
4. Expected: Receive 500 tokens at 1:1 ratio
5. Verify: Token balance increases

Expected Results:
- USDC transferred to vault ✓
- Tokens minted to user ✓
- UserTierInfo PDA created ✓
- Transaction signature visible ✓
```

**Test Case Implementation**:
```typescript
describe("Tier 1 Minting", () => {
  it("should mint tokens at 1:1 ratio", async () => {
    const usdcAmount = new anchor.BN(500_000_000); // 500 USDC (6 decimals)
    
    const tx = await program.methods
      .mintTokens(usdcAmount, false) // isTier2 = false
      .accounts({
        user: userPublicKey,
        launchpadState: launchpadPDA,
        tokenMint: ECOSYSTEM_MINT,
        userUsdcAta: userUsdcATA,
        userTokenAta: userTokenATA,
        vault: vaultATA,
        vaultPda: vaultPDA,
        userTierInfo: userTierPDA,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([userKeypair])
      .rpc();

    // Verify balances
    const userTokenBalance = await getTokenBalance(userTokenATA);
    assert.equal(userTokenBalance, 500_000_000); // 500 tokens
    
    // Verify PDA state
    const tierInfo = await program.account.userTierInfo.fetch(userTierPDA);
    assert.equal(tierInfo.isTier2, false);
    assert.equal(tierInfo.totalTokensMinted.toNumber(), 500_000_000);
  });
});
```

#### Test 2: Tier 2 Minting (Whitelist + Vesting)
```
User Flow:
1. Admin adds user to whitelist
2. User mints 1000 USDC as Tier 2
3. Receives 1400 tokens (40% discount)
4. Tokens locked for 12 months
5. Verify vesting schedule created

Expected Results:
- Discount applied (1400 = 1000 * 1.40) ✓
- VestingSchedule PDA created ✓
- Tokens locked: vested_amount = 0 ✓
- After 6 months: vested_amount ≈ 700 ✓
- After 12 months: vested_amount = 1400 ✓
```

**Test Case**:
```typescript
describe("Tier 2 Minting with Vesting", () => {
  it("should mint with 40% discount and lock tokens", async () => {
    // First: whitelist user
    await program.methods
      .setTier2Whitelist(true)
      .accounts({
        authority: adminPublicKey,
        targetUser: userPublicKey,
        tier2Whitelist: whitelist PDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([adminKeypair])
      .rpc();

    // Then: mint as Tier 2
    const usdcAmount = new anchor.BN(1_000_000_000); // 1000 USDC
    
    await program.methods
      .mintTokens(usdcAmount, true) // isTier2 = true
      .accounts({
        user: userPublicKey,
        userUsdcAta: userUsdcATA,
        userTokenAta: userTokenATA,
        // ... other accounts
      })
      .signers([userKeypair])
      .rpc();

    // Verify vesting
    const tierInfo = await program.account.userTierInfo.fetch(userTierPDA);
    assert.equal(tierInfo.isTier2, true);
    
    const vesting = tierInfo.vestingSchedule;
    assert.isNotNull(vesting);
    assert.equal(vesting.totalAmount.toNumber(), 1_400_000_000); // 1400 tokens
    
    // Verify vesting math
    const now = Math.floor(Date.now() / 1000);
    const vestedAmount = vesting.vestedAmount(now);
    assert.equal(vestedAmount, 0); // Not vested yet
  });
});
```

#### Test 3: Staking & Yield
```
User Flow:
1. User has 500 tokens
2. Stakes 200 tokens
3. Receives yield snapshot (weekly)
4. Claims pro-rata USDC yield
5. Unstakes 100 tokens

Expected Results:
- StakingInfo created ✓
- Staked amount = 200 ✓
- Can unstake up to 200 ✓
- YieldSnapshot created weekly ✓
- Pro-rata share calculated ✓
- USDC transferred to user ✓
```

**Test Case**:
```typescript
describe("Staking and Yield", () => {
  it("should stake, snapshot, and claim yield", async () => {
    // 1. Stake 200 tokens
    const stakeAmount = new anchor.BN(200_000_000);
    
    await program.methods
      .stakeTokens(stakeAmount)
      .accounts({
        user: userPublicKey,
        userTokenAta: userTokenATA,
        stakingVault: stakingVaultATA,
        userTierInfo: userTierPDA,
        stakingInfo: stakingPDA,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([userKeypair])
      .rpc();

    // Verify stake
    let stakingInfo = await program.account.stakingInfo.fetch(stakingPDA);
    assert.equal(stakingInfo.stakedAmount.toNumber(), 200_000_000);

    // 2. Create yield snapshot (simulate 1 week)
    await program.methods
      .createYieldSnapshot()
      .accounts({
        authority: adminPublicKey,
        yieldConfig: yieldConfigPDA,
        taxVault: taxVaultATA,
        yieldSnapshot: snapshotPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([adminKeypair])
      .rpc();

    // 3. Claim yield
    const claimTx = await program.methods
      .claimYield()
      .accounts({
        user: userPublicKey,
        userUsdcAta: userUsdcATA,
        taxVault: taxVaultATA,
        stakingInfo: stakingPDA,
        yieldSnapshot: snapshotPDA,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([userKeypair])
      .rpc();

    // Verify yield claimed
    stakingInfo = await program.account.stakingInfo.fetch(stakingPDA);
    assert.isAbove(stakingInfo.totalYieldClaimed.toNumber(), 0);

    // 4. Unstake partial
    await program.methods
      .unstakeTokens(new anchor.BN(100_000_000))
      .accounts({
        user: userPublicKey,
        userTokenAta: userTokenATA,
        stakingVault: stakingVaultATA,
        stakingInfo: stakingPDA,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([userKeypair])
      .rpc();

    stakingInfo = await program.account.stakingInfo.fetch(stakingPDA);
    assert.equal(stakingInfo.stakedAmount.toNumber(), 100_000_000);
  });
});
```

#### Test 4: Transfer with Tax
```
User Flow:
1. User transfers 100 tokens to recipient
2. 0.1% tax deducted (0.1 tokens)
3. Tax split: 70% treasury (0.07), 30% yield (0.03)
4. Recipient receives 99.9 tokens

Expected Results:
- Net transfer = 99.9 ✓
- Treasury vault += 0.07 ✓
- Yield vault += 0.03 ✓
- Vesting enforced (Tier 2) ✓
```

**Test Case**:
```typescript
describe("Transfer with Tax", () => {
  it("should apply 0.1% tax and split 70/30", async () => {
    const transferAmount = new anchor.BN(100_000_000); // 100 tokens
    const expectedTax = 100_000; // 0.1% = 100k
    const expectedTreasuryTax = 70_000; // 70%
    const expectedYieldTax = 30_000; // 30%

    const balanceBefore = await getTokenBalance(treasuryVaultATA);

    await program.methods
      .transferWithTax(transferAmount)
      .accounts({
        user: userPublicKey,
        from: userTokenATA,
        to: recipientTokenATA,
        treasuryVaultAta: treasuryVaultATA,
        yieldVaultAta: yieldVaultATA,
        userTierInfo: userTierPDA,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([userKeypair])
      .rpc();

    // Verify recipient received net amount
    const recipientBalance = await getTokenBalance(recipientTokenATA);
    assert.equal(
      recipientBalance,
      99_900_000 // 100 - 0.1% tax
    );

    // Verify treasury tax
    const treasuryBalance = await getTokenBalance(treasuryVaultATA);
    assert.equal(
      treasuryBalance.toNumber() - balanceBefore.toNumber(),
      expectedTreasuryTax
    );
  });
});
```

#### Test 5: Admin Whitelist
```
Admin Flow:
1. Admin calls set_tier2_whitelist(user, true)
2. User whitelisted for Tier 2
3. Admin calls set_tier2_whitelist(user, false)
4. User removed from whitelist

Expected Results:
- Tier2Whitelist PDA created ✓
- is_whitelisted = true ✓
- Can be toggled ✓
- Admin-only access enforced ✓
```

### E. Dashboard Testing

#### 1. UI Component Tests
```bash
# Start dev server
cd dashboard
npm run dev

# Test checklist:
- [ ] Wallet connect button visible
- [ ] All 3 wallets selectable (Phantom, Solflare, Torus)
- [ ] Connected state shows wallet address
- [ ] Navigation links work (/, /treasury, /analytics, /admin)
- [ ] Charts render with mock data
- [ ] Forms accept input
- [ ] Responsive on mobile (375px width)
- [ ] Dark theme applied throughout
- [ ] No console errors
```

#### 2. Integration Tests
```typescript
// Test useEcosystemToken hook
describe("useEcosystemToken Hook", () => {
  it("should fetch user state on mount", async () => {
    const { result } = renderHook(() => useEcosystemToken());
    
    await waitFor(() => {
      expect(result.current.userState.isLoading).toBe(false);
    });

    expect(result.current.userState.stakedAmount).toBeGreaterThanOrEqual(0);
    expect(result.current.userState.totalTokensMinted).toBeGreaterThanOrEqual(0);
  });

  it("should handle mint transaction", async () => {
    const { result } = renderHook(() => useEcosystemToken());

    act(() => {
      result.current.mintTokens(500, false);
    });

    await waitFor(() => {
      expect(result.current.txState.isProcessing).toBe(false);
    });

    expect(result.current.txState.txSignature).toBeDefined();
  });
});
```

### F. Security Tests

#### 1. Vesting Enforcement
```
Test: Cannot transfer locked tokens

Input: User with 1000 vesting tokens, tries to transfer 500
Expected: Transaction fails with CannotTransferLocked error
Actual Result: ✓ / ✗
```

#### 2. Whitelist Enforcement
```
Test: Non-whitelisted user cannot mint Tier 2

Input: Non-whitelisted user tries to mint 1000 USDC as Tier 2
Expected: Transaction fails with NotWhitelisted error
Actual Result: ✓ / ✗
```

#### 3. Math Overflow
```
Test: Cannot exceed supply cap

Input: Mint until 101M tokens (> 100M cap)
Expected: Transaction fails with SupplyCapExceeded error
Actual Result: ✓ / ✗
```

#### 4. Tax Accuracy
```
Test: Tax calculation doesn't cause precision loss

Input: Transfer 12345 tokens
Tax: 12345 * 0.001 = 12.345 tokens
Expected: Math rounded, no remainder lost
Actual Result: ✓ / ✗
```

### G. Performance Baseline

#### Load Times
```
Page Load Times (Devnet):
- User Dashboard: target < 2s ___ms
- Treasury Dashboard: target < 2s ___ms
- Analytics Dashboard: target < 3s ___ms
- Admin Dashboard: target < 2s ___ms

Chart Render Times:
- Staking Trend: target < 500ms ___ms
- Aave Position: target < 500ms ___ms
- Leaderboard: target < 300ms ___ms
```

#### Transaction Confirmation
```
Devnet Confirmation Times:
- Single instruction: target < 10s ___s
- CPI to token program: target < 15s ___s
- Full staking flow: target < 20s ___s
```

### H. Test Results Summary

Create `TEST_RESULTS.md`:

```markdown
# Session 4 Testing Results

| Test | Expected | Actual | Status |
|------|----------|--------|--------|
| Tier 1 Mint | 500 → 500 tokens | | ✓/✗ |
| Tier 2 Mint + Vesting | 1000 → 1400 tokens locked | | ✓/✗ |
| Staking | 200 tokens staked | | ✓/✗ |
| Yield Claim | Pro-rata USDC | | ✓/✗ |
| Transfer Tax | 100 → 99.9, tax split | | ✓/✗ |
| Whitelist | User whitelisted | | ✓/✗ |
| Vesting Lock | Cannot transfer locked | | ✓/✗ |
| Admin Controls | Pause/resume works | | ✓/✗ |

## Performance
- Page Load: __ms
- Chart Render: __ms
- Tx Confirmation: __s

## Issues Found
1. ...
2. ...

## Ready for Audit: ✓ / ✗
```

---

## Next: STEP 4 - Security Audit

Once all tests pass, proceed to code review and security audit.
