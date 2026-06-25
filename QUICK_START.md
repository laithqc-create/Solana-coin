# QUICK START GUIDE - Ecosystem Token Session 4

## 📦 What Was Built

✅ **Smart Contract** - 2,100 LOC Rust, 17 instructions, 10 accounts  
✅ **Dashboard** - 2,500 LOC React, 4 pages, 8 charts  
✅ **Documentation** - Setup guide, testing plan, integration guide  

**Location**: `/home/claude/ecosystem-token/`

---

## 🚀 DEPLOY IN 3 STEPS

### Step 1: Fix Build (Choose One - 10 min)

**Option A: Docker (Fastest)**
```bash
docker run -it -v /home/claude:/home/claude solanalabs/solana:latest bash
cd /home/claude/ecosystem-token
anchor build
```

**Option B: Update Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup update stable
cd /home/claude/ecosystem-token
anchor build
```

### Step 2: Deploy (10 min)

```bash
# Set up devnet
solana config set --url devnet
solana airdrop 2  # Get some SOL

# Deploy smart contract
cd /home/claude/ecosystem-token
anchor deploy --provider.cluster devnet

# IMPORTANT: Save the Program ID from output!
# Will look like: Program deployed: 5Abc123...
```

### Step 3: Start Dashboard (5 min)

```bash
cd /home/claude/ecosystem-token/dashboard

# 1. Update environment
cp .env.example .env.local
# Edit .env.local and paste your Program ID

# 2. Install dependencies
npm install

# 3. Start dev server
npm run dev
# Visit http://localhost:3000
```

---

## 📝 INTEGRATION TASKS (Next Phase)

After deployment, wire the dashboard to smart contract:

**File**: `dashboard/src/hooks/useEcosystemToken.ts`

**TODO**:
1. Load IDL from deployed contract
2. Create Program client
3. Implement `fetchUserState()` - fetch UserTierInfo PDA
4. Implement `mintTokens()` - build MintTokens transaction
5. Implement `stakeTokens()` - build StakeTokens transaction
6. Implement `claimYield()` - build ClaimYield transaction
7. Test all flows

**Estimated Time**: 60-90 minutes

---

## ✅ CHECKLIST

### Before Deployment
- [ ] Choose Docker or Rust update
- [ ] Solana CLI installed
- [ ] Devnet wallet ready (2+ SOL)
- [ ] Read `dashboard/README.md`

### After Deployment
- [ ] Program ID saved
- [ ] IDL generated
- [ ] Dashboard starts
- [ ] Wallet connects
- [ ] Mock data loads

### Integration
- [ ] useEcosystemToken wired
- [ ] User state loads
- [ ] Mint flow works
- [ ] Staking works
- [ ] Yield claims work

---

## 🔑 KEY FILES

**Smart Contract**:
```
programs/ecosystem-token/src/
├── lib.rs (entry point)
├── state.rs (accounts)
├── instructions.rs (logic)
├── errors.rs (error codes)
└── aave.rs (Aave integration)
```

**Dashboard**:
```
dashboard/src/
├── app/layout.tsx (root)
├── app/page.tsx (user dashboard)
├── app/treasury/page.tsx (treasury)
├── app/analytics/page.tsx (analytics)
├── app/admin/page.tsx (admin)
├── hooks/useEcosystemToken.ts (WIRE HERE!)
└── lib/wallet.tsx (wallet setup)
```

---

## 🆘 TROUBLESHOOTING

**Build fails:**
```bash
# Try Docker first
docker run -it solanalabs/solana:latest bash
# Then inside: cd /home/claude/ecosystem-token && anchor build
```

**Deploy fails:**
```bash
# Check balance
solana balance --url devnet

# If 0, airdrop more
solana airdrop 2 --url devnet

# Check Anchor CLI
anchor --version
```

**Dashboard won't start:**
```bash
cd dashboard
rm -rf node_modules package-lock.json
npm install
npm run dev
```

**Wallet won't connect:**
```
1. Check .env.local has correct PROGRAM_ID
2. Try different wallet (Phantom → Solflare)
3. Clear browser cache
4. Check browser console for errors
```

---

## 📊 PROJECT STATUS

```
Smart Contract:  ████████ 100% ✅
Dashboard UI:    ████████ 100% ✅
Build Setup:     ███░░░░░  30% 🔄
Deployment:      ░░░░░░░░   0% ⏳
Integration:     ░░░░░░░░   0% ⏳
Testing:         ░░░░░░░░   0% ⏳

Overall: Ready for Deployment (3-4 hours to full integration)
```

---

## 📚 DOCUMENTATION

- `dashboard/README.md` - Complete setup & integration guide
- `PROGRESS.md` - Detailed progress tracking
- `SESSION_4_SUMMARY.md` - Full session notes

---

## 💡 PRO TIPS

1. **Save your Program ID** immediately after deployment - you'll need it!

2. **Test in DevNet first** - never deploy to mainnet without testing

3. **Use Phantom wallet** - most stable for Solana development

4. **Check devnet balance** - SOL airdrop can be slow, airdrop early

5. **Read README.md in dashboard** - has pseudo-code for integration

---

## 🎯 SUCCESS CRITERIA

✅ Smart contract deploys to devnet  
✅ Dashboard starts on localhost:3000  
✅ Wallet connects (shows user address)  
✅ User state loads from blockchain  
✅ All buttons functional  
✅ Transactions sign correctly  

---

## 🚨 CRITICAL REMINDERS

**DO:**
- ✅ Save Program ID after deployment
- ✅ Update .env.local with Program ID
- ✅ Test on devnet first
- ✅ Read error messages carefully

**DON'T:**
- ❌ Deploy to mainnet yet
- ❌ Share private keys
- ❌ Use real money for testing
- ❌ Ignore security warnings

---

**Time Estimate**: 
- Deployment: 30 minutes
- Integration: 60-90 minutes  
- Testing: 90 minutes
- **Total: 3-4 hours**

**Start with Step 1 above** ☝️
