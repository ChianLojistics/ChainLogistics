#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}ChainLogistics Deployment Setup${NC}"
echo ""

# Check if Soroban CLI is installed
if command -v soroban &> /dev/null; then
    echo -e "${GREEN}✓ Soroban CLI is already installed${NC}"
    soroban --version
else
    echo -e "${YELLOW}Installing Soroban CLI...${NC}"
    echo ""
    echo "This may take several minutes. Please wait..."
    
    # Check for required system dependencies
    if ! dpkg -l | grep -q libudev-dev; then
        echo -e "${YELLOW}Missing system dependency: libudev-dev${NC}"
        echo "Please run: sudo apt-get install -y libudev-dev pkg-config"
        echo ""
        read -p "Have you installed the dependencies? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo -e "${RED}Please install dependencies first and run this script again.${NC}"
            exit 1
        fi
    fi
    
    # Install Soroban CLI
    cargo install --locked soroban-cli
    
    # Verify installation
    if command -v soroban &> /dev/null; then
        echo -e "${GREEN}✓ Soroban CLI installed successfully!${NC}"
        soroban --version
    else
        # Check if it's in cargo bin but not in PATH
        if [ -f ~/.cargo/bin/soroban ]; then
            echo -e "${YELLOW}Soroban CLI installed but not in PATH${NC}"
            echo "Add this to your ~/.bashrc or ~/.zshrc:"
            echo "  export PATH=\"\$HOME/.cargo/bin:\$PATH\""
            echo ""
            echo "Then run: source ~/.bashrc (or source ~/.zshrc)"
        else
            echo -e "${RED}Installation may have failed. Check the error messages above.${NC}"
            exit 1
        fi
    fi
fi

echo ""
echo -e "${GREEN}Setup complete!${NC}"
echo ""
echo "Next steps:"
echo "1. Get a Stellar Testnet secret key from: https://laboratory.stellar.org/#account-creator"
echo "2. Run the deployment script:"
echo "   ./scripts/deploy.sh --network testnet --source-key \"YOUR_SECRET_KEY\""
