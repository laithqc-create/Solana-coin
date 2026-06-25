# Deployment Guide - Ecosystem Token

## Overview
This guide walks through deploying the Ecosystem Token smart contract to Solana devnet.

---

## Pre-Deployment Checklist

### Environment Setup
```bash
# 1. Install Solana CLI
curl https://release.solana.com/v1.18.0/install | sh

# 2. Install Anchor (requires Node.js 16+)
npm install -g @coral-xyz/anchor

# 3. Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 4. Update Rust to latest stable
rustup update stable

# 5. Configure Solana for devnet
solana config set --url devnet

# 6. Create devnet keypair (if needed)
solana-keygen new -o ~/.config/solana/id.json

# 7. Get devnet SOL for fees
solana airdrop 5 ~/.config/solana/id.json --url devnet
```

### Verify Setup
```bash
solana --version
# Should show: solana-cli 1.18.x

anchor --version
# Should show: @coral-xyz/anchor 0.28.x

rustc --version
# Should show: rustc 1.75.0+

cargo --version
# Should show: cargo 1.75.0+
```

---

## Step 1: Build Smart Contract

```bash
cd /home/claude/ecosystem-token

# Clean previous builds
rm -rf target/

# Build with anchor
anchor build

# Expected output:
# Compiling ecosystem_token v0.1.0
# ...
# Finished release [optimized] target(s)
# ✓ Built successfully
```

**Troubleshooting**:
- If Cargo version error: `rustup update` or use Docker
- If missing dependencies: `cargo fetch`
- If clippy warnings: Safe to ignore for now

---

## Step 2: Deploy to Devnet

```bash
cd /home/claude/ecosystem-token

# Deploy (requires ~2 SOL)
anchor deploy --provider.cluster devnet

# Output will include:
# Deploying cluster: https://api.devnet.solana.com
# Deployment status: success ✓
# Program Id: <YOUR_PROGRAM_ID>
```

**IMPORTANT**: Save the program ID! You'll need it for the dashboard.

Example output:
```
Deploying cluster: https://api.devnet.solana.com
Upgrade authority: 7Qp...abc (your keypair)
Deploying program "ecosystem_token"...
Program path: target/deploy/ecosystem_token.so...
Signature: 3xY...xyz
Program deployed: <PROGRAM_ID_HERE>
```

---

## Step 3: Generate IDL

```bash
# Generate IDL from deployed program
anchor idl fetch <PROGRAM_ID> -o target/idl/ecosystem_token.json

# Verify IDL was created
ls -la target/idl/ecosystem_token.json
```

The IDL contains all the information needed for the dashboard to interact with the contract.

---

## Step 4: Update Configuration

### Update Anchor.toml
```toml
[programs.devnet]
ecosystem_token = "<YOUR_PROGRAM_ID>"

[programs.testnet]
ecosystem_token = "<YOUR_PROGRAM_ID>"

[programs.mainnet]
ecosystem_token = "<YOUR_PROGRAM_ID>"
```

### Update Dashboard .env
```bash
cd dashboard

cp .env.example .env.local

# Edit .env.local:
NEXT_PUBLIC_PROGRAM_ID=<YOUR_PROGRAM_ID>
NEXT_PUBLIC_SOLANA_NETWORK=devnet
```

---

## Step 5: Initialize Contract on Devnet

Once deployed, you need to initialize the protocol state:

```bash
# Create USDC token on devnet (if not exists)
spl-token create-token

# Or use existing devnet USDC:
# USDC on devnet: EPjFWaJsj7aRtJSZqFbxJLUxfmYhY1phjA7kFEbYMeCt

# Initialize launchpad
# (Can be done via dashboard or CLI)
```

---

## Step 6: Start Dashboard

```bash
cd dashboard

npm install

npm run dev

# Open http://localhost:3000
```

---

## Deployment Architecture

```
┌─────────────────────────────────────────┐
│   Your Solana Keypair                   │
│   (~/.config/solana/id.json)            │
└────────────┬────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────┐
│   Smart Contract Binary                 │
│   (target/deploy/ecosystem_token.so)    │
└────────────┬────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────┐
│   Solana Devnet                         │
│   (https://api.devnet.solana.com)       │
└────────────┬────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────┐
│   Program ID (Public Address)           │
│   (stored in Anchor.toml & .env.local)  │
└────────────┬────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────┐
│   Dashboard (Next.js)                   │
│   (uses Program ID to interact)         │
└─────────────────────────────────────────┘
```

---

## Testing After Deployment

### Verify Deployment
```bash
# Check program exists
solana program show <PROGRAM_ID> --url devnet

# Get program info
solana account <PROGRAM_ID> --url devnet
```

### Test with CLI
```bash
# Load IDL
anchor idl fetch <PROGRAM_ID> -o test-idl.json

# View instructions
jq '.instructions[] | .name' test-idl.json
```

### Test with Dashboard
1. Open http://localhost:3000
2. Connect wallet (Phantom on devnet)
3. Click "Initialize" or "Mint Tokens"
4. Watch transaction go through

---

## Common Issues & Solutions

### Issue: "Insufficient funds for transaction"
**Solution**: Airdrop more SOL
```bash
solana airdrop 5 ~/.config/solana/id.json --url devnet
```

### Issue: "Program not found"
**Solution**: Verify deployment succeeded
```bash
solana program show <PROGRAM_ID> --url devnet
```

### Issue: "IDL fetch failed"
**Solution**: Wait 30 seconds for block confirmation, then retry

### Issue: "Cargo dependency conflict"
**Solution**: Use Docker build (Dockerfile provided in repo)
```bash
docker build -t ecosystem-token .
docker run -v ~/.config/solana:/root/.config/solana ecosystem-token
```

### Issue: "Invalid program ID in dashboard"
**Solution**: Make sure .env.local has correct program ID
```bash
cat .env.local | grep PROGRAM_ID
```

---

## Post-Deployment Steps

### 1. Initialize Launchpad State
```bash
# Via dashboard admin panel, or via CLI:
anchor run initialize_launchpad
```

### 2. Initialize Treasury
```bash
anchor run initialize_treasury \
  --marketing-address <WALLET> \
  --asset-manager-address <WALLET> \
  --owner-address <WALLET>
```

### 3. Create USDC Funding Account
```bash
# Create ATA for vault
spl-token create-account <USDC_MINT> <YOUR_ADDRESS>

# Fund it with devnet USDC
# (Get from faucet or swap on Orca)
```

### 4. Test Mint
```bash
# Via dashboard:
# 1. Navigate to "/"
# 2. Click "Mint Tokens"
# 3. Enter 100 USDC
# 4. Click "Mint"
```

---

## Mainnet Deployment (Future)

When ready for mainnet:

1. **Replace USDC with real token**
   - Current: Devnet USDC
   - Mainnet: EPjFWaJsj7aRtJSZqFbxJLUxfmYhY1phjA7kFEbYMeCt

2. **Update Anchor.toml**
   ```toml
   [provider]
   cluster = "mainnet-beta"
   ```

3. **Increase security**
   - Set up multisig authority
   - Enable timelock on admin functions
   - Audit by professional security firm

4. **Deploy**
   ```bash
   anchor deploy --provider.cluster mainnet-beta
   ```

---

## Support

- **Anchor Docs**: https://docs.anchor-lang.com
- **Solana Docs**: https://docs.solana.com
- **Devnet Explorer**: https://explorer.solana.com/?cluster=devnet

---

## Summary Checklist

- [ ] Rust 1.75+ installed
- [ ] Anchor CLI installed
- [ ] Solana CLI installed
- [ ] Devnet keypair created
- [ ] Devnet SOL balance > 2
- [ ] `anchor build` succeeds
- [ ] `anchor deploy` succeeds
- [ ] Program ID saved
- [ ] `.env.local` updated
- [ ] Dashboard starts (`npm run dev`)
- [ ] Wallet connects in browser
- [ ] Can see dashboard pages

---

**Status**: Ready for deployment  
**Environment**: Devnet  
**Time**: 10-15 minutes (if prerequisites met)
