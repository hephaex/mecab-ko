#!/usr/bin/env python3
"""Expand proper noun dictionary entries.

This tool generates MeCab dictionary entries for proper nouns including:
- Person names (인명)
- Place names (지명)
- Organization names (기관명)

Data sources:
- Korean Wikipedia articles
- Public data portal
- Custom input files

Usage:
    python expand_proper_nouns.py --source wikipedia --category "대한민국의_배우" -o person.csv
    python expand_proper_nouns.py --source public_data --type organizations -o orgs.csv
    python expand_proper_nouns.py --input names.txt --type person -o output.csv
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Iterator

# Add current directory to path
sys.path.insert(0, str(Path(__file__).parent))

from utils.mecab_format import MecabEntry
from utils.korean_utils import get_jongseong_marker, normalize_hangul
from validators.deduplicator import deduplicate_entries
from validators.pos_inference import POSInferencer
from validators.quality_checker import QualityChecker
from data_sources.wikipedia_fetcher import WikipediaFetcher
from data_sources.public_data_fetcher import PublicDataFetcher


class ProperNounExpander:
    """Expands proper noun dictionary entries from various sources."""

    def __init__(
        self,
        semantic_type: str = "고유명",
        validate: bool = True,
        deduplicate: bool = True,
    ):
        """Initialize expander.

        Args:
            semantic_type: Semantic category (인명, 지명, 기관, 고유명).
            validate: Enable validation.
            deduplicate: Enable deduplication.
        """
        self.semantic_type = semantic_type
        self.validate = validate
        self.deduplicate = deduplicate
        self.inferencer = POSInferencer()
        self.checker = QualityChecker() if validate else None

    def expand_from_list(
        self,
        names: list[str],
    ) -> list[MecabEntry]:
        """Expand proper nouns from name list.

        Args:
            names: List of proper noun names.

        Returns:
            List of MeCab entries.
        """
        entries = []

        for name in names:
            # Normalize
            normalized = normalize_hangul(name)
            if not normalized:
                continue

            # Create entry
            entry = self._create_entry(normalized)
            if entry:
                entries.append(entry)

        # Post-process
        entries = self._post_process(entries)
        return entries

    def expand_from_wikipedia(
        self,
        category: str | None = None,
        limit: int | None = None,
    ) -> list[MecabEntry]:
        """Expand from Wikipedia titles.

        Args:
            category: Wikipedia category to fetch.
            limit: Maximum entries.

        Returns:
            List of MeCab entries.
        """
        print(f"Fetching from Wikipedia (category: {category})...")
        fetcher = WikipediaFetcher()

        if category:
            titles = list(fetcher.fetch_titles_by_category(category, limit))
        else:
            titles = list(fetcher.fetch_all_titles(limit=limit))

        print(f"Fetched {len(titles)} titles")
        return self.expand_from_list(titles)

    def expand_from_public_data(
        self,
        data_type: str = "organizations",
        category: str | None = None,
        limit: int | None = None,
    ) -> list[MecabEntry]:
        """Expand from public data sources.

        Args:
            data_type: Type of data (organizations, addresses).
            category: Category filter.
            limit: Maximum entries.

        Returns:
            List of MeCab entries.
        """
        print(f"Fetching from public data ({data_type})...")
        from data_sources.public_data_fetcher import fetch_public_data

        records = fetch_public_data(data_type, category, limit)
        names = [r.name for r in records]

        print(f"Fetched {len(names)} records")
        return self.expand_from_list(names)

    def expand_from_file(
        self,
        file_path: Path,
    ) -> list[MecabEntry]:
        """Expand from text file (one name per line).

        Args:
            file_path: Path to input file.

        Returns:
            List of MeCab entries.
        """
        print(f"Reading from {file_path}...")
        names = []

        with open(file_path, encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#"):
                    names.append(line)

        print(f"Read {len(names)} names")
        return self.expand_from_list(names)

    def _create_entry(self, name: str) -> MecabEntry | None:
        """Create MeCab entry for proper noun.

        Args:
            name: Proper noun name.

        Returns:
            MecabEntry or None if invalid.
        """
        if not name:
            return None

        # Infer semantic category if not specified
        semantic = self.semantic_type
        if semantic == "고유명":
            semantic = self.inferencer.infer_semantic(name, "NNP")
            if semantic == "*":
                semantic = "고유명"

        # Create entry
        entry = MecabEntry(
            surface=name,
            pos="NNP",
            semantic=semantic,
            has_jongseong=get_jongseong_marker(name),
            reading=name,
        )

        # Validate
        if self.validate and self.checker:
            result = self.checker.validate_entry(entry)
            if not result.is_valid:
                print(f"Warning: Invalid entry: {name}")
                for issue in result.issues:
                    print(f"  {issue}")
                return None

        return entry

    def _post_process(self, entries: list[MecabEntry]) -> list[MecabEntry]:
        """Post-process entries (deduplicate, validate).

        Args:
            entries: Entries to process.

        Returns:
            Processed entries.
        """
        if not entries:
            return entries

        # Deduplicate
        if self.deduplicate:
            entries, stats = deduplicate_entries(entries)
            print(f"Deduplication: {stats}")

        return entries


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Expand proper noun dictionary entries",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )

    # Input source
    source_group = parser.add_mutually_exclusive_group(required=True)
    source_group.add_argument(
        "--source",
        choices=["wikipedia", "public_data"],
        help="Data source type",
    )
    source_group.add_argument(
        "--input",
        type=Path,
        help="Input file (one name per line)",
    )

    # Source options
    parser.add_argument(
        "--category",
        help="Category filter (for Wikipedia)",
    )
    parser.add_argument(
        "--type",
        dest="data_type",
        choices=["person", "place", "organization", "proper"],
        default="proper",
        help="Proper noun type (default: proper)",
    )
    parser.add_argument(
        "--limit",
        type=int,
        help="Maximum entries to generate",
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
    parser.add_argument(
        "--no-deduplicate",
        action="store_true",
        help="Disable deduplication",
    )

    args = parser.parse_args()

    # Map type to semantic category
    semantic_map = {
        "person": "인명",
        "place": "지명",
        "organization": "기관",
        "proper": "고유명",
    }
    semantic = semantic_map[args.data_type]

    # Create expander
    expander = ProperNounExpander(
        semantic_type=semantic,
        validate=not args.no_validate,
        deduplicate=not args.no_deduplicate,
    )

    # Expand based on source
    if args.input:
        entries = expander.expand_from_file(args.input)
    elif args.source == "wikipedia":
        entries = expander.expand_from_wikipedia(args.category, args.limit)
    elif args.source == "public_data":
        data_type_map = {
            "person": "persons",
            "place": "addresses",
            "organization": "organizations",
            "proper": "organizations",
        }
        entries = expander.expand_from_public_data(
            data_type_map[args.data_type],
            args.category,
            args.limit,
        )
    else:
        parser.error("Invalid source")

    # Write output
    print(f"\nWriting {len(entries)} entries to {args.output}...")
    args.output.parent.mkdir(parents=True, exist_ok=True)

    with open(args.output, "w", encoding="utf-8") as f:
        for entry in entries:
            f.write(entry.to_csv_line() + "\n")

    print(f"Done! Generated {len(entries)} entries")
    return 0


if __name__ == "__main__":
    sys.exit(main())
