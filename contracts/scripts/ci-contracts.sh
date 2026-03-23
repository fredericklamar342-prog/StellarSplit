#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONTRACTS_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# `split-escrow` is intentionally excluded for now because the Rust source in
# `contracts/split-escrow/src/lib.rs` is draft/broken and does not parse.
# `dispute-resolution` is also excluded because the contract/test API is still
# mid-port and does not compile under the pinned Soroban toolchain yet.
# `multi-sig-splits`, `path-payment`, `split-template`, and `staking` are
# excluded until they are verified clean under the pinned Soroban toolchain.
SUPPORTED_CONTRACTS=(
  "achievement-badges"
  "flash-loan"
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
      echo "Unsupported command: $COMMAND"
      echo "Usage: bash ./scripts/ci-contracts.sh [fmt|test|build]"
      exit 1
      ;;
  esac
}

for contract in "${SUPPORTED_CONTRACTS[@]}"; do
  run_for_contract "$contract"
done

echo ""
echo "All contract checks completed for: ${SUPPORTED_CONTRACTS[*]}"
