#!/usr/bin/env bash
#
# perf_bench.sh — Run `perf stat` on the memory_bench binary to capture
# hardware-level counters (cycles, instructions, cache misses, branch
# misses, etc.).
#
# Usage:
#   ./benches/perf_bench.sh [scenario]
#
# Scenarios (passed to memory_bench): lex, parse, deep_expr, strings,
# globals, all (default).
#
# Prerequisites:
#   - Linux with perf installed (linux-tools-common / linux-tools-generic)
#   - Build first: cargo build --release --bench memory_bench
#
# If you get "permission denied" for perf events, run:
#   sudo sysctl -w kernel.perf_event_paranoid=1
# or run this script under sudo.

set -euo pipefail

SCENARIO="${1:-all}"
BENCH_BIN="target/release/memory_bench"

# Locate the project root (parent of benches/).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_DIR"

echo ">>> Building memory_bench in release mode..."
cargo build --release --bench memory_bench 2>&1

# Find the actual binary — cargo puts bench binaries in a deps path.
ACTUAL_BIN=$(find target/release/deps -name 'memory_bench-*' -executable -type f | head -n 1)
if [[ -z "$ACTUAL_BIN" ]]; then
    echo "ERROR: could not find compiled memory_bench binary."
    exit 1
fi

echo ""
echo ">>> Running perf stat — scenario: $SCENARIO"
echo "    binary: $ACTUAL_BIN"
echo ""

perf stat \
    -e cycles,instructions,cache-references,cache-misses,branches,branch-misses,task-clock,page-faults \
    "$ACTUAL_BIN" "$SCENARIO"
