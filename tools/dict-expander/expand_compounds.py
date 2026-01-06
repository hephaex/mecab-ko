#!/usr/bin/env python3
"""Expand compound noun dictionary entries.

This tool generates MeCab dictionary entries for compound nouns by:
- Combining existing morphemes
- Pattern-based generation
- Frequency filtering

Compound types:
- N + N: 명사 + 명사 (e.g., 컴퓨터공학)
- N + Suffix: 명사 + 접미사 (e.g., 한국어)
- Multi-morpheme: 3+ morphemes (e.g., 인공지능기술)

Usage:
    python expand_compounds.py --dict /path/to/seed --patterns patterns.txt -o compounds.csv
    python expand_compounds.py --input base_nouns.txt --combine -o output.csv
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Iterator
from itertools import product

sys.path.insert(0, str(Path(__file__).parent))

from utils.mecab_format import MecabEntry, parse_mecab_line
from utils.korean_utils import get_jongseong_marker
from validators.deduplicator import deduplicate_entries
from validators.quality_checker import QualityChecker


class CompoundGenerator:
    """Generates compound noun entries."""

    # Common noun suffixes for compounds
    COMMON_SUFFIXES = {
        '어': 'NNG',      # ~어 (언어)
        '학': 'NNG',      # ~학 (과학)
        '법': 'NNG',      # ~법 (방법)
        '론': 'NNG',      # ~론 (이론)
        '력': 'NNG',      # ~력 (능력)
        '성': 'NNG',      # ~성 (특성)
        '적': 'NNG',      # ~적 (역사적)
        '화': 'NNG',      # ~화 (정보화)
        '권': 'NNG',      # ~권 (주권)
        '주의': 'NNG',    # ~주의 (자본주의)
        '관': 'NNG',      # ~관 (세계관)
    }

    def __init__(
        self,
        min_frequency: int = 2,
        max_length: int = 10,
        validate: bool = True,
    ):
        """Initialize generator.

        Args:
            min_frequency: Minimum frequency threshold.
            max_length: Maximum syllable length.
            validate: Enable validation.
        """
        self.min_frequency = min_frequency
        self.max_length = max_length
        self.validate = validate
        self.checker = QualityChecker() if validate else None

    def generate_from_components(
        self,
        components: list[MecabEntry],
        max_components: int = 3,
    ) -> list[MecabEntry]:
        """Generate compounds from base components.

        Args:
            components: Base morpheme entries.
            max_components: Maximum morphemes per compound.

        Returns:
            Generated compound entries.
        """
        compounds = []

        # Two-morpheme compounds
        if max_components >= 2:
            compounds.extend(self._generate_two_morpheme(components))

        # Three-morpheme compounds
        if max_components >= 3:
            compounds.extend(self._generate_three_morpheme(components))

        return self._post_process(compounds)

    def generate_with_suffixes(
        self,
        base_nouns: list[MecabEntry],
    ) -> list[MecabEntry]:
        """Generate compounds with common suffixes.

        Args:
            base_nouns: Base noun entries.

        Returns:
            Compound entries with suffixes.
        """
        compounds = []

        for base in base_nouns:
            if base.pos not in ('NNG', 'NNP'):
                continue

            for suffix, suffix_pos in self.COMMON_SUFFIXES.items():
                compound = self._create_compound(
                    [base.surface, suffix],
                    [base.pos, suffix_pos],
                )
                if compound:
                    compounds.append(compound)

        return self._post_process(compounds)

    def generate_from_patterns(
        self,
        pattern_file: Path,
        morpheme_dict: dict[str, MecabEntry],
    ) -> list[MecabEntry]:
        """Generate compounds from pattern file.

        Pattern file format:
            # Pattern: morpheme1 + morpheme2
            컴퓨터 공학
            인공 지능
            자연 언어 처리

        Args:
            pattern_file: Path to pattern file.
            morpheme_dict: Dictionary of morphemes.

        Returns:
            Generated compounds.
        """
        compounds = []

        with open(pattern_file, encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue

                # Parse pattern
                parts = line.split()
                if len(parts) < 2:
                    continue

                # Look up morphemes
                morphemes = []
                pos_tags = []
                for part in parts:
                    if part in morpheme_dict:
                        entry = morpheme_dict[part]
                        morphemes.append(entry.surface)
                        pos_tags.append(entry.pos)
                    else:
                        # Use as-is
                        morphemes.append(part)
                        pos_tags.append('NNG')

                # Create compound
                compound = self._create_compound(morphemes, pos_tags)
                if compound:
                    compounds.append(compound)

        return self._post_process(compounds)

    def _generate_two_morpheme(
        self,
        components: list[MecabEntry],
    ) -> list[MecabEntry]:
        """Generate two-morpheme compounds."""
        compounds = []
        nouns = [c for c in components if c.pos in ('NNG', 'NNP')]

        for c1, c2 in product(nouns, repeat=2):
            # Skip same morpheme
            if c1.surface == c2.surface:
                continue

            compound = self._create_compound(
                [c1.surface, c2.surface],
                [c1.pos, c2.pos],
            )
            if compound:
                compounds.append(compound)

        return compounds

    def _generate_three_morpheme(
        self,
        components: list[MecabEntry],
    ) -> list[MecabEntry]:
        """Generate three-morpheme compounds."""
        compounds = []
        nouns = [c for c in components if c.pos in ('NNG', 'NNP')]

        # Limit combinations to avoid explosion
        if len(nouns) > 100:
            nouns = nouns[:100]

        for c1, c2, c3 in product(nouns, repeat=3):
            # Skip if all same
            if c1.surface == c2.surface == c3.surface:
                continue

            compound = self._create_compound(
                [c1.surface, c2.surface, c3.surface],
                [c1.pos, c2.pos, c3.pos],
            )
            if compound:
                compounds.append(compound)

        return compounds

    def _create_compound(
        self,
        morphemes: list[str],
        pos_tags: list[str],
    ) -> MecabEntry | None:
        """Create compound entry from morphemes.

        Args:
            morphemes: List of morpheme surfaces.
            pos_tags: List of POS tags.

        Returns:
            Compound entry or None if invalid.
        """
        if not morphemes or len(morphemes) != len(pos_tags):
            return None

        # Combine morphemes
        surface = "".join(morphemes)

        # Check length
        from utils.korean_utils import syllable_count
        if syllable_count(surface) > self.max_length:
            return None

        # Build expression
        expression_parts = []
        for morph, pos in zip(morphemes, pos_tags):
            expression_parts.append(f"{morph}/{pos}/*")
        expression = "+".join(expression_parts)

        # Create entry
        entry = MecabEntry(
            surface=surface,
            pos="NNP" if any(p == "NNP" for p in pos_tags) else "NNG",
            has_jongseong=get_jongseong_marker(surface),
            reading=surface,
            entry_type="Compound",
            first_pos=pos_tags[0],
            last_pos=pos_tags[-1],
            expression=expression,
        )

        # Validate
        if self.validate and self.checker:
            result = self.checker.validate_entry(entry)
            if not result.is_valid:
                return None

        return entry

    def _post_process(self, compounds: list[MecabEntry]) -> list[MecabEntry]:
        """Post-process compounds (deduplicate, filter)."""
        if not compounds:
            return compounds

        # Deduplicate
        compounds, stats = deduplicate_entries(compounds)
        print(f"Generated {len(compounds)} unique compounds")

        return compounds


def load_morpheme_dict(dict_path: Path) -> dict[str, MecabEntry]:
    """Load morpheme dictionary from CSV files.

    Args:
        dict_path: Path to dictionary directory.

    Returns:
        Dictionary mapping surface -> entry.
    """
    morphemes = {}

    for csv_file in dict_path.glob("*.csv"):
        with open(csv_file, encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue

                try:
                    entry = parse_mecab_line(line)
                    morphemes[entry.surface] = entry
                except Exception:
                    continue

    return morphemes


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Generate compound noun dictionary entries",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )

    # Input options
    input_group = parser.add_mutually_exclusive_group(required=True)
    input_group.add_argument(
        "--dict",
        type=Path,
        help="Dictionary directory with base morphemes",
    )
    input_group.add_argument(
        "--input",
        type=Path,
        help="Input file with base nouns",
    )

    parser.add_argument(
        "--patterns",
        type=Path,
        help="Pattern file for guided generation",
    )
    parser.add_argument(
        "--combine",
        action="store_true",
        help="Generate all combinations (use with caution)",
    )
    parser.add_argument(
        "--suffixes",
        action="store_true",
        help="Generate compounds with common suffixes",
    )

    # Generation options
    parser.add_argument(
        "--max-components",
        type=int,
        default=2,
        help="Maximum morphemes per compound (default: 2)",
    )
    parser.add_argument(
        "--max-length",
        type=int,
        default=10,
        help="Maximum syllable length (default: 10)",
    )

    # Output options
    parser.add_argument(
        "-o", "--output",
        type=Path,
        required=True,
        help="Output CSV file",
    )
    parser.add_argument(
        "--no-validate",
        action="store_true",
        help="Disable validation",
    )

    args = parser.parse_args()

    # Create generator
    generator = CompoundGenerator(
        max_length=args.max_length,
        validate=not args.no_validate,
    )

    # Load morphemes
    if args.dict:
        print(f"Loading morphemes from {args.dict}...")
        morpheme_dict = load_morpheme_dict(args.dict)
        print(f"Loaded {len(morpheme_dict)} morphemes")
        components = list(morpheme_dict.values())
    else:
        # Load from simple text file
        print(f"Loading nouns from {args.input}...")
        components = []
        with open(args.input, encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#"):
                    entry = MecabEntry(
                        surface=line,
                        pos="NNG",
                        has_jongseong=get_jongseong_marker(line),
                        reading=line,
                    )
                    components.append(entry)
        print(f"Loaded {len(components)} nouns")
        morpheme_dict = {e.surface: e for e in components}

    # Generate compounds
    compounds = []

    if args.patterns:
        print(f"Generating from patterns...")
        compounds.extend(generator.generate_from_patterns(args.patterns, morpheme_dict))

    if args.suffixes:
        print("Generating with suffixes...")
        compounds.extend(generator.generate_with_suffixes(components))

    if args.combine:
        print("Generating combinations...")
        compounds.extend(generator.generate_from_components(
            components,
            max_components=args.max_components,
        ))

    # Deduplicate all
    if compounds:
        compounds, _ = deduplicate_entries(compounds)

    # Write output
    print(f"\nWriting {len(compounds)} entries to {args.output}...")
    args.output.parent.mkdir(parents=True, exist_ok=True)

    with open(args.output, "w", encoding="utf-8") as f:
        for entry in compounds:
            f.write(entry.to_csv_line() + "\n")

    print(f"Done! Generated {len(compounds)} compound entries")
    return 0


if __name__ == "__main__":
    sys.exit(main())
