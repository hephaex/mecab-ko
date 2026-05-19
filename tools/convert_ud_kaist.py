#!/usr/bin/env python3
"""UD Korean-Kaist (CoNLL-U) → mecab-ko TSV 변환.

Sprint 139 P2: KAIST XPOS tags를 Sejong POS tags로 매핑하고,
lemma column의 형태소 분해를 활용해 mecab-ko의 silver gold 평가셋 생성.

Output format (tab-separated):
    text<TAB>surface1/POS1 surface2/POS2 ...<TAB>eojeol_counts(comma-separated)

Usage:
    python3 tools/convert_ud_kaist.py \\
        data/raw/ud_kaist/ko_kaist-ud-test.conllu \\
        data/eval/ud_kaist_test.tsv

License note: UD_Korean-Kaist is CC BY-SA 4.0.
This converter output is a derivative work and inherits CC BY-SA 4.0.
KAIST → Sejong mapping is lossy — see XPOS_TO_SEJONG dict for choices.
"""
from __future__ import annotations
import sys
from pathlib import Path

# KAIST XPOS → Sejong tag mapping (lossy in some cases — see comments)
# Based on KAIST POS tag specifications + Sejong scheme reference.
XPOS_TO_SEJONG = {
    # Nouns
    "ncn": "NNG",    # concrete common noun
    "ncpa": "NNG",   # action concrete noun (행위성 명사) — kept as NNG (Sejong has no separate)
    "ncps": "NNG",   # state concrete noun
    "nq": "NNP",     # proper noun
    "nbn": "NNB",    # bound noun (의존명사)
    "nbu": "NNBC",   # unit bound noun (단위성 의존명사)
    "nnc": "NNBC",   # counter (수분류사)
    "nno": "NR",     # numeral
    "npp": "NP",     # personal pronoun
    "npd": "NP",     # demonstrative pronoun

    # Predicates
    "pvg": "VV",     # general verb
    "pvd": "VV",     # demonstrative verb (rare)
    "paa": "VA",     # attributive adjective
    "pad": "VA",     # demonstrative adjective
    "px": "VX",      # auxiliary predicate
    "jp": "VCP",     # copula (이다)

    # Particles (조사)
    "jcs": "JKS",    # subjective case
    "jcc": "JKC",    # complement case
    "jco": "JKO",    # objective case
    "jca": "JKB",    # adverbial case
    "jcm": "JKG",    # modifier (genitive) case
    "jcj": "JC",     # conjunctive case
    "jcr": "JKQ",    # quotative case
    "jct": "JKB",    # (rare/uncertain — default JKB)
    "jxt": "JX",     # topical particle (보조사)
    "jxc": "JX",     # auxiliary particle
    "jxf": "JX",     # (rare auxiliary particle)

    # Endings (어미)
    "ecs": "EC",     # subordinate connective ending
    "ecc": "EC",     # coordinate connective ending
    "ecx": "EC",     # auxiliary connective ending
    "etm": "ETM",    # adnominal-formation ending
    "etn": "ETN",    # nominal-formation ending
    "ep": "EP",      # pre-final ending
    "ef": "EF",      # final ending

    # Affixes (접사)
    "xsv": "XSV",    # verbalizing suffix
    "xsa": "XSA",    # adjectivizing suffix
    "xsn": "XSN",    # nominalizing suffix
    "xsm": "XSN",    # modifier suffix → mapped to XSN (closest)
    "xp": "XPN",     # prefix

    # Adverbs
    "mag": "MAG",    # general adverb
    "maj": "MAJ",    # conjunctive adverb
    "mad": "MAG",    # (rare — default MAG)

    # Adnominals (관형사)
    "mmd": "MM",     # demonstrative adnominal
    "mma": "MM",     # (treated as MM)

    # Symbols / Punctuation
    "sf": "SF",      # sentence final (. ! ?)
    "sp": "SP",      # comma-like
    "sc": "SC",      # other punctuation
    "sr": "SC",      # (uncertain — default SC)
    "ss": "SS",      # quotes
    "se": "SE",      # ellipsis
    "so": "SO",      # other special
    "sw": "SW",      # other special
    "su": "SY",      # unit symbol
    "sl": "SL",      # foreign letters
    "sh": "SH",      # Hanja
    "sn": "SN",      # numeric

    # Foreign / interjection / others
    "f": "SL",       # foreign (default SL)
    "ii": "IC",      # interjection
}


def map_xpos(xpos: str) -> str | None:
    """Map KAIST XPOS → Sejong tag. Returns None if unmapped."""
    return XPOS_TO_SEJONG.get(xpos.lower())


def convert_sentence(metadata: dict, tokens: list[tuple[str, str, str]]) -> str | None:
    """Convert one CoNLL-U sentence to TSV line.

    tokens: list of (form, lemma, xpos)
    Returns TSV line: text<TAB>morphemes<TAB>eojeol_counts, or None to skip.
    """
    text = metadata.get("text", "").strip()
    if not text:
        return None

    morphemes = []
    eojeol_counts = []
    for (form, lemma, xpos) in tokens:
        # Split lemma + xpos by "+"
        lemma_parts = lemma.split("+")
        xpos_parts = xpos.split("+")
        if len(lemma_parts) != len(xpos_parts):
            # Misaligned — skip this sentence (rare cases like "않" with OrigLemma)
            return None
        eojeol_morph = []
        for surface, tag in zip(lemma_parts, xpos_parts):
            sejong = map_xpos(tag)
            if sejong is None:
                return None  # Unknown tag — skip
            if not surface:
                return None  # Empty morpheme — skip
            eojeol_morph.append(f"{surface}/{sejong}")
        morphemes.extend(eojeol_morph)
        eojeol_counts.append(len(eojeol_morph))

    morph_str = " ".join(morphemes)
    counts_str = ",".join(str(c) for c in eojeol_counts)
    return f"{text}\t{morph_str}\t{counts_str}"


def parse_conllu(path: Path):
    """Yield (metadata, tokens) per sentence."""
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
            # Skip multi-word tokens (id like "1-2") and empty tokens
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
        # Header comments
        fout.write("# UD Korean-Kaist Evaluation Dataset (converted, silver)\n")
        fout.write("#\n")
        fout.write("# Source: https://github.com/UniversalDependencies/UD_Korean-Kaist\n")
        fout.write("# License: CC BY-SA 4.0 (derivative — same license)\n")
        fout.write("# Conversion: tools/convert_ud_kaist.py (Sprint 139 P2)\n")
        fout.write("#\n")
        fout.write("# KAIST XPOS → Sejong tag mapping is lossy in some cases:\n")
        fout.write("#   - ncpa/ncps → NNG (Sejong has no fine subdivision)\n")
        fout.write("#   - npd → NP (demonstrative pronoun absorbed)\n")
        fout.write("#   - mmd/mma → MM (관형사 subdivision absorbed)\n")
        fout.write("#   - jct → JKB (rare/uncertain)\n")
        fout.write("# Sentences with unknown tags, empty morphemes, or lemma/xpos\n")
        fout.write("# count mismatch are skipped.\n")
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
