#!/usr/bin/env bash
# Local LEZ v0.3.1 sequencer (standalone, dev mode). Usage: lez-local.sh start|stop|status
# Local wallet:   LEE_WALLET_HOME_DIR=$LEZ_HOMES/local    (sequencer http://127.0.0.1:3040)
# Testnet wallet: LEE_WALLET_HOME_DIR=$LEZ_HOMES/testnet  (never mix the two)
set -euo pipefail
LEZ=${LEZ:-/Users/aditya/Dev/Hackathons/logos-works/logos-execution-zone}
export LEZ_HOMES=${LEZ_HOMES:-/Users/aditya/Dev/Hackathons/logos-works/wallet-homes}
LOG=${LEZ_LOG:-/tmp/lez-sequencer.log}
PID=$LEZ_HOMES/sequencer.pid
export LEE_WALLET_HOME_DIR=$LEZ_HOMES/local

case "${1:-}" in
  start)
    cd "$LEZ"
    RISC0_DEV_MODE=1 nohup just run-sequencer-standalone >"$LOG" 2>&1 &
    echo $! >"$PID"; echo "started pid $(cat "$PID"), log $LOG" ;;
  stop)
    # cargo run spawns the real sequencer as a child: kill the whole tree
    pkill -P "$(cat "$PID")" 2>/dev/null || true
    pkill -f 'sequencer_service.*sequencer_config.json' 2>/dev/null || true
    kill "$(cat "$PID")" 2>/dev/null || true; rm -f "$PID"; echo stopped ;;
  status) wallet check-health ;;
  *) echo "usage: $0 start|stop|status" >&2; exit 2 ;;
esac
