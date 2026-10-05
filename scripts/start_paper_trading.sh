#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

if [ -f arb-bot.pid ]; then
    PID=$(cat arb-bot.pid)
    if kill -0 "$PID" 2>/dev/null; then
        echo "arb-bot is already running (PID: $PID)."
        exit 0
    else
        rm -f arb-bot.pid
    fi
fi

echo "Starting arb-bot in paper-trading simulation mode..."
nohup ./arb-bot -c config.toml > paper_trading.log 2>&1 &
PID=$!
echo "$PID" > arb-bot.pid
echo "arb-bot started with PID: $PID"
echo "Log file: $DIR/paper_trading.log"
echo "Use './status_paper_trading.sh' to monitor."
