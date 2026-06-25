# GitHub Push & CI/CD Setup Guide

## Overview

You can now push this entire project to GitHub and it will **automatically**:
1. ✅ Build the smart contract
2. ✅ Run tests
3. ✅ Deploy to Solana devnet (optional)
4. ✅ Generate IDL
5. ✅ Create releases

**No local compilation needed!** GitHub Actions handles everything.

---

## Prerequisites

You need:
1. GitHub account
2. GitHub personal access token (for pushing code)
3. (Optional) Solana keypair secret (for devnet deployment)

---

## Step 1: Create GitHub Repository

### Option A: GitHub Web (Easiest on Mobile)

1. Go to https://github.com/new
2. Create new repository
   - Name: `ecosystem-token`
   - Description: "Solana collateral-backed ecosystem token"
   - Public or Private (your choice)
   - ✅ Add README
   - ✅ Add .gitignore (select Rust)
3. Click "Create repository"

### Option B: Command Line

```bash
# Initialize git in your project
cd /home/claude/ecosystem-token
git init
git add .
git commit -m "Initial commit: Complete ecosystem token"
git branch -M main

# Connect to GitHub
git remote add origin https://github.com/YOUR_USERNAME/ecosystem-token.git
git push -u origin main
```

---

## Step 2: Push Code to GitHub

### From Command Line (Recommended)

```bash
cd /home/claude

# Create .gitignore
cat > .gitignore << 'EOF'
# Rust
/target/
Cargo.lock
*.swp
*.swo

# Node
node_modules/
.next/
dist/

# IDE
.vscode/
.idea/
*.iml

# Environment
.env.local
.env*.local

# OS
.DS_Store
.env

# Build artifacts
*.so
*.wasm
EOF

git init
git config user.name "Your Name"
git config user.email "your.email@example.com"

git add ecosystem-token/
git add .github/
git add .gitignore
git add README.md

git commit -m "Initial commit: Complete Solana ecosystem token

- Smart contract: 17 instructions, 10 accounts
- Dashboard: 4 pages, 8+ charts
- Documentation: Complete guides
- CI/CD: GitHub Actions for automatic builds"

git branch -M main
git remote add origin https://github.com/YOUR_USERNAME/ecosystem-token.git
git push -u origin main -f
```

### Via GitHub Mobile App

1. Download **GitHub mobile app** (iOS/Android)
2. Tap profile icon → "Repositories"
3. Create new repository
4. Choose to push your local code

---

## Step 3: Configure Deployment Secrets (Optional)

If you want GitHub Actions to **automatically deploy to devnet**:

### Generate Solana Keypair

```bash
# Generate a NEW keypair for CI/CD (don't use main wallet!)
solana-keygen new -o ci-keypair.json --no-bip39-passphrase

# Get the secret key content
cat ci-keypair.json
```

### Add to GitHub Secrets

1. Go to your GitHub repo
2. Settings → Secrets and variables → Actions
3. Click "New repository secret"
4. Name: `SOLANA_KEYPAIR`
5. Value: Paste entire JSON from `ci-keypair.json`
6. Click "Add secret"

⚠️ **Important**: Use a separate keypair for CI/CD (not your main wallet!)

---

## Step 4: Run Build

### Option A: Automatic (on Push)

Just push code to `main` branch - GitHub Actions will:
1. Automatically start building
2. Run tests
3. Deploy to devnet (if secret configured)

**View progress**:
- Go to your repo → "Actions" tab
- Click the workflow run to see logs

### Option B: Manual Trigger

On GitHub:
1. Go to "Actions" tab
2. Select "Build & Deploy Ecosystem Token"
3. Click "Run workflow"
4. Click green "Run workflow" button
5. Optionally check "Deploy to devnet" checkbox

---

## What Happens Automatically

### On Every Push to `main`

```
GitHub Action Triggered
    ↓
1. Checkout code (30 sec)
    ↓
2. Install Rust (1 min)
    ↓
3. Install Anchor & Solana CLI (1 min)
    ↓
4. Build smart contract (3-5 min)
    ↓
5. Run tests (1 min)
    ↓
6. Generate IDL (1 min)
    ↓
7. (Optional) Deploy to devnet (2 min)
    ↓
✅ Complete! Total: 8-12 minutes
```

### Artifacts Created

After build completes:
- **Compiled program** (.so file)
- **IDL** (JSON interface definition)
- **Build logs** (for debugging)

Download from Actions tab → Artifacts

---

## Getting Program ID After Deploy

After deployment to devnet:

1. Go to "Actions" tab
2. Click the latest "Deploy to Devnet" job
3. Look for log line: `Program deployed: ABC123...`
4. Save that as your `PROGRAM_ID`

Or from command line:
```bash
# Check devnet for your program
solana program show <YOUR_PROGRAM_ID> --url devnet
```

---

## Workflow Files Created

The GitHub Actions workflow includes:

**File**: `.github/workflows/build-deploy.yml`

**Includes**:
- ✅ Build step (cargo + anchor build)
- ✅ Test step (cargo test)
- ✅ Lint step (cargo fmt)
- ✅ Deploy step (anchor deploy to devnet)
- ✅ IDL generation
- ✅ Artifact upload
- ✅ Release creation

**Triggers**:
- On push to main/develop
- On pull requests
- Manual trigger from Actions tab

---

## Managing Releases

After each successful build:

1. A GitHub Release is automatically created
2. Contains build artifacts
3. Visible in "Releases" tab
4. Can download compiled .so files

To download artifacts:
1. Go to repo → "Releases"
2. Click the latest release
3. Download the .so files
4. Use for deployment

---

## Viewing Build Logs

### From GitHub Web

1. Click "Actions" tab
2. Select the workflow run
3. Click on "Build Smart Contract" job
4. View full logs (all commands and output)

### From GitHub Mobile

1. Open app
2. Go to repo
3. Tap "Actions"
4. Tap the workflow
5. Scroll through logs

---

## Troubleshooting

### Build Failed
- Check "Actions" tab → see logs
- Common issues:
  - `Cargo.lock` conflicts → Delete and retry
  - Network issues → Retry workflow
  - Dependency issue → Check versions in Cargo.toml

### Deploy Failed
- ✅ Verify `SOLANA_KEYPAIR` secret is set
- ✅ Check keypair has devnet SOL balance
- ✅ Check Solana is up (https://status.solana.com)

### Can't Push Code
- ✅ Check GitHub token has repo access
- ✅ Use SSH key instead of token (easier)
- ✅ Run: `git config user.email "you@example.com"`

---

## Next Steps (After Push)

1. **View build** → Go to Actions tab
2. **Get program ID** → Check deployment logs
3. **Update dashboard** → Edit .env.local with program ID
4. **Start dashboard** → `npm run dev`

---

## Helpful Git Commands

```bash
# Check status
git status

# Add all files
git add .

# Commit with message
git commit -m "Your message"

# Push to GitHub
git push origin main

# Create new branch
git checkout -b feature/my-feature

# Pull latest
git pull origin main
```

---

## GitHub Actions Costs

- ✅ **Free tier**: 2,000 minutes/month
- ✅ **Builds take**: ~10 minutes each
- ✅ **Can run**: ~200 builds/month free

Perfect for development!

---

## Security Notes

⚠️ **DO NOT**:
- Commit private keys
- Commit `.env` files with secrets
- Commit `Cargo.lock` unless necessary

✅ **DO**:
- Use GitHub Secrets for sensitive data
- Keep Solana CI/CD keypair separate
- Review all logs before deploying to mainnet

---

## Full Workflow on Mobile

1. **Edit code** on your laptop/desktop
2. **Commit locally**: `git commit -m "my changes"`
3. **Push to GitHub**: `git push origin main`
4. **Watch build** on GitHub mobile app (Actions tab)
5. **Download artifacts** when complete
6. **Use program ID** in dashboard

**Zero compilation needed on your device!**

---

## References

- Solana CLI: https://docs.solana.com/cli
- Anchor: https://docs.anchor-lang.com
- GitHub Actions: https://docs.github.com/actions
- Solana Devnet Faucet: https://faucet.solana.com

---

**Status**: Ready to push to GitHub  
**Next**: Provide GitHub token and run push commands  
**Time**: 5 minutes to setup, then automatic builds forever!
