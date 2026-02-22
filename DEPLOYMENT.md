# Contract Deployment Guide

This guide explains how to deploy the ChainLojistic smart contract to Stellar Testnet.

## Prerequisites

- Rust installed (1.70+)
- Soroban CLI installed
- Stellar account with testnet XLM (for fees)
- Secret key for the deployer account (get from https://laboratory.stellar.org/#account-creator)

## Installation

### Install System Dependencies

On Linux (Ubuntu/Debian), install required system libraries:

```bash
sudo apt-get update
sudo apt-get install -y libudev-dev pkg-config
```

### Install Soroban CLI

```bash
cargo install --locked soroban-cli
```

**Note:** The installation may take 10-15 minutes as it compiles from source. Be patient!

If the installation fails with missing dependencies, install them using your system's package manager:
- **Ubuntu/Debian:** `sudo apt-get install -y libudev-dev pkg-config`
- **Fedora/RHEL:** `sudo dnf install systemd-devel pkg-config`
- **Arch Linux:** `sudo pacman -S systemd pkg-config`

After installation, verify it works:
```bash
soroban --version
```

If `soroban` command is not found, add Cargo's bin directory to your PATH:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
# Add this line to your ~/.bashrc or ~/.zshrc to make it permanent
```

### Get Testnet Account and Secret Key

You need a Stellar Testnet account with a secret key to deploy contracts. Here's how to get one:

#### Option 1: Using Stellar Laboratory (Recommended)

1. **Create a Testnet Account:**
   - Go to https://laboratory.stellar.org/#account-creator
   - Click "Generate Keypair" to create a new account
   - The page will show you:
     - **Public Key** (also called Account ID): Starts with `G...`
     - **Secret Key** (also called Secret Seed): Starts with `S...`

2. **Save Your Secret Key:**
   - **IMPORTANT:** Copy and save your secret key immediately
   - The secret key starts with `S` and looks like: `SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX`
   - Store it securely - you'll need it for deployment
   - **Never share your secret key publicly or commit it to version control**

3. **Fund Your Account:**
   - After generating, click "Fund Account" to get testnet XLM from the friendbot
   - You should receive 10,000 testnet XLM (free, for testing only)
   - Wait a few seconds for the transaction to complete

#### Option 2: Using Stellar CLI

If you have the Stellar CLI installed:

```bash
# Generate a new keypair
stellar-cli generate --network testnet

# This will output:
# Public Key: G...
# Secret Key: S...
```

Then fund it using the friendbot:
```bash
curl "https://friendbot.stellar.org/?addr=YOUR_PUBLIC_KEY"
```

#### Option 3: Using Soroban CLI

The Soroban CLI can also generate keys:

```bash
# Generate a new keypair
soroban keys generate --global testnet-key

# This will create and store a keypair
# To see the secret key:
soroban keys show testnet-key
```

#### Verify Your Account

Check that your account is funded:

```bash
# Using Soroban CLI
soroban keys fund testnet-key --network testnet

# Or check balance manually
soroban contract invoke \
  --id YOUR_PUBLIC_KEY \
  --network testnet \
  -- get_balance
```

**Note:** For testnet, secret keys are safe to use for testing, but still don't commit them to version control. For mainnet, treat secret keys as highly sensitive credentials.

## Deployment Steps

### 1. Build the Contract

```bash
cd smart-contract/contracts
cargo build --target wasm32-unknown-unknown --release
```

The WASM file will be at:
```
target/wasm32-unknown-unknown/release/hello_world.wasm
```

### 2. Deploy Using Script

The easiest way is to use the deployment script. You have several options for providing your secret key:

#### Option A: Pass as Command Line Argument

```bash
cd smart-contract
chmod +x scripts/deploy.sh
./scripts/deploy.sh --network testnet --source-key "SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
```

**Important:** The secret key must be quoted (use double quotes `"..."`) because it's a single string. Replace `SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` with your actual secret key (starts with `S`, 56 characters long).

**Note:** If you have a mnemonic phrase (12 words), that's not a Stellar secret key. You need to get a Stellar secret key from the Stellar Laboratory (see [GETTING_SECRET_KEY.md](./GETTING_SECRET_KEY.md)).

#### Option B: Use Environment Variable (More Secure)

Set the secret key as an environment variable:

```bash
export SOROBAN_SECRET_KEY=SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
cd smart-contract
./scripts/deploy.sh --network testnet
```

This is more secure because the key won't appear in your command history.

#### Option C: Use Soroban Key Management

If you stored your key using Soroban CLI:

```bash
# First, make sure your key is available
soroban keys show testnet-key

# Then deploy (the script will use the key from Soroban's key store)
cd smart-contract
./scripts/deploy.sh --network testnet
```

**Security Tip:** Never commit your secret key to git. Always use environment variables or the `--source-key` flag at runtime.

### 3. Manual Deployment

If you prefer to deploy manually:

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/hello_world.wasm \
  --source-account YOUR_SECRET_KEY \
  --network testnet
```

The command will output the contract ID.

### 4. Save Contract ID

After deployment, save the contract ID:

**For smart-contract:**
```bash
echo "CONTRACT_ID=YOUR_CONTRACT_ID" >> smart-contract/.env
echo "NETWORK=testnet" >> smart-contract/.env
```

**For frontend:**
```bash
echo "NEXT_PUBLIC_CONTRACT_ID=YOUR_CONTRACT_ID" >> frontend/.env.local
echo "NEXT_PUBLIC_STELLAR_NETWORK=testnet" >> frontend/.env.local
```

## Verify Deployment

Test that the contract is deployed:

```bash
soroban contract invoke \
  --id YOUR_CONTRACT_ID \
  --source-account YOUR_SECRET_KEY \
  --network testnet \
  -- ping
```

## Troubleshooting

### "Account not found"
- Make sure your account is funded with testnet XLM
- Check that you're using the correct network

### "Insufficient balance"
- Get more testnet XLM from the friendbot
- Testnet accounts need XLM for transaction fees

### "Contract not found"
- Verify the contract ID is correct
- Check that you're using the right network

## Redeployment

To redeploy (creates a new contract ID):

```bash
./scripts/deploy.sh --network testnet --source-key YOUR_SECRET_KEY
```

**Note:** Redeploying creates a new contract. Update your frontend config with the new contract ID.

## Security Notes

- Never commit secret keys to version control
- Use environment variables for sensitive data
- Testnet keys are safe to use, but mainnet keys must be kept secure
- Consider using a separate deployer account with limited funds

## Next Steps

After deployment:
1. Update frontend `.env.local` with the contract ID
2. Restart the frontend dev server
3. Test product registration and event tracking
4. Verify the timeline loads events correctly
