#!/usr/bin/env python3
"""Generate a synthetic Bower post corpus for benchmarking.

Produces `count` markdown posts under `<out_dir>/posts/`, each with YAML front
matter and a realistic body: several prose paragraphs plus a configurable number
of fenced code blocks (syntect highlighting is the suspected build hot path, so
we want to control how much code there is). Determinstic given the seed so runs
are comparable.

Usage: gen_posts.py <out_dir> <count> [--code-blocks N] [--seed S]
"""
import argparse
import os
import random
import sys

LANGS = ["rust", "python", "javascript", "c", "bash"]

# Snippets use the literal placeholder __N__ (replaced via str.replace, not
# str.format) to avoid clashing with the many literal braces in the code.
CODE_SNIPPETS = {
    "rust": 'fn main() {\n    let xs: Vec<i32> = (0..__N__).collect();\n    let sum: i32 = xs.iter().filter(|x| *x % 2 == 0).sum();\n    println!("sum = {}", sum);\n}',
    "python": 'def fib(n):\n    a, b = 0, 1\n    for _ in range(__N__):\n        a, b = b, a + b\n    return a\n\nprint([fib(i) for i in range(10)])',
    "javascript": 'const items = Array.from({ length: __N__ }, (_, i) => i);\nconst total = items.reduce((acc, x) => acc + x * 2, 0);\nconsole.log(`total=${total}`);',
    "c": '#include <stdio.h>\nint main(void) {\n    long acc = 0;\n    for (int i = 0; i < __N__; i++) acc += i;\n    printf("%ld\\n", acc);\n    return 0;\n}',
    "bash": 'for i in $(seq 1 __N__); do\n  echo "line $i"\ndone | sort -r | head',
}

PROSE = (
    "The quick brown fox jumps over the lazy dog. Static site generators trade "
    "runtime rendering for build-time work, so understanding where that work "
    "goes matters. This paragraph exists purely to give the markdown parser "
    "something representative to chew on, with **bold**, _italic_, and a "
    "[link](https://example.com) thrown in for good measure."
)


def make_post(
    idx: int,
    has_code: bool,
    labeled: bool,
    code_blocks: int,
    rng: random.Random,
) -> str:
    """Build one post. `has_code` decides whether it contains any fenced code;
    `labeled` decides whether that fence carries a language hint (only labeled
    fences trigger syntect highlighting in the build). `code_blocks` is how many
    blocks a code-bearing post gets."""
    year = 2000 + (idx % 25)
    lines = [
        "---",
        f'title: "Synthetic Post {idx}"',
        f"date: {year:04d}-01-15T05:23:14+00:00",
        "---",
        "",
        f"# Synthetic Post {idx}",
        "",
    ]
    blocks_remaining = code_blocks if has_code else 0
    paragraphs = 3 + rng.randint(0, 3)
    for _ in range(paragraphs):
        lines.append(PROSE)
        lines.append("")
        if blocks_remaining > 0:
            blocks_remaining -= 1
            lang = rng.choice(LANGS)
            snippet = CODE_SNIPPETS[lang].replace("__N__", str(rng.randint(5, 100)))
            # Labeled fences (```rust) hit syntect; bare fences (```) don't.
            lines.append(f"```{lang}" if labeled else "```")
            lines.append(snippet)
            lines.append("```")
            lines.append("")
    lines.append("- item one")
    lines.append("- item two")
    lines.append("- item three")
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("out_dir")
    ap.add_argument("count", type=int)
    ap.add_argument("--code-blocks", type=int, default=2,
                    help="code blocks in a post that has code")
    ap.add_argument("--code-fraction", type=float, default=1.0,
                    help="fraction of posts that contain any code (0..1)")
    ap.add_argument("--labeled-fraction", type=float, default=1.0,
                    help="of code-bearing posts, fraction whose fences carry a "
                         "language hint (only these trigger syntect)")
    ap.add_argument("--seed", type=int, default=42)
    args = ap.parse_args()

    posts_dir = os.path.join(args.out_dir, "posts")
    os.makedirs(posts_dir, exist_ok=True)
    rng = random.Random(args.seed)
    for i in range(args.count):
        has_code = rng.random() < args.code_fraction
        labeled = has_code and rng.random() < args.labeled_fraction
        path = os.path.join(posts_dir, f"post-{i:05d}.md")
        with open(path, "w") as f:
            f.write(make_post(i, has_code, labeled, args.code_blocks, rng))
    return 0


if __name__ == "__main__":
    sys.exit(main())
