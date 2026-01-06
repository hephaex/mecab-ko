#!/usr/bin/env python3
"""Expand abbreviation and acronym dictionary entries.

This tool generates MeCab dictionary entries for:
- Abbreviations (줄임말)
- Acronyms (두문자어)
- Initialisms (e.g., KBS, MBC)
- Short forms (e.g., 컴공 <- 컴퓨터공학)

Processing:
- Extract initials from compound nouns
- Common abbreviation patterns
- English acronyms in Korean text

Usage:
    python expand_abbreviations.py --dict /path/to/dict --extract-initials -o abbrevs.csv
    python expand_abbreviations.py --input compounds.csv --patterns korean -o output.csv
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Iterator

sys.path.insert(0, str(Path(__file__).parent))

from utils.mecab_format import MecabEntry, parse_mecab_line
from utils.korean_utils import get_jongseong_marker, decompose_hangul
from validators.deduplicator import deduplicate_entries
from validators.quality_checker import QualityChecker


class AbbreviationGenerator:
    """Generates abbreviation and acronym entries."""

    # Common abbreviation patterns
    ABBREVIATION_PATTERNS = {
        # Korean patterns
        'korean': [
            # Take first syllable of each word
            lambda words: "".join(w[0] for w in words if w),
            # Take first 2 syllables of each word
            lambda words: "".join(w[:2] for w in words if w)[:4],
        ],
        # English initials
        'english': [
            # Just first letters (e.g., KBS)
            lambda words: "".join(w[0].upper() for w in words if w and w[0].isalpha()),
        ],
    }

    # Common Korean abbreviation suffixes
    ABBREV_SUFFIXES = {
        '대', '고', '중', '초',  # Schools
        '청', '부', '원', '국',  # Government
        '시', '구', '동',         # Places
    }

    def __init__(
        self,
        pattern_type: str = "korean",
        min_length: int = 2,
        max_length: int = 6,
        validate: bool = True,
    ):
        """Initialize generator.

        Args:
            pattern_type: Abbreviation pattern type.
            min_length: Minimum abbreviation length.
            max_length: Maximum abbreviation length.
            validate: Enable validation.
        """
        self.pattern_type = pattern_type
        self.min_length = min_length
        self.max_length = max_length
        self.validate = validate
        self.checker = QualityChecker() if validate else None

    def generate_from_compounds(
        self,
        compound_entries: list[MecabEntry],
    ) -> list[MecabEntry]:
        """Generate abbreviations from compound nouns.

        Args:
            compound_entries: Compound noun entries.

        Returns:
            Abbreviation entries.
        """
        abbreviations = []

        for compound in compound_entries:
            # Skip non-compounds
            if compound.entry_type != "Compound":
                continue

            # Extract morphemes
            morphemes = self._extract_morphemes(compound.expression)
            if len(morphemes) < 2:
                continue

            # Generate abbreviations
            abbrevs = self._generate_abbreviations(
                morphemes,
                compound.surface,
                compound.pos,
            )
            abbreviations.extend(abbrevs)

        return self._post_process(abbreviations)

    def generate_from_patterns(
        self,
        text_entries: list[str],
    ) -> list[MecabEntry]:
        """Generate abbreviations using pattern matching.

        Args:
            text_entries: List of full-form texts.

        Returns:
            Abbreviation entries.
        """
        abbreviations = []

        for text in text_entries:
            # Split into words
            words = self._split_words(text)
            if len(words) < 2:
                continue

            # Apply patterns
            for pattern_fn in self.ABBREVIATION_PATTERNS.get(self.pattern_type, []):
                abbrev_surface = pattern_fn(words)

                if self.min_length <= len(abbrev_surface) <= self.max_length:
                    entry = self._create_abbreviation(
                        abbrev_surface,
                        text,
                    )
                    if entry:
                        abbreviations.append(entry)

        return self._post_process(abbreviations)

    def generate_initialisms(
        self,
        full_forms: dict[str, str],
    ) -> list[MecabEntry]:
        """Generate initialisms from full-form dictionary.

        Args:
            full_forms: Dictionary mapping abbreviation -> full form.

        Returns:
            Initialism entries.

        Examples:
            >>> gen = AbbreviationGenerator()
            >>> entries = gen.generate_initialisms({"KBS": "한국방송공사"})
        """
        abbreviations = []

        for abbrev, full_form in full_forms.items():
            entry = self._create_abbreviation(abbrev, full_form)
            if entry:
                abbreviations.append(entry)

        return self._post_process(abbreviations)

    def _extract_morphemes(self, expression: str) -> list[str]:
        """Extract morpheme surfaces from expression.

        Args:
            expression: MeCab expression (e.g., "컴퓨터/NNG/*+공학/NNG/*").

        Returns:
            List of morpheme surfaces.
        """
        if expression == "*":
            return []

        morphemes = []
        for part in expression.split("+"):
            # Extract surface (before /)
            if "/" in part:
                surface = part.split("/")[0]
                morphemes.append(surface)

        return morphemes

    def _split_words(self, text: str) -> list[str]:
        """Split text into words.

        Args:
            text: Text to split.

        Returns:
            List of words.
        """
        # Simple splitting by spaces and common separators
        words = re.split(r'[\s\-_]+', text)
        return [w.strip() for w in words if w.strip()]

    def _generate_abbreviations(
        self,
        morphemes: list[str],
        full_surface: str,
        pos: str,
    ) -> list[MecabEntry]:
        """Generate abbreviations from morphemes.

        Args:
            morphemes: List of morphemes.
            full_surface: Full compound surface.
            pos: POS tag.

        Returns:
            List of abbreviation entries.
        """
        abbreviations = []

        # Pattern 1: First syllable of each morpheme
        abbrev1 = "".join(m[0] for m in morphemes if m)
        if self.min_length <= len(abbrev1) <= self.max_length:
            entry = self._create_abbreviation(abbrev1, full_surface, pos)
            if entry:
                abbreviations.append(entry)

        # Pattern 2: First 2 syllables of each morpheme
        if len(morphemes) >= 2:
            abbrev2 = "".join(m[:2] for m in morphemes if m)[:self.max_length]
            if self.min_length <= len(abbrev2) <= self.max_length and abbrev2 != abbrev1:
                entry = self._create_abbreviation(abbrev2, full_surface, pos)
                if entry:
                    abbreviations.append(entry)

        return abbreviations

    def _create_abbreviation(
        self,
        abbrev_surface: str,
        full_form: str,
        pos: str = "NNG",
    ) -> MecabEntry | None:
        """Create abbreviation entry.

        Args:
            abbrev_surface: Abbreviation surface form.
            full_form: Full form.
            pos: POS tag.

        Returns:
            Abbreviation entry or None.
        """
        if not abbrev_surface:
            return None

        # Create entry with note about full form in expression
        entry = MecabEntry(
            surface=abbrev_surface,
            pos=pos,
            has_jongseong=get_jongseong_marker(abbrev_surface),
            reading=abbrev_surface,
            entry_type="*",
            semantic="약어",  # Abbreviation marker
            expression=f"*+*+*+{full_form}",  # Store full form
        )

        # Validate
        if self.validate and self.checker:
            result = self.checker.validate_entry(entry)
            if not result.is_valid:
                return None

        return entry

    def _post_process(self, abbreviations: list[MecabEntry]) -> list[MecabEntry]:
        """Post-process abbreviations."""
        if not abbreviations:
            return abbreviations

        # Deduplicate
        abbreviations, stats = deduplicate_entries(abbreviations)
        print(f"Generated {len(abbreviations)} unique abbreviations")

        return abbreviations


def load_compounds(input_path: Path) -> list[MecabEntry]:
    """Load compound entries from CSV.

    Args:
        input_path: Path to CSV file.

    Returns:
        List of compound entries.
    """
    compounds = []

    with open(input_path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue

            try:
                entry = parse_mecab_line(line)
                if entry.entry_type == "Compound":
                    compounds.append(entry)
            except Exception:
                continue

    return compounds


def load_abbreviation_map(map_file: Path) -> dict[str, str]:
    """Load abbreviation mapping from file.

    Format:
        abbreviation=full form
        KBS=한국방송공사
        MBC=문화방송

    Args:
        map_file: Path to mapping file.

    Returns:
        Dictionary mapping abbreviation -> full form.
    """
    abbrev_map = {}

    with open(map_file, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue

            if "=" in line:
                abbrev, full = line.split("=", 1)
                abbrev_map[abbrev.strip()] = full.strip()

    return abbrev_map


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Generate abbreviation and acronym entries",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )

    # Input options
    parser.add_argument(
        "--dict",
        type=Path,
        help="Dictionary CSV with compound nouns",
    )
    parser.add_argument(
        "--input",
        type=Path,
        help="Input text file or CSV",
    )
    parser.add_argument(
        "--map",
        type=Path,
        help="Abbreviation mapping file (abbrev=full)",
    )

    # Generation options
    parser.add_argument(
        "--extract-initials",
        action="store_true",
        help="Extract initials from compounds",
    )
    parser.add_argument(
        "--patterns",
        choices=["korean", "english", "both"],
        default="korean",
        help="Abbreviation patterns to use",
    )
    parser.add_argument(
        "--min-length",
        type=int,
        default=2,
        help="Minimum abbreviation length (default: 2)",
    )
    parser.add_argument(
        "--max-length",
        type=int,
        default=6,
        help="Maximum abbreviation length (default: 6)",
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
    generator = AbbreviationGenerator(
        pattern_type=args.patterns,
        min_length=args.min_length,
        max_length=args.max_length,
        validate=not args.no_validate,
    )

    # Generate abbreviations
    abbreviations = []

    if args.map:
        print(f"Loading abbreviation mappings from {args.map}...")
        abbrev_map = load_abbreviation_map(args.map)
        print(f"Loaded {len(abbrev_map)} mappings")
        abbreviations.extend(generator.generate_initialisms(abbrev_map))

    if args.dict or args.input:
        input_file = args.dict or args.input

        if args.extract_initials:
            print(f"Loading compounds from {input_file}...")
            compounds = load_compounds(input_file)
            print(f"Loaded {len(compounds)} compounds")
            abbreviations.extend(generator.generate_from_compounds(compounds))
        else:
            print(f"Loading text from {input_file}...")
            with open(input_file, encoding="utf-8") as f:
                texts = [line.strip() for line in f if line.strip()]
            print(f"Loaded {len(texts)} entries")
            abbreviations.extend(generator.generate_from_patterns(texts))

    # Deduplicate all
    if abbreviations:
        abbreviations, _ = deduplicate_entries(abbreviations)

    # Write output
    print(f"\nWriting {len(abbreviations)} entries to {args.output}...")
    args.output.parent.mkdir(parents=True, exist_ok=True)

    with open(args.output, "w", encoding="utf-8") as f:
        for entry in abbreviations:
            f.write(entry.to_csv_line() + "\n")

    print(f"Done! Generated {len(abbreviations)} abbreviation entries")
    return 0


if __name__ == "__main__":
    sys.exit(main())
