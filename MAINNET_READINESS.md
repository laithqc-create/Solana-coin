# Session 4: Mainnet Readiness & Deployment

## STEP 5: Production Preparation

### A. Pre-Mainnet Checklist

#### Devnet Complete
- [ ] All 17 instructions deployed and tested
- [ ] All 4 dashboard pages working
- [ ] End-to-end flows validated
- [ ] Security audit passed (0 critical issues)
- [ ] Performance benchmarks met
- [ ] Documentation complete

#### Code Finalized
- [ ] No console.logs in production code
- [ ] All error messages user-friendly
- [ ] No hardcoded testnet addresses
- [ ] All features enabled (no feature gates)
- [ ] Build passes without warnings

**Pre-deploy checks**:
```bash
# Smart contract
cd ecosystem-token
cargo clippy -- -D warnings  # No warnings
anchor build --release

# Dashboard
cd dashboard
npm run build  # No errors
npm run lint  # All checks pass
```

#### Audit Complete
- [ ] External audit by third party (recommended)
- [ ] Internal audit by 2+ reviewers
- [ ] All issues resolved
- [ ] Audit report published
- [ ] No open security issues

---

### B. Mainnet Configuration

#### B1. Token Creation

```bash
# Create Ecosystem Token on mainnet
solana config set --url mainnet-beta

# Create token mint (6 decimals)
spl-token create-token --decimals 6 --mint-authority <MULTISIG_ADDRESS>

# Example output:
# Creating token XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
# Mint authority: Multisig2222...
```

**Save Configuration**:
```json
{
  "ECOSYSTEM_TOKEN_MINT": "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  "ECOSYSTEM_MINT_AUTHORITY": "Multisig2222...",
  "DECIMALS": 6,
  "NETWORK": "mainnet-beta"
}
```

#### B2. USDC Integration

```bash
# Use existing Solana USDC mainnet token
# Mainnet USDC: EPjFWaKUvTqiQS5aaRrASJeFdbMqnRBcifH2

# Mainnet details:
{
  "USDC_MINT": "EPjFWaKUvTqiQS5aaRrASJeFdbMqnRBcifH2",
  "USDC_DECIMALS": 6,
  "ISSUER": "Circle",
  "BRIDGE": "Wormhole",
  "TOTAL_SUPPLY": "100,000,000,000 USDC"
}
```

#### B3. Multisig Setup

**2-of-3 Multisig Authority**:
```bash
# Signers
SIGNER_1="<CEO_PUBKEY>"
SIGNER_2="<CTO_PUBKEY>"
SIGNER_3="<TREASURY_PUBKEY>"

# Create multisig
anchor multisig create-multisig \
  --signers $SIGNER_1 $SIGNER_2 $SIGNER_3 \
  --threshold 2

# Output:
# Multisig Authority: Multisig5555...
# Required signatures: 2 of 3
```

**Custody**:
- Signer 1: CEO (hot wallet, Ledger)
- Signer 2: CTO (cold storage, Ledger)
- Signer 3: Treasury Manager (multisig contract)

---

### C. Mainnet Deployment

#### C1. Smart Contract Deployment

```bash
cd ecosystem-token

# 1. Update Anchor.toml
cat > Anchor.toml << 'EOF'
[programs.mainnet]
ecosystem_token = "YOUR_PROGRAM_ID"

[provider]
cluster = "mainnet-beta"
wallet = "~/.config/solana/id.json"
EOF

# 2. Build for mainnet
anchor build --provider.cluster mainnet-beta

# 3. Deploy
# Cost: ~2 SOL (~$100 at current prices)
anchor deploy --provider.cluster mainnet-beta

# Output:
# Deploying cluster: https://api.mainnet-beta.solana.com
# Program: YOUR_PROGRAM_ID
# Deployment Status: ✓ Success
```

#### C2. IDL Publishing

```bash
# Fetch and publish IDL
anchor idl fetch YOUR_PROGRAM_ID -o idl/ecosystem_token.json

# Store in repository
git add idl/ecosystem_token.json
git commit -m "Add mainnet IDL"
git push origin main
```

#### C3. Initialization

**Create `scripts/mainnet-init.ts`**:

```typescript
import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import IDL from "../idl/ecosystem_token.json";

const PROGRAM_ID = new PublicKey("YOUR_PROGRAM_ID");
const USDC_MINT = new PublicKey("EPjFWaKUvTqiQS5aaRrASJeFdbMqnRBcifH2");
const ECOSYSTEM_MINT = new PublicKey("YOUR_ECOSYSTEM_MINT");
const MULTISIG_AUTHORITY = new PublicKey("Multisig5555...");

// Treasury addresses
const MARKETING_ADDRESS = new PublicKey("...");
const MANAGER_ADDRESS = new PublicKey("...");
const OWNER_ADDRESS = new PublicKey("...");

async function initializeMainnet() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = new anchor.Program(IDL as any, PROGRAM_ID, provider);

  console.log("🚀 Initializing mainnet...");

  // 1. Initialize Launchpad
  console.log("📋 Setting up Launchpad...");
  const launchpadTx = await program.methods
    .initializeLaunchpad(255, 255)
    .accounts({
      authority: MULTISIG_AUTHORITY,
      // ... other accounts
    })
    .rpc();
  console.log("✓ Launchpad initialized:", launchpadTx);

  // 2. Initialize Treasury
  console.log("💼 Setting up Treasury...");
  const treasuryTx = await program.methods
    .initializeTreasury(
      MARKETING_ADDRESS,
      MANAGER_ADDRESS,
      OWNER_ADDRESS
    )
    .accounts({
      authority: MULTISIG_AUTHORITY,
      // ... other accounts
    })
    .rpc();
  console.log("✓ Treasury initialized:", treasuryTx);

  // 3. Verify state
  console.log("\n✅ Mainnet initialization complete!");
  console.log(`Program ID: ${PROGRAM_ID.toBase58()}`);
  console.log(`Authority: ${MULTISIG_AUTHORITY.toBase58()}`);
  console.log(`Ecosystem Token: ${ECOSYSTEM_MINT.toBase58()}`);
}

initializeMainnet().catch(console.error);
```

**Execute initialization**:
```bash
# Requires 2-of-3 multisig approval
npx ts-node scripts/mainnet-init.ts
```

---

### D. Dashboard Deployment

#### D1. Production Build

```bash
cd dashboard

# 1. Update environment
cat > .env.production << 'EOF'
NEXT_PUBLIC_SOLANA_NETWORK=mainnet-beta
NEXT_PUBLIC_PROGRAM_ID=YOUR_PROGRAM_ID
NEXT_PUBLIC_RPC_ENDPOINT=https://api.mainnet-beta.solana.com
EOF

# 2. Build
npm run build

# 3. Verify build
npm run start  # Test production build locally
```

#### D2. Deployment Options

**Option A: Vercel (Recommended)**
```bash
# 1. Push to GitHub
git push origin main

# 2. Connect to Vercel
# https://vercel.com/new
# Select ecosystem-token repository
# Auto-deploy on push

# 3. Set environment variables in Vercel dashboard
# NEXT_PUBLIC_SOLANA_NETWORK=mainnet-beta
# NEXT_PUBLIC_PROGRAM_ID=YOUR_PROGRAM_ID
```

**Option B: Self-Hosted (AWS/GCP)**
```bash
# 1. Build Docker image
docker build -t ecosystem-token-dashboard:latest .

# 2. Push to registry
docker push your-registry/ecosystem-token-dashboard:latest

# 3. Deploy to Kubernetes/EC2
# Update deployment manifests with new image
# Apply: kubectl apply -f k8s/deployment.yaml
```

**Option C: AWS S3 + CloudFront**
```bash
# 1. Build Next.js static export
npm run export

# 2. Sync to S3
aws s3 sync out/ s3://ecosystem-token-dashboard/ --delete

# 3. Invalidate CloudFront cache
aws cloudfront create-invalidation \
  --distribution-id E123456 \
  --paths "/*"
```

#### D3. DNS Configuration

```bash
# Point domain to deployment
app.ecosystem-token.io  → Vercel / CloudFront / Load Balancer

# Example (Route 53):
Name: app.ecosystem-token.io
Type: A
Value: 76.75.26.X (Vercel IP)
TTL: 300
```

---

### E. Launch Sequence

#### Phase 1: Soft Launch (48 hours)
```
Timeline:
Day 1 08:00 UTC: Deploy smart contract
Day 1 10:00 UTC: Whitelist seed investors (50 addresses)
Day 1 12:00 UTC: Soft launch → max 5 USDC mints to test
Day 2 08:00 UTC: Increase limit → max 500 USDC
Day 2 20:00 UTC: Monitor for bugs
Day 3 00:00 UTC: Full launch if no issues
```

**Soft Launch Checklist**:
- [ ] Smart contract deployed
- [ ] Dashboard accessible
- [ ] Wallet connections working (all 3)
- [ ] Mint function works with small amounts
- [ ] Staking operational
- [ ] Yield claims work
- [ ] No critical errors in logs
- [ ] Transaction confirmations < 30s

#### Phase 2: Full Launch
```
Day 3 00:00 UTC: Remove mint limits
Day 3 00:00 UTC: Announce via Twitter/Discord
Day 3 06:00 UTC: Monitor 6 hours
Day 3 12:00 UTC: All-clear signal given
```

**Monitoring**:
- TVL (total value locked)
- Transaction volume
- Unique users
- Error rates
- RPC latency
- Smart contract events

---

### F. Operational Procedures

#### F1. Daily Operations

```bash
# Morning check (08:00 UTC)
1. Check TVL on-chain
   solana balance --owner <VAULT_PDA>

2. Verify smart contract state
   anchor idl fetch <PROGRAM_ID> | jq .

3. Check dashboard metrics
   curl https://app.ecosystem-token.io/api/health

4. Review error logs
   tail -100 /var/log/app.log | grep ERROR

5. Monitor transaction fees
   Total fees this week: ____ SOL
```

#### F2. Weekly Treasury Management

```bash
# Thursday 10:00 UTC: Treasury snapshot
1. Calculate total Aave position
2. Calculate accrued yield
3. Proposal for yield distribution
4. Multisig voting (2 of 3 required)

# Friday 14:00 UTC: Execute distributions
1. Claim Aave yields
2. Split revenue (40/20/20/20)
3. Transfer to recipients
4. Log transaction hashes
```

#### F3. Emergency Procedures

**If critical bug found**:
```
1. Pause launchpad immediately
   - Multisig vote (2 of 3)
   - Prevents new mints
   - Allows claims/unstaking

2. Announce on Twitter
   - "Maintenance mode engaged"
   - Estimated duration
   - User guidance

3. Investigate root cause
   - Code review
   - On-chain state inspection
   - Devise fix

4. Deploy fix
   - Upload patched code
   - Multisig upgrade vote
   - Deploy
   - Resume

5. Post-mortem
   - Public explanation
   - Preventive measures
   - Timeline to resume full operations
```

**If Aave is exploited**:
```
1. Pause Aave deposits immediately
2. Withdraw all USDC (multisig vote)
3. Investigate Aave hack
4. Decide: Resume with Aave or use alternative
```

---

### G. Documentation for Users

#### G1. Getting Started Guide

**Website Section: `/learn`**
```markdown
# Ecosystem Token Guide

## 1. What is Ecosystem Token?
- Collateral-backed token (backed 1:1 by USDC)
- Stake to earn yields
- Two tiers: Tier 1 (immediate), Tier 2 (vested)
- 12-month vesting with daily unlock

## 2. How to Buy
1. Connect wallet (Phantom, Solflare, Torus)
2. Have USDC ready
3. Choose tier:
   - Tier 1: 1 USDC = 1 token (no vesting)
   - Tier 2: 1 USDC = 1.4 tokens (12-month vesting)
4. Click Mint → approve in wallet

## 3. How to Earn Yield
1. Stake tokens (voluntary)
2. Receive weekly USDC payouts
3. Currently 4% APY (via Aave)

## 4. How to Unstake
1. Click "Unstake"
2. Enter amount
3. Immediate access (no lockup)

## 5. Trading
- Buy: Minting (see above)
- Sell: Redeem (Tier 1 only) = 1 token → 1 USDC
- Transfer: Pay 0.1% tax (collected for treasury)

## Tiers Explained
| Feature | Tier 1 | Tier 2 |
|---------|--------|--------|
| Discount | 0% | 40-50% |
| Price | 1 USDC/token | 0.6-0.5 USDC/token |
| Vesting | None | 12 months |
| Staking | Yes | Yes |
| Yield | Yes | Yes |

## Support
Email: support@ecosystem-token.io
Discord: [Link]
Twitter: @EcosystemToken
```

#### G2. FAQ

```markdown
# FAQ

## General
**Q: Is this token a security?**
A: Consult your legal advisor. This is not financial advice.

**Q: How is the 1:1 USDC backing maintained?**
A: All USDC collected goes to vault. Smart contract enforces 1:1 redemption.

## Tier 2 / Vesting
**Q: Can I sell Tier 2 tokens before vesting?**
A: No. Transfer is blocked until tokens vest. Exception: can transfer claimed yield.

**Q: What if I die during vesting?**
A: Your beneficiary can claim vested tokens. Unvested tokens remain locked per contract.

**Q: Does vesting pause/unpause?**
A: No. Vesting continues regardless. It's a linear 365-day unlock.

## Yields
**Q: Where does yield come from?**
A: Aave lending (4% APY). 30% of transfer taxes also goes to yield vault.

**Q: How often do I get paid?**
A: Weekly (Thursdays 14:00 UTC). Must claim in dashboard.

**Q: What if I unstake before yield snapshot?**
A: You forfeit that week's yield. Claim before unstaking if available.

## Technical
**Q: Is smart contract audited?**
A: Yes. [Audit report link]

**Q: What if there's a bug?**
A: Launchpad can be paused. Emergency withdrawal procedures documented.

**Q: Which RPC does the dashboard use?**
A: Official Solana mainnet-beta RPC. Never custom RPC.
```

#### G3. Terms of Service

```markdown
# Terms of Service

## 1. Disclaimers
- This is NOT financial advice
- Cryptocurrency trading involves risk
- Smart contracts could be exploited
- USDC is stablecoin, not USD
- Aave yields not guaranteed

## 2. User Responsibilities
- Secure your wallet/seed phrase
- Approve transactions carefully
- Understand vesting terms
- Understand tax implications
- Don't trade on margin

## 3. Protocol Governance
- Upgrades via multisig (2 of 3)
- Pause/emergency features documented
- Transparency reports monthly
- User voting: TBD

## 4. Liability
- Smart contracts "as-is" without warranty
- Ecosystem Token team not liable for smart contract bugs
- Aave exploits: no recovery guarantee
- RPC downtime: not our fault

## 5. Contact
Privacy policy: [Link]
Terms (full): [Link]
```

---

### H. Post-Launch Monitoring

#### Metrics Dashboard

```
Real-time Monitoring:
├── TVL (Total Value Locked)
│   ├── Current: $__M
│   ├── Target: $10M (Week 1)
│   └── Status: 🟢 ON TRACK
│
├── Users
│   ├── Current: ____
│   ├── Target: 5,000 (Week 1)
│   └── Status: 🟢 ON TRACK
│
├── Smart Contract
│   ├── Last block: #16,500,000
│   ├── Failed txs (24h): 0
│   └── Status: 🟢 HEALTHY
│
├── Dashboard
│   ├── Uptime: 99.9%
│   ├── Avg response: 250ms
│   └── Status: 🟢 OPERATIONAL
│
└── Aave Position
    ├── Position: $___M
    ├── Interest earned: $___
    └── Status: 🟢 EARNING
```

#### Weekly Report Template

```markdown
# Weekly Operations Report [Week of DATE]

## Summary
- TVL: $__M (↑ __% from last week)
- Users: ____ (↑ __% from last week)
- Transactions: ____ (↑ __% from last week)
- Revenue (Treasury): $____ (↑ __% from last week)

## Key Events
1. [Event 1]
2. [Event 2]
3. [Event 3]

## Incidents
1. [Incident 1] - STATUS
2. [Incident 2] - RESOLVED

## Aave Position
- Principal: $__M
- Accrued interest: $____
- APY: 4.0%

## User Feedback
- Feature requests: [List]
- Bugs reported: [List]
- Social sentiment: [Assessment]

## Next Week
- [ ] Treasury distribution (Friday)
- [ ] Tier 2 whitelist update
- [ ] Dashboard improvements
- [ ] Monitoring optimization

## Sign-off
Prepared by: ___________
Date: _________________
```

---

### I. Upgrade Path

#### Version 2.0: Enhanced Yields
```
Timeline: 6 months post-launch

Features:
- Curve Finance LP rewards
- Yearn vault integration
- Dynamic APY based on TVL
- Governance token (v2 EGO)

Changes:
- New aave.rs module
- Updated treasury distribution
- Backward compatible (no migration)
```

#### Version 3.0: Cross-Chain
```
Timeline: 12 months post-launch

Features:
- Ethereum bridge (Wormhole)
- Polygon deployment
- Arbitrum deployment
- Multi-chain liquidity

Changes:
- Wormhole SPL token wrapper
- New contract on each chain
- Unified governance
```

---

## Checklist: Ready for Mainnet?

- [ ] Devnet testing complete
- [ ] Security audit passed (0 critical)
- [ ] External audit done (recommended)
- [ ] All code reviewed by 2+ senior devs
- [ ] Multisig wallet set up and tested
- [ ] USDC integration verified
- [ ] Dashboard tested on mainnet RPC
- [ ] Emergency procedures documented
- [ ] Monitoring system operational
- [ ] Team trained on operations
- [ ] User docs published
- [ ] Legal review complete
- [ ] Insurance obtained (if applicable)

---

## Launch Date: ________________

**Final Approval**:
- CEO: ___________ Date: ___
- CTO: ___________ Date: ___
- Auditor: ___________ Date: ___

---

**Status**: 🟡 IN DEVELOPMENT

**Completion Target**: Session 4, End of Day
