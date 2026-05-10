#!/usr/bin/env python3
"""Inspect KLUE DP dataset format for mecab-ko evaluation integration.

Usage:
    source /tmp/klue-env/bin/activate
    python3 tools/inspect_klue_dp.py [--samples N]

Outputs:
    1. First N example structures
    2. POS tag distribution (single + compound)
    3. Compound tag depth distribution (1, 2, 3+ morphemes per eojeol)
    4. Sample sentences with eojeol-level annotation
"""

import argparse
from collections import Counter
from datasets import load_dataset


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--samples", type=int, default=10, help="Sample sentences to print")
    args = parser.parse_args()

    print("Loading KLUE DP val split (this may download)...")
    ds = load_dataset("klue/klue", "dp", split="validation")
    print(f"Loaded {len(ds)} examples\n")

    # Inspect first example structure
    print("=" * 70)
    print("FIRST EXAMPLE — schema inspection")
    print("=" * 70)
    ex = ds[0]
    print(f"Keys: {list(ex.keys())}")
    for k, v in ex.items():
        if isinstance(v, list):
            preview = v[:3] if len(v) > 3 else v
            print(f"  {k}: list({len(v)}) — first: {preview}")
        else:
            print(f"  {k}: {type(v).__name__} — {str(v)[:80]}")
    print()

    # POS tag analysis
    pos_counter: Counter[str] = Counter()
    compound_depth: Counter[int] = Counter()
    sentence_lengths = []
    eojeol_lengths = []

    align_mismatches = 0
    eojeols_total = 0
    for ex in ds:
        # KLUE DP schema: lemma (space-separated morphemes per eojeol),
        #                 pos (+-separated POS tags per eojeol)
        for lemma_str, pos_str in zip(ex["lemma"], ex["pos"]):
            morphs = lemma_str.split(" ")
            tags = pos_str.split("+")
            eojeols_total += 1
            if len(morphs) != len(tags):
                align_mismatches += 1
                continue
            compound_depth[len(tags)] += 1
            for tag in tags:
                pos_counter[tag] += 1

        sentence_lengths.append(len(ex["word_form"]))
        for w in ex["word_form"]:
            eojeol_lengths.append(len(w))

    print(f"Eojeol count: {eojeols_total}, align mismatches: {align_mismatches}")

    print("=" * 70)
    print("POS TAG DISTRIBUTION (top 30)")
    print("=" * 70)
    for pos, cnt in pos_counter.most_common(30):
        print(f"  {pos:<6} {cnt:>7}")
    print(f"  ... total {len(pos_counter)} unique POS tags")
    print()

    print("=" * 70)
    print("COMPOUND TAG DEPTH (morphemes per eojeol)")
    print("=" * 70)
    total_eojeols = sum(compound_depth.values())
    for depth in sorted(compound_depth.keys()):
        cnt = compound_depth[depth]
        pct = cnt / total_eojeols * 100 if total_eojeols > 0 else 0
        print(f"  {depth} morphemes: {cnt:>7} ({pct:.1f}%)")
    print()

    if sentence_lengths:
        print(f"Sentence length: min={min(sentence_lengths)} "
              f"max={max(sentence_lengths)} "
              f"avg={sum(sentence_lengths)/len(sentence_lengths):.1f}")
        print(f"Eojeol length: min={min(eojeol_lengths)} "
              f"max={max(eojeol_lengths)} "
              f"avg={sum(eojeol_lengths)/len(eojeol_lengths):.1f}")
    print()

    # Sample sentences
    print("=" * 70)
    print(f"SAMPLE SENTENCES (first {args.samples})")
    print("=" * 70)
    for i in range(min(args.samples, len(ds))):
        ex = ds[i]
        print(f"\n--- Example {i + 1} ---")
        if "sentence" in ex:
            print(f"  sentence: {ex['sentence']}")
        print("  word_form    →  lemma          |  pos")
        for w, l, p in zip(ex["word_form"], ex["lemma"], ex["pos"]):
            print(f"    {w:<12} →  {l:<14} |  {p}")


if __name__ == "__main__":
    main()
