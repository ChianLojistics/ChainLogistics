#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
NETWORK="testnet"
CONTRACT_NAME="hello_world"
WASM_PATH="target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm"

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --network)
      NETWORK="$2"
      shift 2
      ;;
    --source-key)
      SOURCE_KEY="$2"
      shift 2
      ;;
    --contract-name)
      CONTRACT_NAME="$2"
      WASM_PATH="target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm"
      shift 2
      ;;
    --help)
      echo "Usage: $0 [OPTIONS]"
      echo "  --network NETWORK     Network (testnet, mainnet, futurenet)"
      echo "  --source-key KEY      Deployer secret key"
      echo "  --contract-name NAME  Contract name (default: hello_world)"
      exit 0
      ;;
    *)
      echo -e "${RED}Unknown option: $1${NC}"
      echo "Use --help for usage information"
      exit 1
      ;;
  esac
done

echo -e "${GREEN}Building contract...${NC}"
cd "$(dirname "$0")/.."
cd contracts

# Build the contract
cargo build --target wasm32-unknown-unknown --release

if [ ! -f "../${WASM_PATH}" ]; then
  echo -e "${RED}Error: WASM file not found at ${WASM_PATH}${NC}"
  exit 1
fi

echo -e "${GREEN}Contract built successfully${NC}"

if [ -z "$SOURCE_KEY" ]; then
  if [ -z "$SOROBAN_SECRET_KEY" ]; then
    echo -e "${RED}Error: No source key provided${NC}"
    echo "Set SOROBAN_SECRET_KEY or use --source-key"
    exit 1
  fi
  SOURCE_KEY="$SOROBAN_SECRET_KEY"
fi

if [ "${#SOURCE_KEY}" -lt 50 ] || ([ "${SOURCE_KEY:0:1}" != "S" ] && [ "${SOURCE_KEY:0:2}" != "SB" ]); then
  echo -e "${RED}Error: Invalid secret key format${NC}"
  echo "Secret key must start with 'S' or 'SB' and be at least 50 characters"
  echo "Get a key from: https://laboratory.stellar.org/#account-creator"
  exit 1
fi

echo -e "${GREEN}Deploying to ${NETWORK}...${NC}"

# Deploy the contract
echo -e "${YELLOW}Deploying contract...${NC}"
DEPLOY_OUTPUT=$(soroban contract deploy \
  --wasm "../${WASM_PATH}" \
  --source-account "$SOURCE_KEY" \
  --network "$NETWORK" 2>&1)

DEPLOY_EXIT_CODE=$?

if [ $DEPLOY_EXIT_CODE -ne 0 ]; then
  echo -e "${RED}Error: Deployment failed${NC}"
  echo "$DEPLOY_OUTPUT"
  exit 1
fi

CONTRACT_ID=$(echo "$DEPLOY_OUTPUT" | grep -E "^C[A-Z0-9]{55}$" | tail -1 | tr -d '[:space:]')

if [ -z "$CONTRACT_ID" ]; then
  echo -e "${RED}Error: Failed to extract contract ID${NC}"
  echo "$DEPLOY_OUTPUT"
  exit 1
fi

echo -e "${GREEN}Deployed: ${CONTRACT_ID}${NC}"

ENV_FILE="../.env"
echo "CONTRACT_ID=${CONTRACT_ID}" > "$ENV_FILE"
echo "NETWORK=${NETWORK}" >> "$ENV_FILE"

FRONTEND_ENV_FILE="../../frontend/.env.local"
if [ -f "$FRONTEND_ENV_FILE" ]; then
  sed -i '/^NEXT_PUBLIC_CONTRACT_ID=/d' "$FRONTEND_ENV_FILE"
  sed -i '/^NEXT_PUBLIC_STELLAR_NETWORK=/d' "$FRONTEND_ENV_FILE"
fi
echo "NEXT_PUBLIC_CONTRACT_ID=${CONTRACT_ID}" >> "$FRONTEND_ENV_FILE"
echo "NEXT_PUBLIC_STELLAR_NETWORK=${NETWORK}" >> "$FRONTEND_ENV_FILE"
