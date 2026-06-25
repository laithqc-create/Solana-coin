# Devnet Deployment Guide

Complete step-by-step instructions for deploying the Ecosystem Token to Solana Devnet.

## Prerequisites

✅ Installed:
- Solana CLI 1.18+
- Rust 1.70+ (for smart contract)
- Node.js 18+
- Anchor 0.28+
- Git

✅ Configured:
- Devnet keypair: `~/.config/solana/id.json`
- Solana CLI network: `solana config set --url https://api.devnet.solana.com`

## Step 1: Verify Environment

```bash
# Check Solana CLI version
solana --version
# Expected: solana-cli 1.18.x

# Check Rust version
rustc --version
# Expected: rustc 1.75.0+

# Check Anchor version
anchor --version
# Expected: @coral-xyz/anchor/0.28.0

# Check devnet RPC
solana ping --url devnet
# Expected: Success after ~1s

# Check devnet balance
solana balance --url devnet
# Expected: If 0, request airdrop:
solana airdrop 10 --url devnet
```

## Step 2: Build Smart Contract

```bash
cd /home/claude/ecosystem-token

# Clean previous builds
rm -rf target/ Cargo.lock

# Build for devnet
cargo build --lib --release 2>&1 | tee build.log

# Verify no errors
grep -c "error" build.log
# Expected: 0 errors
```

**If build fails:**
- Check Rust version: needs `rustc 1.70+`
- Update Cargo: `cargo update`
- Clear caches: `rm -rf ~/.cargo/registry/cache target`

## Step 3: Deploy to Devnet

```bash
# Set devnet as target
solana config set --url https://api.devnet.solana.com

# Get your keypair address
solana address
# Expected: Wallet... (save this as DEPLOYER_ADDRESS)

# Deploy program
anchor deploy --provider.cluster devnet

# Expected output:
# Program deployed to: [PROGRAM_ID]
# Tx signature: [TX_SIGNATURE]

# Save PROGRAM_ID (needed for dashboard)
export PROGRAM_ID=[PROGRAM_ID_FROM_ABOVE]
```

**If deployment fails:**
- Insufficient SOL: request airdrop (`solana airdrop 10 --url devnet`)
- Network timeout: try again in 30s
- Invalid program: rebuild and check for errors

## Step 4: Generate IDL (Interface Definition Language)

```bash
# Fetch IDL from deployed program
anchor idl fetch $PROGRAM_ID -o target/idl/ecosystem_token.json

# Verify IDL file
ls -lh target/idl/ecosystem_token.json
# Expected: ~50-100KB JSON file

# Extract IDL for dashboard (optional)
cp target/idl/ecosystem_token.json ../dashboard/public/idl.json
```

## Step 5: Initialize Launchpad State

```bash
# Create token mint for ecosystem token
spl-token create-token \
  --decimals 6 \
  --url devnet

# Expected: Token mint created at [TOKEN_MINT]
export TOKEN_MINT=[TOKEN_MINT_FROM_ABOVE]

# Create USDC mock token (for testing)
spl-token create-token \
  --decimals 6 \
  --url devnet

# Expected: Mock USDC mint created at [USDC_MINT]
export USDC_MINT=[USDC_MINT_FROM_ABOVE]

# Create ATA for vault
spl-token create-account $TOKEN_MINT
# Expected: ATA created at [TOKEN_ATA]
export TOKEN_ATA=[TOKEN_ATA_FROM_ABOVE]

# Create ATA for USDC vault
spl-token create-account $USDC_MINT
# Expected: ATA created at [USDC_ATA]
export USDC_ATA=[USDC_ATA_FROM_ABOVE]
```

## Step 6: Initialize Protocol

```bash
# Create initialization script
cat > init_protocol.js << 'EOF'
const anchor = require("@project-serum/anchor");
const { PublicKey, SystemProgram } = require("@solana/web3.js");

async function initializeProtocol() {
  const provider = anchor.AnchorProvider.env();
  const program = new anchor.Program(IDL, PROGRAM_ID, provider);

  const launchpadPda = PublicKey.findProgramAddressSync(
    [Buffer.from("launchpad")],
    program.programId
  )[0];

  const yieldConfigPda = PublicKey.findProgramAddressSync(
    [Buffer.from("yield-config")],
    program.programId
  )[0];

  const vaultPda = PublicKey.findProgramAddressSync(
    [Buffer.from("vault")],
    program.programId
  )[0];

  // Initialize launchpad
  const tx = await program.methods
    .initializeLaunchpad(251, 251) // Bump values for PDAs
    .accounts({
      authority: provider.wallet.publicKey,
      launchpadState: launchpadPda,
      yieldConfig: yieldConfigPda,
      tokenMint: TOKEN_MINT,
      usdcMint: USDC_MINT,
      vault: TOKEN_ATA,
      taxVault: USDC_ATA,
      vaultPda: vaultPda,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  console.log("✓ Launchpad initialized");
  console.log("  Tx:", tx);
  console.log("  LaunchpadState PDA:", launchpadPda.toString());
  console.log("  YieldConfig PDA:", yieldConfigPda.toString());
}

initializeProtocol().catch(console.error);
EOF

# Run initialization
# Note: Requires proper IDL and environment setup
# For now, this is template - will be executed in Session 4 continuation
```

## Step 7: Update Dashboard Configuration

```bash
# Update .env.local with devnet values
cat > dashboard/.env.local << EOF
NEXT_PUBLIC_SOLANA_NETWORK=devnet
NEXT_PUBLIC_PROGRAM_ID=$PROGRAM_ID
NEXT_PUBLIC_TOKEN_MINT=$TOKEN_MINT
NEXT_PUBLIC_USDC_MINT=$USDC_MINT
EOF

# Verify values
cat dashboard/.env.local
```

## Step 8: Start Dashboard

```bash
cd dashboard

# Install dependencies (if not done)
npm install

# Start development server
npm run dev

# Expected: Server running at http://localhost:3000
```

## Step 9: Test Core Flows

### Test Tier 1 Mint
```bash
# In browser at http://localhost:3000:
1. Connect wallet
2. Go to Dashboard
3. Click "Mint" tab
4. Enter 100 USDC
5. Click "Mint Tokens"
6. Verify transaction succeeds
```

### Test Staking
```bash
1. After successful mint
2. Go to "Manage Stake"
3. Enter 1000 tokens
4. Click "Stake Tokens"
5. Verify StakingInfo PDA created
```

### Test Yield Claims
```bash
1. After 1 week of staking (or create snapshot)
2. Go to "Claim Yield"
3. Click "Claim Yield"
4. Verify USDC transferred to wallet
```

## Step 10: Verify Deployment

```bash
# Check program state
solana program show $PROGRAM_ID --url devnet
# Expected: Shows program info, upgradeable

# Check token mint
spl-token supply $TOKEN_MINT --url devnet
# Expected: Shows total supply

# Check vault balances
spl-token balance $TOKEN_ATA --url devnet
# spl-token balance $USDC_ATA --url devnet
# Expected: Shows collateral amounts
```

## Deployment Checklist

- [ ] Rust builds without errors
- [ ] Program deploys to devnet
- [ ] IDL generated successfully
- [ ] Token mints created
- [ ] ATAs created for vault
- [ ] Launchpad initialized
- [ ] Dashboard connects to program
- [ ] Tier 1 mint works
- [ ] Tier 2 mint works (after whitelist)
- [ ] Staking works
- [ ] Yield claims work

## Useful Commands

```bash
# View all accounts owned by program
solana account-info $PROGRAM_ID --url devnet

# View recent transactions
solana transaction-history $PROGRAM_ID --url devnet

# Check program logs
solana logs $PROGRAM_ID --url devnet

# Monitor real-time activity
watch 'solana balance $DEPLOYER_ADDRESS --url devnet'

# Get recent blockhash
solana blockhash --url devnet

# Estimate transaction fees
solana rent 1000 --url devnet
```

## Troubleshooting

### Build fails with "failed to download"
**Solution**: Update Rust and clear caches
```bash
rustup update stable
cargo clean
cargo build --lib --release
```

### Deployment fails with "insufficient funds"
**Solution**: Request more SOL
```bash
solana airdrop 10 --url devnet
# Wait 10s and try again
```

### IDL fetch fails
**Solution**: Ensure program deployed correctly
```bash
solana program show $PROGRAM_ID --url devnet
# If "Account doesn't exist", redeploy
```

### Dashboard can't connect to program
**Solution**: Verify environment variables
```bash
cat dashboard/.env.local
# Check PROGRAM_ID matches deployed address
```

### Transactions fail with "account not found"
**Solution**: Ensure all PDAs initialized
```bash
# Check LaunchpadState exists:
solana address -s launchpad --url devnet
```

## Next Steps (Session 4 Continuation)

1. ✅ Smart contract code generated
2. ✅ Dashboard UI generated
3. 🔄 Deploy to devnet (this guide)
4. 🔄 Initialize protocol
5. 🔄 Test all flows
6. 🔄 Run security audit
7. 🔄 Generate mainnet deployment guide

## Support

**Devnet Issues?**
- Check RPC status: https://status.solana.com
- Monitor network: https://explorer.solana.com/?cluster=devnet
- Ask in Solana Discord: https://discord.gg/solana

**Smart Contract Issues?**
- Review Anchor docs: https://book.anchor-lang.com
- Check program logs with `solana logs`

**Dashboard Issues?**
- Check browser console (F12)
- Verify wallet is connected
- Test with different wallet (Phantom vs Solflare)

---

**Status**: 🟡 Ready for Deployment (Session 4 Step 1)  
**Time Estimate**: 30-45 minutes  
**Environment**: Devnet (safe for testing)
