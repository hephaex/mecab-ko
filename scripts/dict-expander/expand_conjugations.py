#!/usr/bin/env python3
"""Expand verb/adjective conjugation dictionary entries.

This tool generates MeCab dictionary entries for Korean verb and adjective
conjugations including:
- Irregular verb forms
- Common conjugation patterns
- Tense/aspect variations
- Honorific forms

Conjugation types:
- VV (동사): 하다, 가다, 먹다, etc.
- VA (형용사): 크다, 작다, 좋다, etc.

Usage:
    python expand_conjugations.py --dict /path/to/verbs.csv --conjugate -o conjugations.csv
    python expand_conjugations.py --input verbs.txt --patterns common -o output.csv
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Iterator

sys.path.insert(0, str(Path(__file__).parent))

from utils.mecab_format import MecabEntry, parse_mecab_line
from utils.korean_utils import (
    get_jongseong_marker,
    decompose_hangul,
    compose_hangul,
    is_hangul,
)
from validators.deduplicator import deduplicate_entries
from validators.quality_checker import QualityChecker


class ConjugationGenerator:
    """Generates verb/adjective conjugation entries."""

    # Common verb endings to conjugate
    VERB_ENDINGS = {
        '다': True,  # Dictionary form
    }

    # Conjugation patterns (simplified)
    CONJUGATION_PATTERNS = {
        # Present tense
        'present_informal': [
            ('아', 'EC'),  # -아
            ('어', 'EC'),  # -어
            ('여', 'EC'),  # -여
        ],
        # Past tense
        'past': [
            ('았', 'EP'),  # -았
            ('었', 'EP'),  # -었
            ('였', 'EP'),  # -였
        ],
        # Future/guess
        'future': [
            ('겠', 'EP'),  # -겠
        ],
        # Honorific
        'honorific': [
            ('시', 'EP'),  # -시
        ],
        # Connecting
        'connecting': [
            ('고', 'EC'),  # -고
            ('며', 'EC'),  # -며
            ('면', 'EC'),  # -면
            ('지만', 'EC'),  # -지만
        ],
    }

    # Common irregular verbs (simplified)
    IRREGULAR_VERBS = {
        '하다': {
            '하': ('VV', '하다'),
            '해': ('VV', '하다'),
        },
        '되다': {
            '되': ('VV', '되다'),
            '돼': ('VV', '되다'),
        },
    }

    def __init__(
        self,
        patterns: list[str] | None = None,
        include_irregular: bool = True,
        validate: bool = True,
    ):
        """Initialize generator.

        Args:
            patterns: Conjugation patterns to generate (None = all).
            include_irregular: Include irregular forms.
            validate: Enable validation.
        """
        self.patterns = patterns or list(self.CONJUGATION_PATTERNS.keys())
        self.include_irregular = include_irregular
        self.validate = validate
        self.checker = QualityChecker() if validate else None

    def generate_conjugations(
        self,
        verb_entries: list[MecabEntry],
    ) -> list[MecabEntry]:
        """Generate conjugations from verb entries.

        Args:
            verb_entries: Base verb/adjective entries.

        Returns:
            Generated conjugation entries.
        """
        conjugations = []

        for verb in verb_entries:
            if verb.pos not in ('VV', 'VA'):
                continue

            # Extract stem
            stem = self._extract_stem(verb.surface)
            if not stem:
                continue

            # Generate patterns
            for pattern_name in self.patterns:
                if pattern_name not in self.CONJUGATION_PATTERNS:
                    continue

                for suffix, suffix_pos in self.CONJUGATION_PATTERNS[pattern_name]:
                    conjugated = self._create_conjugation(
                        verb,
                        stem,
                        suffix,
                        suffix_pos,
                    )
                    if conjugated:
                        conjugations.append(conjugated)

        # Handle irregulars
        if self.include_irregular:
            conjugations.extend(self._generate_irregular(verb_entries))

        return self._post_process(conjugations)

    def _extract_stem(self, verb: str) -> str | None:
        """Extract verb stem by removing -다.

        Args:
            verb: Verb in dictionary form (e.g., 하다, 먹다).

        Returns:
            Verb stem (e.g., 하, 먹) or None.
        """
        if not verb.endswith('다'):
            return None

        stem = verb[:-1]
        if not stem:
            return None

        return stem

    def _create_conjugation(
        self,
        base_verb: MecabEntry,
        stem: str,
        suffix: str,
        suffix_pos: str,
    ) -> MecabEntry | None:
        """Create conjugated form entry.

        Args:
            base_verb: Base verb entry.
            stem: Verb stem.
            suffix: Conjugation suffix.
            suffix_pos: POS tag for suffix.

        Returns:
            Conjugated entry or None.
        """
        # Combine stem + suffix
        surface = stem + suffix

        # Build expression
        expression = f"{stem}/{base_verb.pos}/*+{suffix}/{suffix_pos}/*"

        # Create entry as inflected form
        entry = MecabEntry(
            surface=surface,
            pos=base_verb.pos,
            has_jongseong=get_jongseong_marker(surface),
            reading=surface,
            entry_type="Inflect",
            first_pos=base_verb.pos,
            last_pos=suffix_pos,
            expression=expression,
        )

        # Validate
        if self.validate and self.checker:
            result = self.checker.validate_entry(entry)
            if not result.is_valid:
                return None

        return entry

    def _generate_irregular(
        self,
        verb_entries: list[MecabEntry],
    ) -> list[MecabEntry]:
        """Generate irregular verb forms.

        Args:
            verb_entries: Base verb entries.

        Returns:
            Irregular conjugation entries.
        """
        irregulars = []

        for verb in verb_entries:
            if verb.surface not in self.IRREGULAR_VERBS:
                continue

            forms = self.IRREGULAR_VERBS[verb.surface]
            for form, (pos, base) in forms.items():
                entry = MecabEntry(
                    surface=form,
                    pos=pos,
                    has_jongseong=get_jongseong_marker(form),
                    reading=form,
                    entry_type="Inflect",
                    expression=f"{form}/{pos}/*",
                )
                irregulars.append(entry)

        return irregulars

    def _post_process(self, conjugations: list[MecabEntry]) -> list[MecabEntry]:
        """Post-process conjugations."""
        if not conjugations:
            return conjugations

        # Deduplicate
        conjugations, stats = deduplicate_entries(conjugations)
        print(f"Generated {len(conjugations)} unique conjugations")

        return conjugations


def load_verbs(input_path: Path) -> list[MecabEntry]:
    """Load verb entries from file.

    Args:
        input_path: Path to input file.

    Returns:
        List of verb entries.
    """
    verbs = []

    if input_path.suffix == '.csv':
        # CSV format
        with open(input_path, encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue

                try:
                    entry = parse_mecab_line(line)
                    if entry.pos in ('VV', 'VA'):
                        verbs.append(entry)
                except Exception:
                    continue
    else:
        # Text format (one verb per line)
        with open(input_path, encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue

                # Assume VV by default
                entry = MecabEntry(
                    surface=line,
                    pos="VV",
                    has_jongseong=get_jongseong_marker(line),
                    reading=line,
                )
                verbs.append(entry)

    return verbs


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Generate verb/adjective conjugation entries",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )

    # Input options
    parser.add_argument(
        "--dict",
        type=Path,
        help="Dictionary CSV file with verbs",
    )
    parser.add_argument(
        "--input",
        type=Path,
        help="Input text file with verbs (one per line)",
    )

    # Generation options
    parser.add_argument(
        "--conjugate",
        action="store_true",
        help="Generate common conjugations",
    )
    parser.add_argument(
        "--patterns",
        nargs="+",
        choices=["present_informal", "past", "future", "honorific", "connecting", "common"],
        help="Conjugation patterns to generate",
    )
    parser.add_argument(
        "--no-irregular",
        action="store_true",
        help="Skip irregular forms",
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

    # Require at least one input
    if not args.dict and not args.input:
        parser.error("Either --dict or --input required")

    # Default patterns
    if args.patterns:
        if "common" in args.patterns:
            patterns = ["present_informal", "past", "connecting"]
        else:
            patterns = args.patterns
    else:
        patterns = None  # All patterns

    # Load verbs
    verbs = []
    if args.dict:
        print(f"Loading verbs from {args.dict}...")
        verbs.extend(load_verbs(args.dict))
    if args.input:
        print(f"Loading verbs from {args.input}...")
        verbs.extend(load_verbs(args.input))

    print(f"Loaded {len(verbs)} verbs")

    # Generate conjugations
    generator = ConjugationGenerator(
        patterns=patterns,
        include_irregular=not args.no_irregular,
        validate=not args.no_validate,
    )

    conjugations = generator.generate_conjugations(verbs)

    # Write output
    print(f"\nWriting {len(conjugations)} entries to {args.output}...")
    args.output.parent.mkdir(parents=True, exist_ok=True)

    with open(args.output, "w", encoding="utf-8") as f:
        for entry in conjugations:
            f.write(entry.to_csv_line() + "\n")

    print(f"Done! Generated {len(conjugations)} conjugation entries")
    return 0


if __name__ == "__main__":
    sys.exit(main())
