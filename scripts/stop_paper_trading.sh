#!/usr/bin/env bash

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

if [ -f arb-bot.pid ]; then
    PID=$(cat arb-bot.pid)
    if kill -0 "$PID" 2>/dev/null; then
        echo "Stopping arb-bot (PID: $PID)..."
        kill "$PID"
        sleep 2
        if kill -0 "$PID" 2>/dev/null; then
            kill -9 "$PID"
        fi
        echo "arb-bot stopped."
    else
        echo "Process $PID is not running."
    fi
    rm -f arb-bot.pid
else
    # Fallback to pkill if pidfile is missing
    if pgrep -f "arb-bot -c config.toml" > /dev/null; then
        echo "Stopping arb-bot via pkill..."
        pkill -f "arb-bot -c config.toml"
        echo "arb-bot stopped."
    else
        echo "arb-bot is not running."
    fi
fi
