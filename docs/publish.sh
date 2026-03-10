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

# Slugify: replace spaces with _ and strip parentheses
slugify() {
    echo "$1" | tr ' ' '_' | tr -d '()'
}

# --- Criterion reports ---
BENCH_DIR="$DOCS_DIR/benchmarks"
rm -rf "$BENCH_DIR"
mkdir -p "$BENCH_DIR"

if [[ -d "$CRITERION_DIR" ]]; then
    # Copy the top-level report index (preserve report/ subdirectory
    # so that Criterion's ../name/report/ relative links work)
    if [[ -d "$CRITERION_DIR/report" ]]; then
        mkdir -p "$BENCH_DIR/report"
        cp -r "$CRITERION_DIR/report/"* "$BENCH_DIR/report/"
    fi

    # Copy each individual benchmark, preserving the report/ subdirectory
    # and renaming to URL-safe slug names
    find "$CRITERION_DIR" -mindepth 1 -maxdepth 1 -type d ! -name report | while read -r bench_dir; do
        bench_name="$(basename "$bench_dir")"
        slug="$(slugify "$bench_name")"
        if [[ -d "$bench_dir/report" ]]; then
            mkdir -p "$BENCH_DIR/$slug/report"
            cp -r "$bench_dir/report/"* "$BENCH_DIR/$slug/report/"
        fi
    done

    # Rewrite the overview index.html links to use slugified names
    if [[ -f "$BENCH_DIR/report/index.html" ]]; then
        cp "$BENCH_DIR/report/index.html" "$BENCH_DIR/report/index.html.tmp"
        find "$CRITERION_DIR" -mindepth 1 -maxdepth 1 -type d ! -name report | while read -r bench_dir; do
            bench_name="$(basename "$bench_dir")"
            slug="$(slugify "$bench_name")"
            if [[ "$bench_name" != "$slug" ]]; then
                # Use perl for literal (non-regex) string replacement
                perl -pi -e "s/\Q${bench_name}\E/${slug}/g" "$BENCH_DIR/report/index.html.tmp"
            fi
        done
        mv "$BENCH_DIR/report/index.html.tmp" "$BENCH_DIR/report/index.html"
    fi

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
