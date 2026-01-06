#!/usr/bin/env python3
"""
MeCab-Ko Dictionary Merger

Merges multiple MeCab dictionary CSV files with conflict resolution.
"""

from __future__ import annotations

import argparse
import csv
import logging
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Iterator, TextIO

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


@dataclass
class MecabEntry:
    """MeCab dictionary entry."""

    surface: str
    left_id: int
    right_id: int
    cost: int
    pos: str
    pos_detail1: str = "*"
    pos_detail2: str = "*"
    pos_detail3: str = "*"
    inflection_type: str = "*"
    inflection_form: str = "*"
    base_form: str = "*"
    reading: str = "*"
    pronunciation: str = "*"

    @classmethod
    def from_csv_row(cls, row: list[str]) -> MecabEntry:
        """Parse from CSV row."""
        if len(row) < 5:
            raise ValueError(f"Invalid CSV row: {row}")

        # Ensure we have all 13 fields
        while len(row) < 13:
            row.append("*")

        return cls(
            surface=row[0],
            left_id=int(row[1]) if row[1] != "*" else 0,
            right_id=int(row[2]) if row[2] != "*" else 0,
            cost=int(row[3]) if row[3] != "*" else 0,
            pos=row[4],
            pos_detail1=row[5],
            pos_detail2=row[6],
            pos_detail3=row[7],
            inflection_type=row[8],
            inflection_form=row[9],
            base_form=row[10],
            reading=row[11],
            pronunciation=row[12],
        )

    def to_csv_row(self) -> list[str]:
        """Convert to CSV row."""
        return [
            self.surface,
            str(self.left_id),
            str(self.right_id),
            str(self.cost),
            self.pos,
            self.pos_detail1,
            self.pos_detail2,
            self.pos_detail3,
            self.inflection_type,
            self.inflection_form,
            self.base_form,
            self.reading,
            self.pronunciation,
        ]

    def key(self) -> tuple[str, str]:
        """Return unique key for deduplication."""
        return (self.surface, self.pos)

    def __hash__(self) -> int:
        return hash(self.key())

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, MecabEntry):
            return NotImplemented
        return self.key() == other.key()


class DictionaryMerger:
    """Merge multiple MeCab dictionaries."""

    def __init__(
        self,
        strategy: str = "min_cost",
        deduplicate: bool = True
    ) -> None:
        """
        Initialize merger.

        Args:
            strategy: Conflict resolution strategy
                - "min_cost": Choose entry with minimum cost
                - "max_cost": Choose entry with maximum cost
                - "first": Keep first occurrence
                - "last": Keep last occurrence
                - "avg_cost": Average the costs
            deduplicate: Remove duplicate entries
        """
        self.strategy = strategy
        self.deduplicate = deduplicate
        self.stats: dict[str, int] = Counter()

    def merge(
        self,
        input_files: list[Path],
        output_file: Path
    ) -> None:
        """Merge dictionary files."""
        logger.info(f"Merging {len(input_files)} dictionary files")
        logger.info(f"Strategy: {self.strategy}")
        logger.info(f"Deduplication: {self.deduplicate}")

        # Load all entries
        all_entries: list[MecabEntry] = []

        for input_file in input_files:
            logger.info(f"Loading: {input_file}")
            entries = list(self._load_dict(input_file))
            self.stats[f"loaded_{input_file.name}"] = len(entries)
            all_entries.extend(entries)

        logger.info(f"Total entries loaded: {len(all_entries):,}")

        # Deduplicate and resolve conflicts
        if self.deduplicate:
            merged = self._deduplicate(all_entries)
        else:
            merged = all_entries

        logger.info(f"Entries after processing: {len(merged):,}")

        # Sort by surface form for consistency
        merged.sort(key=lambda e: (e.surface, e.pos))

        # Write output
        logger.info(f"Writing merged dictionary: {output_file}")
        output_file.parent.mkdir(parents=True, exist_ok=True)

        with open(output_file, "w", encoding="utf-8", newline="") as f:
            self._write_dict(merged, f)

        self.stats["total_output"] = len(merged)
        self._print_stats()

    def _load_dict(self, input_file: Path) -> Iterator[MecabEntry]:
        """Load dictionary from CSV file."""
        try:
            with open(input_file, encoding="utf-8") as f:
                reader = csv.reader(f)
                for row in reader:
                    try:
                        yield MecabEntry.from_csv_row(row)
                    except ValueError as e:
                        logger.warning(f"Skipping invalid row: {e}")
        except Exception as e:
            logger.error(f"Error loading {input_file}: {e}")

    def _deduplicate(self, entries: list[MecabEntry]) -> list[MecabEntry]:
        """Deduplicate entries based on strategy."""
        logger.info("Deduplicating entries...")

        # Group by key
        groups: dict[tuple[str, str], list[MecabEntry]] = defaultdict(list)
        for entry in entries:
            groups[entry.key()].append(entry)

        self.stats["unique_keys"] = len(groups)
        self.stats["duplicates_found"] = len(entries) - len(groups)

        # Resolve conflicts
        merged: list[MecabEntry] = []

        for key, entry_list in groups.items():
            if len(entry_list) == 1:
                merged.append(entry_list[0])
            else:
                resolved = self._resolve_conflict(entry_list)
                merged.append(resolved)
                self.stats["conflicts_resolved"] += 1

        return merged

    def _resolve_conflict(self, entries: list[MecabEntry]) -> MecabEntry:
        """Resolve conflict between duplicate entries."""
        if self.strategy == "min_cost":
            return min(entries, key=lambda e: e.cost)
        elif self.strategy == "max_cost":
            return max(entries, key=lambda e: e.cost)
        elif self.strategy == "first":
            return entries[0]
        elif self.strategy == "last":
            return entries[-1]
        elif self.strategy == "avg_cost":
            # Average the costs
            avg_cost = sum(e.cost for e in entries) // len(entries)
            result = entries[0]  # Use first entry as base
            result.cost = avg_cost
            return result
        else:
            logger.warning(f"Unknown strategy: {self.strategy}, using first")
            return entries[0]

    def _write_dict(
        self,
        entries: list[MecabEntry],
        output_file: TextIO
    ) -> None:
        """Write dictionary to CSV file."""
        writer = csv.writer(output_file, lineterminator="\n")
        for entry in entries:
            writer.writerow(entry.to_csv_row())

    def _print_stats(self) -> None:
        """Print merge statistics."""
        logger.info("=" * 60)
        logger.info("Merge Statistics")
        logger.info("=" * 60)

        for key, value in sorted(self.stats.items()):
            logger.info(f"  {key:30s}: {value:,}")

        logger.info("=" * 60)


class DictionaryAnalyzer:
    """Analyze dictionary statistics."""

    def __init__(self) -> None:
        self.pos_dist: Counter[str] = Counter()
        self.length_dist: Counter[int] = Counter()
        self.cost_stats: list[int] = []

    def analyze(self, dict_file: Path) -> None:
        """Analyze dictionary file."""
        logger.info(f"Analyzing: {dict_file}")

        total_entries = 0

        with open(dict_file, encoding="utf-8") as f:
            reader = csv.reader(f)
            for row in reader:
                try:
                    entry = MecabEntry.from_csv_row(row)
                    total_entries += 1

                    self.pos_dist[entry.pos] += 1
                    self.length_dist[len(entry.surface)] += 1
                    self.cost_stats.append(entry.cost)
                except ValueError:
                    pass

        self._print_analysis(total_entries)

    def _print_analysis(self, total: int) -> None:
        """Print analysis results."""
        logger.info("=" * 60)
        logger.info("Dictionary Analysis")
        logger.info("=" * 60)
        logger.info(f"Total entries: {total:,}")
        logger.info("")

        logger.info("POS Distribution (Top 20):")
        for pos, count in self.pos_dist.most_common(20):
            pct = (count / total) * 100
            logger.info(f"  {pos:10s}: {count:8,} ({pct:5.2f}%)")
        logger.info("")

        logger.info("Length Distribution (Top 10):")
        for length, count in sorted(
            self.length_dist.items(),
            key=lambda x: x[1],
            reverse=True
        )[:10]:
            logger.info(f"  {length:2d} chars: {count:8,}")
        logger.info("")

        if self.cost_stats:
            import statistics
            logger.info("Cost Statistics:")
            logger.info(f"  Min:    {min(self.cost_stats):,}")
            logger.info(f"  Max:    {max(self.cost_stats):,}")
            logger.info(f"  Mean:   {statistics.mean(self.cost_stats):,.2f}")
            logger.info(f"  Median: {statistics.median(self.cost_stats):,.2f}")

        logger.info("=" * 60)


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Merge MeCab dictionary files",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Merge multiple dictionaries with default settings
  %(prog)s -i dict1.csv dict2.csv dict3.csv -o merged.csv

  # Merge with minimum cost strategy
  %(prog)s -i *.csv -o merged.csv --strategy min_cost

  # Analyze dictionary statistics
  %(prog)s --analyze dict.csv

  # Merge without deduplication
  %(prog)s -i dict1.csv dict2.csv -o merged.csv --no-deduplicate
        """
    )

    parser.add_argument(
        "-i", "--input",
        nargs="+",
        type=Path,
        help="Input dictionary CSV files"
    )
    parser.add_argument(
        "-o", "--output",
        type=Path,
        help="Output merged dictionary CSV file"
    )
    parser.add_argument(
        "--strategy",
        choices=["min_cost", "max_cost", "first", "last", "avg_cost"],
        default="min_cost",
        help="Conflict resolution strategy (default: min_cost)"
    )
    parser.add_argument(
        "--no-deduplicate",
        action="store_true",
        help="Disable deduplication"
    )
    parser.add_argument(
        "--analyze",
        type=Path,
        help="Analyze dictionary statistics only"
    )
    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Enable verbose logging"
    )

    args = parser.parse_args()

    if args.verbose:
        logger.setLevel(logging.DEBUG)

    # Analyze mode
    if args.analyze:
        if not args.analyze.exists():
            logger.error(f"File not found: {args.analyze}")
            return 1

        analyzer = DictionaryAnalyzer()
        analyzer.analyze(args.analyze)
        return 0

    # Merge mode
    if not args.input or not args.output:
        parser.print_help()
        return 1

    # Validate input files
    for input_file in args.input:
        if not input_file.exists():
            logger.error(f"Input file not found: {input_file}")
            return 1

    # Merge dictionaries
    merger = DictionaryMerger(
        strategy=args.strategy,
        deduplicate=not args.no_deduplicate
    )

    try:
        merger.merge(args.input, args.output)
        logger.info("Merge completed successfully!")
        return 0
    except Exception as e:
        logger.error(f"Merge failed: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
