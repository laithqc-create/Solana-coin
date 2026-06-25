#!/bin/bash

# Ecosystem Token - Build & Deploy Script
# This script handles the complete build and deployment process

set -e

echo "=========================================="
echo "Ecosystem Token - Build & Deploy Script"
echo "=========================================="
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check prerequisites
echo -e "${YELLOW}[1/5] Checking prerequisites...${NC}"
echo ""

# Check Rust
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}✗ Rust not found${NC}"
    echo "  Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "  Then: source \$HOME/.cargo/env"
    exit 1
else
    RUST_VERSION=$(rustc --version)
    echo -e "${GREEN}✓ $RUST_VERSION${NC}"
fi

# Check Cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ Cargo not found${NC}"
    exit 1
else
    CARGO_VERSION=$(cargo --version)
    echo -e "${GREEN}✓ $CARGO_VERSION${NC}"
fi

# Check Anchor
if ! command -v anchor &> /dev/null; then
    echo -e "${RED}✗ Anchor not found${NC}"
    echo "  Install: npm install -g @coral-xyz/anchor"
    exit 1
else
    ANCHOR_VERSION=$(anchor --version)
    echo -e "${GREEN}✓ $ANCHOR_VERSION${NC}"
fi

# Check Solana
if ! command -v solana &> /dev/null; then
    echo -e "${RED}✗ Solana CLI not found${NC}"
    echo "  Install: curl https://release.solana.com/v1.18.0/install | sh"
    exit 1
else
    SOLANA_VERSION=$(solana --version)
    echo -e "${GREEN}✓ $SOLANA_VERSION${NC}"
fi

echo ""
echo -e "${YELLOW}[2/5] Checking Solana configuration...${NC}"
echo ""

# Check Solana config
SOLANA_URL=$(solana config get | grep "RPC URL" | awk '{print $NF}')
echo -e "${GREEN}✓ RPC URL: $SOLANA_URL${NC}"

# Check keypair
if [ -f ~/.config/solana/id.json ]; then
    PUBKEY=$(solana-keygen pubkey ~/.config/solana/id.json)
    echo -e "${GREEN}✓ Keypair: $PUBKEY${NC}"
    
    # Check balance
    BALANCE=$(solana balance --url devnet | awk '{print $1}')
    echo -e "${GREEN}✓ Devnet balance: $BALANCE SOL${NC}"
    
    if (( $(echo "$BALANCE < 2" | bc -l) )); then
        echo -e "${YELLOW}⚠ Balance low (<2 SOL), airdropping...${NC}"
        solana airdrop 5 --url devnet
    fi
else
    echo -e "${RED}✗ Keypair not found at ~/.config/solana/id.json${NC}"
    echo "  Create: solana-keygen new"
    exit 1
fi

echo ""
echo -e "${YELLOW}[3/5] Building smart contract...${NC}"
echo ""

cd "$(dirname "$0")/ecosystem-token"

# Clean
echo "Cleaning previous builds..."
rm -rf target/ Cargo.lock

# Build
echo "Building with Anchor..."
anchor build --skip-lint 2>&1 | tail -20

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    echo ""
    echo "Troubleshooting:"
    echo "  1. Try: rustup update stable"
    echo "  2. Try: cargo clean && cargo build"
    echo "  3. Or use Docker: docker build -t ecosystem-token ."
    exit 1
fi

echo ""
echo -e "${YELLOW}[4/5] Deploying to Devnet...${NC}"
echo ""

# Deploy
echo "Deploying program..."
DEPLOY_OUTPUT=$(anchor deploy --provider.cluster devnet 2>&1)
echo "$DEPLOY_OUTPUT"

# Extract program ID
PROGRAM_ID=$(echo "$DEPLOY_OUTPUT" | grep "Program Id:" | awk '{print $NF}' | tr -d '\n' | tr -d '\r')

if [ -z "$PROGRAM_ID" ]; then
    echo -e "${RED}✗ Deployment failed - could not find Program ID${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Deployment successful${NC}"
echo ""
echo -e "${GREEN}Program ID: $PROGRAM_ID${NC}"

# Save to file
echo "$PROGRAM_ID" > ../PROGRAM_ID.txt

echo ""
echo -e "${YELLOW}[5/5] Generating IDL...${NC}"
echo ""

# Generate IDL
sleep 5  # Wait for block confirmation
echo "Fetching IDL..."
anchor idl fetch "$PROGRAM_ID" -o target/idl/ecosystem_token.json

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ IDL generated${NC}"
    cp target/idl/ecosystem_token.json ../ECOSYSTEM_TOKEN_IDL.json
else
    echo -e "${YELLOW}⚠ IDL fetch failed (will work on next attempt)${NC}"
fi

echo ""
echo "=========================================="
echo -e "${GREEN}DEPLOYMENT COMPLETE!${NC}"
echo "=========================================="
echo ""
echo "Program ID: $PROGRAM_ID"
echo "Saved to: PROGRAM_ID.txt"
echo ""
echo "Next steps:"
echo "1. Update dashboard/.env.local:"
echo "   NEXT_PUBLIC_PROGRAM_ID=$PROGRAM_ID"
echo ""
echo "2. Start dashboard:"
echo "   cd dashboard"
echo "   npm install"
echo "   npm run dev"
echo ""
echo "3. Open http://localhost:3000"
echo ""
