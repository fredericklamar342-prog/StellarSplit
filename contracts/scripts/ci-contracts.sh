#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONTRACTS_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# All production-ready contracts should be listed here.
SUPPORTED_CONTRACTS=(
  "achievement-badges"
  "dispute-resolution"
  "flash-loan"
  "multi-sig-splits"
  "path-payment"
  "split-escrow"
  "split-template"
  "staking"
)

COMMAND="${1:-all}"

run_for_contract() {
  local contract="$1"
  local manifest="$CONTRACTS_DIR/$contract/Cargo.toml"

  echo ""
  echo "==> $COMMAND :: $contract"

  case "$COMMAND" in
    fmt)
      cargo fmt --manifest-path "$manifest" --all -- --check
      ;;
    test)
      cargo test --manifest-path "$manifest"
      ;;
    build)
      cargo build --manifest-path "$manifest" --target wasm32-unknown-unknown --release
      ;;
    *)
      echo "Unsupported command: $COMMAND (expected fmt, test, build, or all)"
      echo "Usage: bash ./scripts/ci-contracts.sh [fmt|test|build|all]"
      exit 1
      ;;
  esac
}

if [ "$COMMAND" = "all" ]; then
  # Run fmt, test, and build sequentially for all supported contracts
  for step in fmt test build; do
    COMMAND="$step"
    for contract in "${SUPPORTED_CONTRACTS[@]}"; do
      run_for_contract "$contract"
    done
  done
  COMMAND="all"
else
  for contract in "${SUPPORTED_CONTRACTS[@]}"; do
    run_for_contract "$contract"
  done
fi

echo ""
echo "All contract checks completed for: ${SUPPORTED_CONTRACTS[*]}"
