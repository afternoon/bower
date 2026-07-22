# Build performance harness

Tools for profiling the Bower build to identify bottlenecks from data.

## Components

- **Phase instrumentation** (in `src/main.rs`): when the `BOWER_PROFILE` env var
  is set, the build emits a `BOWER_PROFILE_JSON {...}` line to stderr with
  per-phase wall-clock timings (`setup`, `parse`, `render_posts`,
  `render_index`, `rss`, `sitemap`, `assets`). Unprofiled runs are unaffected.
- **`gen_posts.py`** — generates a synthetic post corpus of a given size, with a
  configurable number of fenced code blocks per post (deterministic given the
  seed). Code density is a knob because syntect highlighting is the suspected
  hot path.
- **`profile_build.sh`** — for each corpus size, generates a corpus, runs the
  instrumented binary `--repeats` times (plus a discarded warmup), and writes
  one CSV row per run to `bench/results/build_perf.csv`.
- **`analyze.py`** — summarizes the CSV: median per-phase durations by size,
  each phase's share of total, and a linear fit separating fixed startup cost
  from per-post cost.

## Usage

```sh
# Collect the dataset (default sizes: 1 10 50 100 250 500, 5 repeats each)
bench/profile_build.sh

# Custom sweep
bench/profile_build.sh --sizes "10 100 1000" --repeats 8 --code-blocks 3

# Summarize
python3 bench/analyze.py
```

Manual single-run profile:

```sh
cd example && BOWER_PROFILE=1 ../target/release/bower 2>&1 | grep PROFILE
```

## Results (see `results/summary.txt`)

On this machine, **syntect syntax highlighting dominates the build** — it runs
single-threaded inside the serial parse loop and accounts for ~75-85% of total
build time on code-heavy corpora (parse time is ~33x higher with 2 code
blocks/post than with 0). Everything except `parse`, `render_posts`, and the
fixed `setup` cost is negligible. See `results/summary.txt` for the numbers.
