#!/usr/bin/env bash
#
# gen_versions_index.sh — Generate an index.html for the versions directory.
#
# Usage:
#   ./docs/gen_versions_index.sh <versions-dir>
#
# Scans <versions-dir> for version subdirectories (v*) and writes
# an index.html with links to each version's Criterion overview.

set -euo pipefail

VERSIONS_DIR="${1:?Usage: gen_versions_index.sh <versions-dir>}"
OUTPUT="$VERSIONS_DIR/index.html"

cat > "$OUTPUT" << 'HEAD'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Version History — B Compiler Benchmarks</title>
    <style>
        :root {
            --bg: #0d1117;
            --surface: #161b22;
            --border: #30363d;
            --text: #e6edf3;
            --text-muted: #8b949e;
            --accent: #58a6ff;
            --accent-hover: #79c0ff;
        }
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
            background: var(--bg);
            color: var(--text);
            line-height: 1.6;
            padding: 2rem;
            max-width: 900px;
            margin: 0 auto;
        }
        h1 { margin-bottom: 0.5rem; font-size: 1.75rem; }
        .subtitle { color: var(--text-muted); margin-bottom: 2rem; }
        a { color: var(--accent); text-decoration: none; }
        a:hover { color: var(--accent-hover); text-decoration: underline; }
        table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 1rem;
        }
        th, td {
            text-align: left;
            padding: 0.6rem 1rem;
            border-bottom: 1px solid var(--border);
        }
        th {
            color: var(--text-muted);
            font-size: 0.85rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }
        td { font-size: 0.95rem; }
        tr:hover td { background: var(--surface); }
        .back {
            display: inline-block;
            margin-top: 2rem;
            color: var(--text-muted);
            font-size: 0.9rem;
        }
        .empty {
            color: var(--text-muted);
            font-style: italic;
            margin-top: 1rem;
        }
    </style>
</head>
<body>
    <h1>Version History</h1>
    <p class="subtitle">Criterion benchmark reports archived for each tagged release. Click a version to view its full report.</p>
HEAD

# Collect version directories (newest first via version sort)
VERSIONS=()
for dir in "$VERSIONS_DIR"/v*/; do
    [[ -d "$dir" ]] && VERSIONS+=("$(basename "$dir")")
done

if [[ ${#VERSIONS[@]} -eq 0 ]]; then
    echo '    <p class="empty">No tagged versions have been benchmarked yet.</p>' >> "$OUTPUT"
else
    # Sort versions in reverse order
    IFS=$'\n' SORTED=($(printf '%s\n' "${VERSIONS[@]}" | sort -rV)); unset IFS

    cat >> "$OUTPUT" << 'TABLE_HEAD'
    <table>
        <thead>
            <tr><th>Version</th><th>Compared Against</th><th>Report</th></tr>
        </thead>
        <tbody>
TABLE_HEAD

    for tag in "${SORTED[@]}"; do
        baseline="—"
        if [[ -f "$VERSIONS_DIR/$tag/.baseline" ]]; then
            bl="$(cat "$VERSIONS_DIR/$tag/.baseline")"
            [[ -n "$bl" ]] && baseline="$bl"
        fi
        cat >> "$OUTPUT" << ROW
            <tr>
                <td><strong>${tag}</strong></td>
                <td>${baseline}</td>
                <td><a href="${tag}/report/index.html">View Report →</a></td>
            </tr>
ROW
    done

    cat >> "$OUTPUT" << 'TABLE_FOOT'
        </tbody>
    </table>
TABLE_FOOT
fi

cat >> "$OUTPUT" << 'FOOT'
    <a class="back" href="../">← Back to main page</a>
</body>
</html>
FOOT

echo "✓ Generated versions index at $OUTPUT"
