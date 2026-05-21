#!/usr/bin/env python3
"""NIKL 모두의말뭉치 형태분석 (Modu Morphological Analysis Corpus) → mecab-ko TSV 변환.

Sprint 159 F: NIKL Modu corpus 통합 (5번째 silver dataset, 구어/SNS 도메인 확장).

NIKL Modu JSON structure (typical):
    {
      "document": [{
        "sentence": [{
          "id": "...",
          "form": "원문 문장",
          "morpheme": [
            {"id": 1, "form": "표면형", "label": "POS태그", "position": 0},
            ...
          ]
        }]
      }]
    }

Output format (tab-separated):
    text<TAB>surface1/POS1 surface2/POS2 ...<TAB>eojeol_counts(comma-separated)

Usage:
    python3 tools/convert_nikl_modu.py \\
        ~/Korpora/NIKL_MP/NXMP1902008051.json \\
        data/eval/nikl_modu_sample.tsv \\
        --max-sentences 5000

License: NIKL 모두의말뭉치는 학술 사용 라이선스 (academic only).
Conversion output은 평가용 (silver dataset)으로만 사용, redistribution 금지.

Note: NIKL Modu는 이미 Sejong-compatible POS tags 사용 (mapping 불필요).
다만 일부 비표준 태그가 있을 수 있으므로 unknown 태그는 stdout에 경고.

References:
- NIKL Modu portal: https://kli.korean.go.kr
- Korpora docs: https://ko-nlp.github.io/Korpora/ko-docs/corpuslist/modu_mp.html
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

# Sejong-compatible POS tags (NIKL Modu uses these natively)
KNOWN_TAGS = {
    # Nouns
    "NNG", "NNP", "NNB", "NP", "NR",
    # Verbs / Adjectives
    "VV", "VA", "VX", "VCP", "VCN",
    # Adnominals
    "MM", "MMD", "MMN", "MMA",
    # Adverbs
    "MAG", "MAJ",
    # Interjection
    "IC",
    # Postpositions (Josa)
    "JKS", "JKC", "JKG", "JKO", "JKB", "JKV", "JKQ", "JX", "JC",
    # Endings
    "EP", "EF", "EC", "ETN", "ETM",
    # Affixes
    "XPN", "XSN", "XSV", "XSA", "XR",
    # Symbols
    "SF", "SP", "SS", "SE", "SO", "SW", "SY", "SH", "SL", "SN",
    "SSO", "SSC", "SC",
}


def convert_morpheme_label(label: str, unknown_counter: Counter) -> str:
    """Normalize POS label. Handles compound tags (X+Y) by splitting."""
    # NIKL Modu may use compound tags like "NNG+JKS" — keep as-is for compound POS
    if "+" in label:
        parts = label.split("+")
        normalized = []
        for p in parts:
            if p not in KNOWN_TAGS:
                unknown_counter[p] += 1
            normalized.append(p)
        return "+".join(normalized)
    if label not in KNOWN_TAGS:
        unknown_counter[label] += 1
    return label


def reconstruct_text(morphemes: list[dict]) -> str:
    """Reconstruct sentence text from morpheme positions.

    NIKL Modu provides absolute positions which let us reconstruct original spacing.
    If positions are missing, fall back to space-joining morphemes.
    """
    if not morphemes:
        return ""
    # Try position-based reconstruction
    if all("position" in m for m in morphemes):
        sorted_morphs = sorted(morphemes, key=lambda m: m["position"])
        text = ""
        prev_end = 0
        for m in sorted_morphs:
            pos = m["position"]
            form = m["form"]
            if pos > prev_end:
                text += " "
            text += form
            prev_end = pos + len(form)
        return text
    # Fallback: simple concat
    return "".join(m["form"] for m in morphemes)


def convert_sentence(sentence: dict, unknown_counter: Counter) -> tuple[str, str, str] | None:
    """Convert NIKL Modu sentence to (text, morphemes_string, eojeol_counts).

    Returns None if sentence is malformed.
    """
    text = sentence.get("form", "")
    morphemes = sentence.get("morpheme", [])
    if not text or not morphemes:
        return None

    # Build morpheme string: "form/label form/label ..."
    morpheme_parts = []
    for m in morphemes:
        form = m.get("form", "")
        label = m.get("label", "")
        if not form or not label:
            continue
        normalized = convert_morpheme_label(label, unknown_counter)
        morpheme_parts.append(f"{form}/{normalized}")

    if not morpheme_parts:
        return None

    morphemes_string = " ".join(morpheme_parts)

    # Eojeol-level morpheme counts (split text by whitespace, then count morphemes per eojeol)
    # Approximation: distribute morphemes evenly if exact alignment is missing
    # (NIKL Modu typically aligns via word_id, but we use position-based heuristic here)
    eojeols = text.split()
    if not eojeols:
        return None
    morphemes_per_eojeol = len(morpheme_parts) // len(eojeols)
    remainder = len(morpheme_parts) % len(eojeols)
    counts = []
    for i in range(len(eojeols)):
        c = morphemes_per_eojeol + (1 if i < remainder else 0)
        if c == 0:
            c = 1  # ensure non-zero
        counts.append(c)
    eojeol_counts = ",".join(str(c) for c in counts)

    return text, morphemes_string, eojeol_counts


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("input", type=Path, help="NIKL Modu JSON file (NXMP*.json)")
    parser.add_argument("output", type=Path, help="Output TSV file")
    parser.add_argument("--max-sentences", type=int, default=5000,
                        help="Maximum sentences to convert (default: 5000)")
    args = parser.parse_args()

    if not args.input.exists():
        print(f"ERROR: Input file not found: {args.input}", file=sys.stderr)
        print("\nNIKL Modu 다운로드 방법:", file=sys.stderr)
        print("1. https://kli.korean.go.kr 에서 학술 사용 등록", file=sys.stderr)
        print("2. '모두의말뭉치 형태분석' 선택 후 다운로드", file=sys.stderr)
        print("3. JSON 파일을 input 경로에 배치", file=sys.stderr)
        return 1

    print(f"Loading {args.input}...")
    with args.input.open(encoding="utf-8") as f:
        data = json.load(f)

    sentences = []
    # Handle different NIKL Modu JSON structures
    if "document" in data:
        for doc in data["document"]:
            sentences.extend(doc.get("sentence", []))
    elif "sentence" in data:
        sentences.extend(data["sentence"]) if isinstance(data["sentence"], list) else [data["sentence"]]
    else:
        print(f"ERROR: Unrecognized JSON structure (no 'document' or 'sentence' key)", file=sys.stderr)
        return 1

    print(f"Found {len(sentences)} sentences")
    if args.max_sentences > 0:
        sentences = sentences[: args.max_sentences]
        print(f"Limiting to {args.max_sentences} sentences")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    unknown_counter: Counter = Counter()
    converted = 0
    skipped = 0

    with args.output.open("w", encoding="utf-8") as f:
        for s in sentences:
            result = convert_sentence(s, unknown_counter)
            if result is None:
                skipped += 1
                continue
            text, morphemes_string, eojeol_counts = result
            f.write(f"{text}\t{morphemes_string}\t{eojeol_counts}\n")
            converted += 1

    print(f"\n✓ Converted {converted} sentences ({skipped} skipped)")
    print(f"✓ Output: {args.output}")

    if unknown_counter:
        print(f"\n⚠ Unknown tags ({len(unknown_counter)}):")
        for tag, count in unknown_counter.most_common(20):
            print(f"  {tag}: {count}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
