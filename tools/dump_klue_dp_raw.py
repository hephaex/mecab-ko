#!/usr/bin/env python3
"""Dump raw KLUE DP data to JSONL for reproducibility.

Saves the full HuggingFace KLUE DP val split as JSONL so the repo doesn't
depend on HF availability. Includes ALL fields (sentence, word_form, lemma,
pos, head, deprel, index) for completeness.

License: KLUE Benchmark CC BY-SA 4.0 — see data/raw/klue/LICENSE-KLUE.md
"""

import argparse
import json
import sys
from pathlib import Path

from datasets import load_dataset


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--split", default="validation",
                        choices=["train", "validation"])
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    print(f"Loading KLUE DP {args.split} split...", file=sys.stderr)
    ds = load_dataset("klue/klue", "dp", split=args.split)
    print(f"Loaded {len(ds)} examples", file=sys.stderr)

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", encoding="utf-8") as f:
        for ex in ds:
            f.write(json.dumps(ex, ensure_ascii=False) + "\n")

    size_kb = args.output.stat().st_size / 1024
    print(f"Wrote {len(ds)} examples ({size_kb:.1f} KiB) to {args.output}",
          file=sys.stderr)


if __name__ == "__main__":
    main()
