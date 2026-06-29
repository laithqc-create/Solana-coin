# Solana Ecosystem Token — PROGRESS.md
**Last updated:** Session 9 — Architecture pivot confirmed

---

## ✅ COMPLETED

### Smart Contract (Rust/Anchor 0.29.0)
All 27 instructions implemented across 8 source files:
- `lib.rs` — Program entry, declare_id (placeholder, injected via CI)
- `state.rs` — All account structs (Treasury, UserAccount, LaunchpadState, etc.)
- `instructions.rs` — Core: initialize, mint, burn, claim_yield, request_unstake, complete_unstake
- `investment_instructions.rs` — Sky Protocol + Ethena/Meteora dual investment strategies
- `yield_strategy.rs` — 45% APY (holders), 70% APY (campaign buyers), 80/20 invested/liquid
- `morpho.rs` — USDC pool integration (Sky Protocol)
- `rwa.rs` — USDT pool integration (Ethena/Meteora)
- `errors.rs` — All custom error codes

### Architecture (Locked)
- 1:1 USDT peg, unlimited minting
- 0.1% mint fee, 0.1% burn fee, 0.5% emergency redeem fee
- Auto-yield: 45% APY (regular holders), 70% APY (campaign buyers)
- 48-business-hour unstaking countdown
- Dual investment: Sky Protocol (USDC, 80/20), Ethena/Meteora (USDT, 80/20)
- Revenue split: 70% treasury / 30% stakers

### GitHub Actions CI/CD (FIXED Session 9)
- **Root workflow:** `.github/workflows/build.yml` — REWRITTEN
  - `anchor build` → real SBF bytecode (NOT `cargo build --release`)
  - Solana CLI v1.18.26, Anchor CLI 0.29.0, Rust 1.75.0
  - PROGRAM_ID injected via GitHub Secret (not hardcoded)
  - Auto-deploy to devnet when `PROGRAM_ID` + `DEPLOY_KEYPAIR` secrets are set
  - Validates .so file size (> 10KB)
  - Uploads `ecosystem_token.so` + IDL as artifacts (30-day retention)
- **Removed:** Duplicate `ecosystem-token/.github/workflows/build.yml`
- **Removed:** Incorrect `cargo build --release` (was producing x86, not SBF)

### License Audit ✅
- anchor-lang 0.29.0 → Apache 2.0 ✅
- anchor-spl 0.29.0 → Apache 2.0 ✅
- spl-token 3.5.0 → Apache 2.0 ✅
- rust_decimal 1.27 → MIT ✅

---

## 🏗️ ARCHITECTURE (Updated Session 9)

```
GitHub Actions (CI/CD)
  └── anchor build → ecosystem_token.so (SBF bytecode)
      └── anchor deploy → Solana Devnet/Mainnet

Solana Program (on-chain)
  └── RPC Endpoint (devnet / mainnet)
      │
      ├── Telegram Mini App (frontend + tenant control)
      │   ├── Bot: @YourBot
      │   ├── Tech stack: TBD (React/Next.js or plain HTML/JS)
      │   ├── Auth: Telegram WebApp.initData
      │   └── Tenant control panel: admin commands via bot
      │
      └── Supabase (backend / database)
          ├── Tables: users, transactions, yield_snapshots, campaigns
          ├── Auth: Telegram user_id → Supabase row
          ├── Edge Functions: yield calculations, fee tracking
          └── Realtime: live balance updates to Mini App
```

---

## 🔴 CURRENT BLOCKERS (in order)

### BLOCKER 1: Program ID + GitHub Secrets (YOU DO THIS)
The CI workflow injects PROGRAM_ID from GitHub Secrets at build time.
Steps:
1. Install Solana CLI locally OR use GitHub Codespaces
2. `solana-keygen new -o program-keypair.json --no-passphrase`
3. Extract Program ID: `solana-keygen pubkey program-keypair.json`
4. Go to: https://github.com/laithqc-create/Solana-coin/settings/secrets/actions
5. Add secret: `PROGRAM_ID` = your public key
6. Add secret: `DEPLOY_KEYPAIR` = contents of `program-keypair.json` (the JSON array)
7. `git push origin main` → triggers CI → auto-compiles + deploys to devnet

### BLOCKER 2: Telegram Mini App (NEEDS YOUR INPUT)
Waiting for your instructions on:
- Telegram Bot username/token (set as `TELEGRAM_BOT_TOKEN` secret)
- Mini App pages needed (user dashboard? admin panel? both?)
- Tech stack preference (React? Vue? Plain HTML?)
- Which on-chain data to display

### BLOCKER 3: Supabase Setup (NEEDS YOUR INPUT)
Waiting for your instructions on:
- Supabase project URL + anon key (set as GitHub/env secrets)
- Tables needed (users, transactions, campaigns, etc.)
- Edge Functions (yield snapshots, fee aggregation?)
- Auth flow (Telegram initData → Supabase JWT?)

---

## 📁 CURRENT FILE STATE

```
Solana-coin/
├── PROGRESS.md                          ← YOU ARE HERE
├── .github/
│   └── workflows/
│       └── build.yml                    ← FIXED: anchor build → SBF ✅
├── ecosystem-token/
│   ├── Anchor.toml                      ← UPDATED: versions locked ✅
│   ├── Cargo.toml                       ← Workspace deps (unchanged)
│   └── programs/
│       └── ecosystem-token/
│           ├── Cargo.toml               ← crate-type = cdylib ✅
│           └── src/
│               ├── lib.rs               ← declare_id = placeholder (CI injects real)
│               ├── state.rs             ← All account structs ✅
│               ├── instructions.rs      ← Core instructions ✅
│               ├── investment_instructions.rs ← Dual investment ✅
│               ├── yield_strategy.rs    ← APY logic ✅
│               ├── morpho.rs            ← Sky Protocol ✅
│               ├── rwa.rs               ← Ethena/Meteora ✅
│               └── errors.rs            ← Custom errors ✅
└── [dashboard/ — DEPRECATED, replaced by Telegram Mini App]
```

---

## 📋 NEXT STEPS (Exact)

1. **You:** Add `PROGRAM_ID` + `DEPLOY_KEYPAIR` to GitHub Secrets
   - URL: https://github.com/laithqc-create/Solana-coin/settings/secrets/actions
2. **You:** Tell Claude — Telegram Bot details + Mini App requirements
3. **You:** Tell Claude — Supabase project URL + what tables/functions needed
4. **Claude:** Build Telegram Mini App (production-ready, Rules 1–8)
5. **Claude:** Build Supabase schema + Edge Functions
6. **Claude:** Wire Mini App → Supabase → Solana RPC
7. **E2E test:** mint, burn, yield, unstake, admin controls

---

## ⚠️ SECRETS NEEDED (GitHub → Settings → Secrets → Actions)

| Secret Name | Value | Status |
|-------------|-------|--------|
| `PROGRAM_ID` | Your solana-keygen public key | ❌ NOT SET |
| `DEPLOY_KEYPAIR` | Contents of program-keypair.json | ❌ NOT SET |
| `TELEGRAM_BOT_TOKEN` | From @BotFather | ❌ NOT SET (pending your input) |
| `SUPABASE_URL` | https://xxx.supabase.co | ❌ NOT SET (pending your input) |
| `SUPABASE_ANON_KEY` | Your anon key | ❌ NOT SET (pending your input) |

**NEVER commit these values to git.**

---

## RESUME FROM HERE (next session)

1. Read this file first ✅
2. Check if `PROGRAM_ID` secret was added → if yes, check CI run status
3. Get Telegram Bot + Supabase details from user
4. Build Telegram Mini App + Supabase schema
5. Wire everything end-to-end

---
*Rules 1–8 active. All code: error handling, input validation, env vars for secrets, no redundant loops.*
