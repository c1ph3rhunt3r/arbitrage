#!/usr/bin/env bash
set -e

REPO="c1ph3rhunt3r/arbitrage"
INSTALL_DIR="$HOME/Solana"

echo "=== Deploying arb-bot binary on VPS ==="
cd "$INSTALL_DIR"

DOWNLOAD_URL="https://github.com/$REPO/releases/download/latest-build/arb-bot-linux-x86_64.tar.gz"

echo "Downloading release from: $DOWNLOAD_URL"
if command -v curl >/dev/null 2>&1; then
    curl -L -o arb-bot-linux-x86_64.tar.gz "$DOWNLOAD_URL"
elif command -v wget >/dev/null 2>&1; then
    wget -O arb-bot-linux-x86_64.tar.gz "$DOWNLOAD_URL"
else
    echo "Error: Neither curl nor wget is installed."
    exit 1
fi

echo "Extracting binary..."
tar -xzvf arb-bot-linux-x86_64.tar.gz
chmod +x arb-bot
rm -f arb-bot-linux-x86_64.tar.gz

echo "Verifying installation..."
./arb-bot -v

echo "=== Deployment successful! ==="
echo "To test run in paper trading mode:"
echo "  cd $INSTALL_DIR && ./arb-bot -c config.toml"
