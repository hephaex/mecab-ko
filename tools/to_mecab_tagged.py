#!/usr/bin/env python3
"""UD CoNLL-U → mecab `.tagged` 학습 형식 변환.

Sprint 165 B-1: Track B Step 2 — CRF retrain 학습 데이터 준비.

Input: UD Korean CoNLL-U file (ko_kaist-ud-dev.conllu or ko_gsd-ud-dev.conllu)
Output: mecab .tagged file (학습용)

Format:
    surface<TAB>POS,*,*,*,*,*,*,*,*
    surface<TAB>POS,*,*,*,*,*,*,*,*
    ...
    EOS
    surface<TAB>...

각 morpheme이 한 줄 (lemma + xpos를 +로 split하여 분해).
mecab features는 POS만 정확, 나머지는 `*` (wildcard, optional features in feature.def).

Usage:
    python3 tools/to_mecab_tagged.py \\
        data/raw/ud_kaist/ko_kaist-ud-dev.conllu \\
        data/train/ud_kaist_dev.tagged --xpos-map kaist

    python3 tools/to_mecab_tagged.py \\
        data/raw/ud_gsd/ko_gsd-ud-dev.conllu \\
        data/train/ud_gsd_dev.tagged --xpos-map gsd

License: UD Korean-Kaist/GSD는 CC BY-SA 4.0. 학습 산출물도 동일 라이선스.
"""
from __future__ import annotations

import argparse
import sys
from collections import Counter
from pathlib import Path

# Sejong POS tags (mecab-ko-dic 호환)
SEJONG_TAGS = {
    "NNG", "NNP", "NNB", "NP", "NR",
    "VV", "VA", "VX", "VCP", "VCN",
    "MM", "MMD", "MMN", "MMA",
    "MAG", "MAJ",
    "IC",
    "JKS", "JKC", "JKG", "JKO", "JKB", "JKV", "JKQ", "JX", "JC",
    "EP", "EF", "EC", "ETN", "ETM",
    "XPN", "XSN", "XSV", "XSA", "XR",
    "SF", "SP", "SS", "SE", "SO", "SW", "SY", "SH", "SL", "SN",
    "SSO", "SSC", "SC",
}

# KAIST XPOS → Sejong (Sprint 139 P2 convert_ud_kaist.py에서 추출)
KAIST_TO_SEJONG = {
    # Nouns
    "ncn": "NNG", "ncpa": "NNG", "ncps": "NNG", "ncr": "NNG",
    "nq": "NNP", "nbn": "NNB", "nbu": "NNB", "nnc": "NR", "nno": "NR",
    "npd": "NP", "npp": "NP", "nb": "NNB",
    # Verbs / Adjectives
    "pvg": "VV", "pvd": "VV", "paa": "VA", "pad": "VA", "px": "VX",
    "jp": "VCP",
    # Adnominals / Adverbs
    "mma": "MMD", "mmd": "MMD", "mag": "MAG", "maj": "MAJ", "mad": "MAG",
    # Interjection
    "ii": "IC",
    # Postpositions
    "jcs": "JKS", "jcc": "JKC", "jcm": "JKG", "jco": "JKO",
    "jca": "JKB", "jcv": "JKV", "jcr": "JKQ", "jcj": "JC",
    "jxt": "JX", "jxc": "JX",
    # Endings
    "ep": "EP", "ef": "EF", "ecc": "EC", "ecs": "EC", "ecx": "EC",
    "etn": "ETN", "etm": "ETM",
    # Affixes
    "xp": "XPN", "xsn": "XSN", "xsv": "XSV", "xsa": "XSA",
    # Symbols
    "sf": "SF", "sp": "SP", "ss": "SS", "se": "SE", "sy": "SY",
    "sl": "SL", "sh": "SH", "sn": "SN",
}


def map_pos(pos: str, xpos_scheme: str) -> str:
    """xpos → Sejong POS 매핑."""
    if xpos_scheme == "gsd":
        # GSD는 Sejong 직접
        return pos if pos in SEJONG_TAGS else pos.upper()
    elif xpos_scheme == "kaist":
        return KAIST_TO_SEJONG.get(pos, "NNG")  # 미매핑 시 NNG (보수적)
    else:
        return pos


def parse_conllu(path: Path):
    """CoNLL-U 파서. yields list of (surface_eojeol, lemma_decomp, xpos_decomp) per sentence."""
    sentence = []
    with path.open(encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")
            if not line:
                if sentence:
                    yield sentence
                    sentence = []
                continue
            if line.startswith("#"):
                continue
            parts = line.split("\t")
            if len(parts) < 5:
                continue
            tok_id = parts[0]
            if "-" in tok_id or "." in tok_id:
                continue  # multi-word range or empty node
            surface = parts[1]
            lemma = parts[2]
            xpos = parts[4]
            sentence.append((surface, lemma, xpos))
    if sentence:
        yield sentence


def convert_sentence(sentence, xpos_scheme: str, unknown: Counter) -> list[tuple[str, str]]:
    """Sentence → list of (morpheme_surface, sejong_pos) tuples."""
    morphemes = []
    for surface_eojeol, lemma, xpos in sentence:
        # lemma + xpos를 +로 split
        lemma_parts = lemma.split("+")
        xpos_parts = xpos.split("+")
        # 길이 불일치 시 skip 또는 fallback
        if len(lemma_parts) != len(xpos_parts):
            # surface 전체를 하나의 morpheme으로 (lossy)
            mapped = map_pos(xpos_parts[0] if xpos_parts else "ncn", xpos_scheme)
            if mapped not in SEJONG_TAGS:
                unknown[mapped] += 1
            morphemes.append((surface_eojeol, mapped))
            continue
        for l, p in zip(lemma_parts, xpos_parts):
            if not l or not p:
                continue
            mapped = map_pos(p, xpos_scheme)
            if mapped not in SEJONG_TAGS:
                unknown[mapped] += 1
            morphemes.append((l, mapped))
    return morphemes


def write_tagged(morphemes_per_sent: list[list[tuple[str, str]]], output: Path) -> int:
    """Write .tagged format. Returns sentence count."""
    output.parent.mkdir(parents=True, exist_ok=True)
    count = 0
    with output.open("w", encoding="utf-8") as f:
        for morphs in morphemes_per_sent:
            if not morphs:
                continue
            for surface, pos in morphs:
                # mecab features: POS,*,*,*,*,*,*,*,* (9 fields after POS)
                # We don't know the cost columns since this is training input.
                features = f"{pos},*,*,*,*,*,*,*,*"
                f.write(f"{surface}\t{features}\n")
            f.write("EOS\n")
            count += 1
    return count


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("input", type=Path, help="UD CoNLL-U file")
    parser.add_argument("output", type=Path, help="Output .tagged file")
    parser.add_argument(
        "--xpos-map", choices=["kaist", "gsd"], required=True,
        help="xpos scheme: kaist (lowercase, 매핑 필요) or gsd (Sejong 직접)"
    )
    args = parser.parse_args()

    if not args.input.exists():
        print(f"ERROR: {args.input} not found", file=sys.stderr)
        return 1

    unknown: Counter = Counter()
    sentences = []
    for sent in parse_conllu(args.input):
        morphs = convert_sentence(sent, args.xpos_map, unknown)
        sentences.append(morphs)

    count = write_tagged(sentences, args.output)
    print(f"✓ Converted {count} sentences → {args.output}")

    if unknown:
        print(f"\n⚠ Unknown POS tags (top 10):")
        for tag, n in unknown.most_common(10):
            print(f"  {tag}: {n}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
