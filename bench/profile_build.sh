#!/usr/bin/env bash
#
# Profiling harness for the Bower build.
#
# For each corpus size, generates a synthetic post corpus, runs the
# instrumented `bower` binary REPEATS times (plus one discarded warmup), parses
# the per-phase BOWER_PROFILE_JSON line emitted to stderr, and appends one row
# per run to a CSV dataset. The result is a small dataset we can analyze to see
# which build phases dominate and how each scales with post count.
#
# Usage:
#   bench/profile_build.sh [--sizes "1 10 50 100 250 500"] \
#                          [--repeats 5] [--code-blocks 2] \
#                          [--out bench/results/build_perf.csv]
#
# Requires: cargo (release binary), python3.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SIZES="1 10 50 100 250 500"
REPEATS=5
CODE_BLOCKS=2
CODE_FRACTION=1.0
LABELED_FRACTION=1.0
OUT="$REPO_ROOT/bench/results/build_perf.csv"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --sizes) SIZES="$2"; shift 2 ;;
    --repeats) REPEATS="$2"; shift 2 ;;
    --code-blocks) CODE_BLOCKS="$2"; shift 2 ;;
    --code-fraction) CODE_FRACTION="$2"; shift 2 ;;
    --labeled-fraction) LABELED_FRACTION="$2"; shift 2 ;;
    --out) OUT="$2"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

BIN="$REPO_ROOT/target/release/bower"
GEN="$REPO_ROOT/bench/gen_posts.py"
SITE_SCM="$REPO_ROOT/example/site.scm"

echo "Building release binary..." >&2
cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml" >&2

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

mkdir -p "$(dirname "$OUT")"
echo "run,post_count,code_blocks,total_ms,setup_ms,parse_ms,render_posts_ms,render_index_ms,rss_ms,sitemap_ms,assets_ms" > "$OUT"

# Extract a numeric field from the JSON blob using python (robust vs. field order).
field() { python3 -c "import json,sys;print(json.loads(sys.argv[1]).get(sys.argv[2],''))" "$1" "$2"; }

for size in $SIZES; do
  echo "== size=$size ==" >&2
  CORPUS="$WORK/corpus-$size"
  rm -rf "$CORPUS"
  mkdir -p "$CORPUS"
  cp "$SITE_SCM" "$CORPUS/site.scm"
  python3 "$GEN" "$CORPUS" "$size" --code-blocks "$CODE_BLOCKS" \
    --code-fraction "$CODE_FRACTION" --labeled-fraction "$LABELED_FRACTION"

  # +1 warmup run (discarded) to prime filesystem/OS caches.
  for run in $(seq 0 "$REPEATS"); do
    rm -rf "$CORPUS/build"
    json="$(cd "$CORPUS" && BOWER_PROFILE=1 "$BIN" 2>&1 >/dev/null | grep '^BOWER_PROFILE_JSON ' | sed 's/^BOWER_PROFILE_JSON //')"
    if [[ -z "$json" ]]; then
      echo "  run $run: no profile output!" >&2
      continue
    fi
    [[ "$run" -eq 0 ]] && continue  # warmup

    printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
      "$run" \
      "$(field "$json" post_count)" \
      "$CODE_BLOCKS" \
      "$(field "$json" total_ms)" \
      "$(field "$json" setup)" \
      "$(field "$json" parse)" \
      "$(field "$json" render_posts)" \
      "$(field "$json" render_index)" \
      "$(field "$json" rss)" \
      "$(field "$json" sitemap)" \
      "$(field "$json" assets)" >> "$OUT"
    echo "  run $run: total=$(field "$json" total_ms)ms" >&2
  done
done

echo "" >&2
echo "Dataset written to: $OUT" >&2
