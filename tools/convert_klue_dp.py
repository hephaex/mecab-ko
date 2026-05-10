#!/usr/bin/env python3
"""Convert KLUE DP dataset to mecab-ko evaluation TSV format.

KLUE DP schema (per example):
    word_form: list[str]   # eojeols (input as-is)
    lemma:     list[str]   # space-separated morphemes per eojeol (already split)
    pos:       list[str]   # +-separated POS tags per eojeol (Sejong-compatible)

Output TSV format (one line per sentence):
    <original sentence>\t<surface1>/<POS1> <surface2>/<POS2> ...

Conversion rules:
- For each eojeol: split lemma on spaces, split pos on '+'.
- If counts mismatch, skip the entire sentence (log to stderr).
- Output morphemes as `surface/POS` pairs, separated by spaces.
- The `surface` here is the lemmatized form (e.g., 흘리 for 흘렸).
  This matches mecab-ko's tokenize output convention.

License: Output derives from KLUE Benchmark (CC BY-SA 4.0). Add attribution
in any file using this output.

Usage:
    source /tmp/klue-env/bin/activate
    python3 tools/convert_klue_dp.py \\
        --split validation \\
        --output data/eval/klue_dp_val.tsv
"""

import argparse
import sys
from pathlib import Path

from datasets import load_dataset


HEADER = """\
# KLUE DP Evaluation Dataset (converted)
#
# Source: KLUE Benchmark (https://huggingface.co/datasets/klue/klue, config=dp)
# License: CC BY-SA 4.0 (https://creativecommons.org/licenses/by-sa/4.0/)
# Citation: Park et al., "KLUE: Korean Language Understanding Evaluation"
#           NeurIPS 2021 Datasets & Benchmarks
#
# Converted by tools/convert_klue_dp.py from KLUE DP {split} split.
# Format: <sentence>\\t<surface1/POS1> <surface2/POS2> ...\\t<eojeol_count1,count2,...>
#
# 3rd column: comma-separated morpheme counts per eojeol, enabling
# eojeol-level evaluation. Optional — parsers that read 2 columns continue
# to work (the 3rd column is ignored for legacy TSVs without it).
#
# Sentences with eojeol-level alignment mismatch (lemma vs POS count differ)
# are skipped. Surface forms are KLUE's lemmatized morphemes, matching
# mecab-ko's tokenize() output convention.
"""


def convert(split: str, output_path: Path) -> tuple[int, int, int]:
    """Convert KLUE DP split to TSV.

    Returns:
        (total_examples, written_count, skipped_count)
    """
    print(f"Loading KLUE DP {split} split...", file=sys.stderr)
    ds = load_dataset("klue/klue", "dp", split=split)
    total = len(ds)
    print(f"Loaded {total} examples", file=sys.stderr)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    written = 0
    skipped = 0

    with output_path.open("w", encoding="utf-8") as out:
        out.write(HEADER.format(split=split))

        for ex in ds:
            sentence = ex["sentence"]
            morpheme_pairs: list[str] = []
            eojeol_counts: list[int] = []
            sentence_ok = True

            for lemma_str, pos_str in zip(ex["lemma"], ex["pos"]):
                morphs = lemma_str.split(" ")
                tags = pos_str.split("+")
                if len(morphs) != len(tags):
                    sentence_ok = False
                    break
                eojeol_counts.append(len(morphs))
                for m, t in zip(morphs, tags):
                    # Skip empty morpheme strings (defensive; should not occur)
                    if not m:
                        sentence_ok = False
                        break
                    morpheme_pairs.append(f"{m}/{t}")
                if not sentence_ok:
                    break

            if not sentence_ok or not morpheme_pairs:
                skipped += 1
                continue

            line = sentence.replace("\t", " ").replace("\n", " ")
            gold = " ".join(morpheme_pairs)
            counts = ",".join(str(c) for c in eojeol_counts)
            out.write(f"{line}\t{gold}\t{counts}\n")
            written += 1

    return total, written, skipped


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--split", default="validation",
                        choices=["train", "validation"])
    parser.add_argument("--output", type=Path, required=True,
                        help="Output TSV path")
    args = parser.parse_args()

    total, written, skipped = convert(args.split, args.output)
    print(f"\nConversion complete:", file=sys.stderr)
    print(f"  Total examples: {total}", file=sys.stderr)
    print(f"  Written:        {written}", file=sys.stderr)
    print(f"  Skipped:        {skipped} ({skipped/total*100:.2f}%)",
          file=sys.stderr)
    print(f"  Output:         {args.output}", file=sys.stderr)


if __name__ == "__main__":
    main()
