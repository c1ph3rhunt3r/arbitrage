#!/usr/bin/env bash

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

echo "=== Arb-Bot Paper Trading Status ==="

IS_RUNNING=0
if [ -f arb-bot.pid ]; then
    PID=$(cat arb-bot.pid)
    if kill -0 "$PID" 2>/dev/null; then
        echo "Status: RUNNING (PID: $PID)"
        IS_RUNNING=1
    else
        echo "Status: STOPPED (stale PID file found)"
    fi
else
    if pgrep -f "arb-bot -c config.toml" > /dev/null; then
        echo "Status: RUNNING (detected via pgrep)"
        IS_RUNNING=1
    else
        echo "Status: STOPPED"
    fi
fi

if [ $IS_RUNNING -eq 1 ]; then
    echo ""
    echo "--- Resource Usage ---"
    ps aux | grep "[a]rb-bot -c config.toml" | awk '{printf "PID: %s | CPU: %s%% | MEM: %s%% (%s KB)\n", $2, $3, $4, $6}'
fi

if [ -f paper_trading.log ]; then
    echo ""
    echo "--- Recent Log Output (Last 25 lines) ---"
    tail -n 25 paper_trading.log
else
    echo "No paper_trading.log found yet."
fi
