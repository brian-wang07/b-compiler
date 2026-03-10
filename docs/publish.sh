#!/usr/bin/env bash
#
# publish.sh — Copy benchmark outputs into docs/ for GitHub Pages.
#
# Usage:
#   ./docs/publish.sh
#
# Run this after `cargo bench` and `cargo bench --bench memory_bench`
# to refresh the published reports.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCS_DIR="$SCRIPT_DIR"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
B_DIR="$PROJECT_DIR/b"
CRITERION_DIR="$B_DIR/target/criterion"
DHAT_FILE="$B_DIR/dhat-heap.json"

# --- Criterion reports ---
BENCH_DIR="$DOCS_DIR/benchmarks"
rm -rf "$BENCH_DIR"
mkdir -p "$BENCH_DIR"

if [[ -d "$CRITERION_DIR" ]]; then
    # Copy the top-level report index
    if [[ -d "$CRITERION_DIR/report" ]]; then
        cp -r "$CRITERION_DIR/report/"* "$BENCH_DIR/"
    fi

    # Copy each individual benchmark's report
    find "$CRITERION_DIR" -mindepth 1 -maxdepth 1 -type d ! -name report | while read -r bench_dir; do
        bench_name="$(basename "$bench_dir")"
        if [[ -d "$bench_dir/report" ]]; then
            mkdir -p "$BENCH_DIR/$bench_name"
            cp -r "$bench_dir/report/"* "$BENCH_DIR/$bench_name/"
        fi
    done
    echo "✓ Criterion reports copied to docs/benchmarks/"
else
    echo "⚠ No Criterion output found at $CRITERION_DIR"
    echo "  Run: cd b && cargo bench --bench lexer_bench && cargo bench --bench parser_bench"
fi

# --- dhat heap profile ---
MEMORY_DIR="$DOCS_DIR/memory"
rm -rf "$MEMORY_DIR"
mkdir -p "$MEMORY_DIR"

if [[ -f "$DHAT_FILE" ]]; then
    cp "$DHAT_FILE" "$MEMORY_DIR/dhat-heap.json"
    echo "✓ dhat-heap.json copied to docs/memory/"
else
    echo "⚠ No dhat-heap.json found at $DHAT_FILE"
    echo "  Run: cd b && cargo bench --bench memory_bench"
fi

echo ""
echo "Done. Commit and push to update GitHub Pages."
