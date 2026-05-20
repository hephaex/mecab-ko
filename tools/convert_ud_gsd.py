#!/usr/bin/env python3
"""UD Korean-GSD (CoNLL-U) → mecab-ko TSV 변환.

Sprint 143 C: UD GSD는 KAIST와 달리 XPOS column이 이미 **Sejong 태그를 직접 사용**
(NNG, NNP, JKO, ETM, EP, EF, VV, VA, XSV...). 따라서 KAIST → Sejong 매핑이
불필요하며 identity 매핑 + unknown tag(`NA`) skip만 수행.

Output format (tab-separated, KLUE DP / UD Kaist 동일):
    text<TAB>surface1/POS1 surface2/POS2 ...<TAB>eojeol_counts(comma-separated)

Usage:
    python3 tools/convert_ud_gsd.py \\
        data/raw/ud_gsd/ko_gsd-ud-test.conllu \\
        data/eval/ud_gsd_test.tsv

License: UD_Korean-GSD is CC BY-SA 4.0. Output inherits CC BY-SA 4.0.
"""
from __future__ import annotations
import sys
from pathlib import Path

# GSD XPOS는 이미 Sejong 호환. 알려지지 않은 태그(`NA` 등)만 skip.
# 모든 GSD 태그 목록(빈도순):
# NNG, EC, VV, JKB, ETM, JX, NNB, NNP, SF, EF, XSV, JKO, MAG, JKS, EP,
# SN, SS, VA, VX, JKG, VCP, XSN, SP, XSA, MM, XR, SL, JC, NP, ETN, NR,
# XPN, MAJ, SW, JKQ, SH, JKC, VCN, SE, SO, IC, NA, JKV
KNOWN_SEJONG = {
    "NNG", "NNP", "NNB", "NNBC", "NP", "NR",
    "VV", "VA", "VX", "VCP", "VCN",
    "MM", "MMA", "MMD", "MMN",
    "MAG", "MAJ",
    "IC",
    "JKS", "JKC", "JKG", "JKO", "JKB", "JKV", "JKQ", "JX", "JC",
    "EP", "EF", "EC", "ETN", "ETM",
    "XPN", "XSN", "XSV", "XSA", "XR",
    "SF", "SP", "SS", "SE", "SO", "SW", "SL", "SH", "SN", "SY", "SC", "SSO", "SSC",
}


def map_xpos(xpos: str) -> str | None:
    """GSD identity mapping. Unknown tags (e.g., `NA`) → None to skip."""
    xpos = xpos.strip()
    return xpos if xpos in KNOWN_SEJONG else None


def convert_sentence(metadata: dict, tokens: list[tuple[str, str, str]]) -> str | None:
    """Convert one CoNLL-U sentence to TSV line.

    tokens: list of (form, lemma, xpos)
    Returns TSV line: text<TAB>morphemes<TAB>eojeol_counts, or None to skip.

    Note: text is reconstructed from token forms (each token = one eojeol)
    to ensure morpheme/eojeol alignment in evaluation.
    """
    morphemes = []
    eojeol_counts = []
    forms = []
    for (form, lemma, xpos) in tokens:
        lemma_parts = lemma.split("+")
        xpos_parts = xpos.split("+")
        if len(lemma_parts) != len(xpos_parts):
            return None
        eojeol_morph = []
        for surface, tag in zip(lemma_parts, xpos_parts):
            sejong = map_xpos(tag)
            if sejong is None:
                return None
            if not surface:
                return None
            eojeol_morph.append(f"{surface}/{sejong}")
        morphemes.extend(eojeol_morph)
        eojeol_counts.append(len(eojeol_morph))
        forms.append(form)

    if not forms:
        return None

    text = " ".join(forms)
    morph_str = " ".join(morphemes)
    counts_str = ",".join(str(c) for c in eojeol_counts)
    return f"{text}\t{morph_str}\t{counts_str}"


def parse_conllu(path: Path):
    metadata = {}
    tokens = []
    with path.open(encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")
            if not line:
                if tokens:
                    yield metadata, tokens
                metadata = {}
                tokens = []
                continue
            if line.startswith("#"):
                if " = " in line:
                    key, val = line[1:].strip().split(" = ", 1)
                    metadata[key.strip()] = val
                continue
            parts = line.split("\t")
            if len(parts) < 10:
                continue
            tid, form, lemma, upos, xpos = parts[:5]
            if "-" in tid or "." in tid:
                continue
            tokens.append((form, lemma, xpos))
    if tokens:
        yield metadata, tokens


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <input.conllu> <output.tsv>", file=sys.stderr)
        sys.exit(1)
    inp = Path(sys.argv[1])
    out = Path(sys.argv[2])

    converted = 0
    skipped = 0
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8") as fout:
        fout.write("# UD Korean-GSD Evaluation Dataset (converted, silver)\n")
        fout.write("#\n")
        fout.write("# Source: https://github.com/UniversalDependencies/UD_Korean-GSD\n")
        fout.write("# License: CC BY-SA 4.0 (derivative — same license)\n")
        fout.write("# Conversion: tools/convert_ud_gsd.py (Sprint 143 C)\n")
        fout.write("#\n")
        fout.write("# GSD XPOS already uses Sejong tags directly (identity mapping).\n")
        fout.write("# Sentences with unknown tags (e.g., `NA`), empty morphemes, or\n")
        fout.write("# lemma/xpos count mismatch are skipped.\n")
        fout.write("#\n")
        for metadata, tokens in parse_conllu(inp):
            line = convert_sentence(metadata, tokens)
            if line is None:
                skipped += 1
                continue
            fout.write(line + "\n")
            converted += 1

    print(f"Converted: {converted} sentences")
    print(f"Skipped:   {skipped} sentences")
    print(f"Output:    {out}")


if __name__ == "__main__":
    main()
