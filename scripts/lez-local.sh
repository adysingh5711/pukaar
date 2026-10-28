#!/usr/bin/env bash
# Local LEZ v0.3.1 sequencer (standalone, dev mode). Usage: lez-local.sh start|stop|status   (start: LEZ_BLOCK_TIMEOUT, default 2s)
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
    # A copy of the debug config with a faster block time (the clone's tracked config says 15s). The data dir is the same.
    CFG=$LEZ_HOMES/sequencer_config_fast.json
    sed "s/\"block_create_timeout\": \"[0-9a-z]*\"/\"block_create_timeout\": \"${LEZ_BLOCK_TIMEOUT:-2s}\"/" \
      "$LEZ/lez/sequencer/service/configs/debug/sequencer_config.json" >"$CFG"
    cd "$LEZ"
    [ -x target/release/sequencer_service ] || cargo build --features standalone --release -p sequencer_service
    cd lez/sequencer/service
    RISC0_DEV_MODE=1 RUST_LOG=info,kameo=warn nohup "$LEZ/target/release/sequencer_service" "$CFG" >"$LOG" 2>&1 &
    echo $! >"$PID"; echo "started pid $(cat "$PID"), block time ${LEZ_BLOCK_TIMEOUT:-2s}, log $LOG (a restart replays the chain: wait for status)" ;;
  stop)
    # cargo run spawns the real sequencer as a child: kill the whole tree
    pkill -P "$(cat "$PID")" 2>/dev/null || true
    pkill -f 'sequencer_service.*sequencer_config.json' 2>/dev/null || true
    kill "$(cat "$PID")" 2>/dev/null || true; rm -f "$PID"; echo stopped ;;
  status) wallet check-health ;;
  *) echo "usage: $0 start|stop|status" >&2; exit 2 ;;
esac
