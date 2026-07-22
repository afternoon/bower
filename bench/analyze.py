#!/usr/bin/env python3
"""Summarize the build-performance dataset produced by profile_build.sh.

Reports, per corpus size, the median duration of each build phase and its share
of the median total. Also fits a rough per-post cost (slope of phase-time vs.
post-count) so we can separate fixed startup cost from work that scales.

Usage: analyze.py [bench/results/build_perf.csv]
"""
import csv
import statistics
import sys

PHASES = [
    "setup_ms", "parse_ms", "render_posts_ms", "render_index_ms",
    "rss_ms", "sitemap_ms", "assets_ms",
]


def main() -> int:
    path = sys.argv[1] if len(sys.argv) > 1 else "bench/results/build_perf.csv"
    rows = list(csv.DictReader(open(path)))
    if not rows:
        print("no data")
        return 1

    by_size = {}
    for r in rows:
        by_size.setdefault(int(r["post_count"]), []).append(r)

    sizes = sorted(by_size)
    med = {}  # size -> phase -> median ms
    for size in sizes:
        group = by_size[size]
        med[size] = {p: statistics.median(float(r[p]) for r in group) for p in PHASES}
        med[size]["total_ms"] = statistics.median(float(r["total_ms"]) for r in group)

    print("=== Median phase durations (ms) by post count ===\n")
    header = f"{'phase':<18}" + "".join(f"{s:>12}" for s in sizes)
    print(header)
    print("-" * len(header))
    for p in PHASES + ["total_ms"]:
        line = f"{p:<18}" + "".join(f"{med[s][p]:>12.2f}" for s in sizes)
        print(line)

    print("\n=== Phase share of total at largest corpus (n=%d) ===\n" % sizes[-1])
    big = med[sizes[-1]]
    for p in sorted(PHASES, key=lambda p: -big[p]):
        share = 100.0 * big[p] / big["total_ms"] if big["total_ms"] else 0
        print(f"  {p:<18} {big[p]:>9.2f} ms  ({share:4.1f}%)")

    print("\n=== Scaling: fixed cost vs. per-post cost (linear fit) ===\n")
    n = [float(s) for s in sizes]
    for p in PHASES + ["total_ms"]:
        y = [med[s][p] for s in sizes]
        slope, intercept = _linfit(n, y)
        print(f"  {p:<18} fixed ~{intercept:8.2f} ms  +  {slope*1000:7.2f} us/post")
    return 0


def _linfit(x, y):
    mx = statistics.mean(x)
    my = statistics.mean(y)
    denom = sum((xi - mx) ** 2 for xi in x)
    if denom == 0:
        return 0.0, my
    slope = sum((xi - mx) * (yi - my) for xi, yi in zip(x, y)) / denom
    return slope, my - slope * mx


if __name__ == "__main__":
    sys.exit(main())
