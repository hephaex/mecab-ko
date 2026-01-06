#!/usr/bin/env python3
"""
MeCab-Ko Neologism Extractor

Extracts neologisms (신조어) from Korean corpus data.
Uses various heuristics to identify new or uncommon words.
"""

from __future__ import annotations

import argparse
import csv
import json
import logging
import re
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Iterator, TextIO

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


@dataclass
class Neologism:
    """Neologism entry with metadata."""

    surface: str
    frequency: int
    pos: str
    contexts: list[str]
    first_seen: str | None = None
    pattern_type: str | None = None  # compound, loanword, abbreviation, etc.

    def to_dict(self) -> dict[str, str | int | list[str] | None]:
        """Convert to dictionary for JSON serialization."""
        return {
            "surface": self.surface,
            "frequency": self.frequency,
            "pos": self.pos,
            "contexts": self.contexts[:5],  # Limit context examples
            "first_seen": self.first_seen,
            "pattern_type": self.pattern_type,
        }


class NeologismExtractor:
    """Extract neologisms from corpus data."""

    # Common Korean syllables for pattern detection
    HANGUL_PATTERN = re.compile(r"[가-힣]+")
    ENGLISH_PATTERN = re.compile(r"[A-Za-z]+")
    NUMBER_PATTERN = re.compile(r"\d+")

    # Patterns that suggest neologisms
    NEOLOGISM_PATTERNS = {
        # 외래어/영어 혼용 (e.g., "먹방", "셀카")
        "loanword_compound": re.compile(r"[가-힣]+[A-Za-z]+|[A-Za-z]+[가-힣]+"),

        # 축약어 (e.g., "강추", "별다줄")
        "abbreviation": re.compile(r"^[가-힣]{2,4}$"),

        # 접두사/접미사 패턴 (e.g., "극혐", "~질")
        "affix_pattern": re.compile(r"^(극|핵|초|개|대|존|쌉)[가-힣]+|[가-힣]+(질|러|템|짱)$"),

        # 반복 패턴 (e.g., "쩝쩝", "방방")
        "repetition": re.compile(r"^(.{1,2})\1+$"),

        # 이모티콘/의태어 (e.g., "ㅋㅋ", "ㅠㅠ")
        "emoticon": re.compile(r"^[ㄱ-ㅎㅏ-ㅣ]{2,}$"),
    }

    # Common words to exclude (blacklist)
    COMMON_WORDS = {
        "이다", "있다", "없다", "하다", "되다", "것", "수", "등",
        "그", "이", "저", "나", "너", "우리", "저희",
        "그리고", "그러나", "하지만", "그래서",
    }

    def __init__(
        self,
        min_frequency: int = 3,
        max_frequency: int = 1000,
        min_length: int = 2,
        max_length: int = 10,
        use_reference_dict: bool = True,
        reference_dict_path: Path | None = None,
    ) -> None:
        self.min_frequency = min_frequency
        self.max_frequency = max_frequency
        self.min_length = min_length
        self.max_length = max_length
        self.use_reference_dict = use_reference_dict
        self.reference_dict: set[str] = set()

        if use_reference_dict and reference_dict_path:
            self._load_reference_dict(reference_dict_path)

        self.word_contexts: dict[str, list[str]] = defaultdict(list)
        self.word_dates: dict[str, str] = {}

    def _load_reference_dict(self, dict_path: Path) -> None:
        """Load reference dictionary for filtering known words."""
        logger.info(f"Loading reference dictionary: {dict_path}")

        try:
            with open(dict_path, encoding="utf-8") as f:
                for line in f:
                    parts = line.strip().split(",")
                    if parts:
                        self.reference_dict.add(parts[0])

            logger.info(f"Loaded {len(self.reference_dict):,} reference words")
        except Exception as e:
            logger.warning(f"Failed to load reference dictionary: {e}")

    def is_neologism_candidate(self, surface: str, pos: str) -> tuple[bool, str | None]:
        """Check if word is a neologism candidate."""
        # Length filter
        if not (self.min_length <= len(surface) <= self.max_length):
            return False, None

        # Exclude common words
        if surface in self.COMMON_WORDS:
            return False, None

        # Exclude if in reference dictionary
        if self.use_reference_dict and surface in self.reference_dict:
            return False, None

        # Check for neologism patterns
        for pattern_name, pattern in self.NEOLOGISM_PATTERNS.items():
            if pattern.match(surface):
                return True, pattern_name

        # Check POS tags that often indicate neologisms
        neologism_pos = {"NNG", "NNP", "MAG", "IC", "SL", "SN"}
        if pos in neologism_pos:
            # Check if contains unusual character combinations
            if self._has_unusual_patterns(surface):
                return True, "unusual_pattern"

        return False, None

    def _has_unusual_patterns(self, surface: str) -> bool:
        """Check for unusual character combination patterns."""
        # Mixed script (Hangul + English/Number)
        has_hangul = bool(self.HANGUL_PATTERN.search(surface))
        has_english = bool(self.ENGLISH_PATTERN.search(surface))
        has_number = bool(self.NUMBER_PATTERN.search(surface))

        # If mixed scripts, likely a neologism
        if sum([has_hangul, has_english, has_number]) > 1:
            return True

        # Check for uncommon syllable combinations
        # (This is a simplified heuristic)
        if has_hangul and len(surface) >= 3:
            # Check for repeated syllables
            for i in range(len(surface) - 1):
                if surface[i] == surface[i + 1]:
                    return True

        return False

    def extract_from_corpus(
        self,
        input_path: Path,
        corpus_format: str
    ) -> Iterator[Neologism]:
        """Extract neologisms from corpus."""
        logger.info(f"Extracting neologisms from {corpus_format} corpus: {input_path}")

        # Collect word frequencies and contexts
        word_freq: Counter[tuple[str, str]] = Counter()

        if corpus_format == "modu":
            self._collect_from_modu(input_path, word_freq)
        elif corpus_format == "sejong":
            self._collect_from_sejong(input_path, word_freq)
        elif corpus_format == "conllu":
            self._collect_from_conllu(input_path, word_freq)
        else:
            logger.error(f"Unknown corpus format: {corpus_format}")
            return

        logger.info(f"Collected {len(word_freq):,} unique (surface, POS) pairs")

        # Filter and yield neologisms
        neologisms_found = 0

        for (surface, pos), freq in word_freq.most_common():
            # Frequency filter
            if not (self.min_frequency <= freq <= self.max_frequency):
                continue

            # Check if neologism candidate
            is_candidate, pattern_type = self.is_neologism_candidate(surface, pos)

            if is_candidate:
                contexts = self.word_contexts.get(surface, [])
                first_seen = self.word_dates.get(surface)

                neologism = Neologism(
                    surface=surface,
                    frequency=freq,
                    pos=pos,
                    contexts=contexts,
                    first_seen=first_seen,
                    pattern_type=pattern_type,
                )

                neologisms_found += 1
                yield neologism

        logger.info(f"Found {neologisms_found:,} neologism candidates")

    def _collect_from_modu(
        self,
        input_path: Path,
        word_freq: Counter[tuple[str, str]]
    ) -> None:
        """Collect from 모두의 말뭉치."""
        if input_path.is_file():
            files = [input_path]
        else:
            files = list(input_path.glob("**/*.json"))

        for json_file in files:
            try:
                with open(json_file, encoding="utf-8") as f:
                    data = json.load(f)

                # Extract date from filename or metadata
                date_str = self._extract_date(json_file.stem)

                self._process_modu_data(data, word_freq, date_str)
            except Exception as e:
                logger.warning(f"Error processing {json_file}: {e}")

    def _process_modu_data(
        self,
        data: dict | list,
        word_freq: Counter[tuple[str, str]],
        date_str: str | None
    ) -> None:
        """Process Modu corpus data recursively."""
        if isinstance(data, dict):
            # Extract morphemes
            if "morpheme" in data:
                sentence_text = data.get("form", "")
                for morph in data["morpheme"]:
                    surface = morph.get("form", "")
                    pos = morph.get("label", "")
                    if surface and pos:
                        word_freq[(surface, pos)] += 1
                        if sentence_text:
                            self.word_contexts[surface].append(sentence_text)
                        if date_str and surface not in self.word_dates:
                            self.word_dates[surface] = date_str

            # Recurse into nested structures
            for value in data.values():
                if isinstance(value, (dict, list)):
                    self._process_modu_data(value, word_freq, date_str)

        elif isinstance(data, list):
            for item in data:
                self._process_modu_data(item, word_freq, date_str)

    def _collect_from_sejong(
        self,
        input_path: Path,
        word_freq: Counter[tuple[str, str]]
    ) -> None:
        """Collect from 세종 말뭉치."""
        import xml.etree.ElementTree as ET

        if input_path.is_file():
            files = [input_path]
        else:
            files = list(input_path.glob("**/*.xml"))

        for xml_file in files:
            try:
                tree = ET.parse(xml_file)
                root = tree.getroot()

                for morph in root.findall(".//morph"):
                    surface = morph.text
                    pos = morph.get("tag", morph.get("pos", ""))

                    if surface and pos:
                        word_freq[(surface, pos)] += 1

                        # Try to get sentence context
                        sentence = morph.find("..//..")
                        if sentence is not None and sentence.text:
                            self.word_contexts[surface].append(sentence.text)
            except Exception as e:
                logger.warning(f"Error processing {xml_file}: {e}")

    def _collect_from_conllu(
        self,
        input_path: Path,
        word_freq: Counter[tuple[str, str]]
    ) -> None:
        """Collect from CoNLL-U format."""
        if input_path.is_file():
            files = [input_path]
        else:
            files = list(input_path.glob("**/*.conllu"))

        for conllu_file in files:
            try:
                with open(conllu_file, encoding="utf-8") as f:
                    sentence_tokens = []

                    for line in f:
                        line = line.strip()

                        if not line:
                            # End of sentence
                            if sentence_tokens:
                                sentence_text = " ".join(sentence_tokens)
                                sentence_tokens = []
                        elif line.startswith("#"):
                            continue
                        else:
                            parts = line.split("\t")
                            if len(parts) >= 4 and "-" not in parts[0]:
                                surface = parts[1]
                                pos = parts[3]

                                word_freq[(surface, pos)] += 1
                                sentence_tokens.append(surface)
            except Exception as e:
                logger.warning(f"Error processing {conllu_file}: {e}")

    def _extract_date(self, filename: str) -> str | None:
        """Extract date from filename if present."""
        # Try to find date patterns like YYYYMMDD or YYYY-MM-DD
        date_patterns = [
            r"(\d{8})",           # YYYYMMDD
            r"(\d{4}-\d{2}-\d{2})",  # YYYY-MM-DD
            r"(\d{4}\d{2})",      # YYYYMM
        ]

        for pattern in date_patterns:
            match = re.search(pattern, filename)
            if match:
                return match.group(1)

        return None


def write_neologisms_json(
    neologisms: Iterator[Neologism],
    output_file: TextIO
) -> int:
    """Write neologisms to JSON format."""
    neo_list = [neo.to_dict() for neo in neologisms]

    json.dump(
        {
            "metadata": {
                "generated_at": datetime.now().isoformat(),
                "total_neologisms": len(neo_list),
            },
            "neologisms": neo_list,
        },
        output_file,
        ensure_ascii=False,
        indent=2
    )

    return len(neo_list)


def write_neologisms_csv(
    neologisms: Iterator[Neologism],
    output_file: TextIO
) -> int:
    """Write neologisms to CSV format."""
    writer = csv.writer(output_file, lineterminator="\n")
    writer.writerow([
        "surface", "frequency", "pos", "pattern_type",
        "first_seen", "example_context"
    ])

    count = 0
    for neo in neologisms:
        example = neo.contexts[0] if neo.contexts else ""
        writer.writerow([
            neo.surface,
            neo.frequency,
            neo.pos,
            neo.pattern_type or "",
            neo.first_seen or "",
            example
        ])
        count += 1

    return count


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Extract neologisms from Korean corpus",
        formatter_class=argparse.RawDescriptionHelpFormatter
    )

    parser.add_argument(
        "-f", "--format",
        choices=["modu", "sejong", "conllu"],
        required=True,
        help="Input corpus format"
    )
    parser.add_argument(
        "-i", "--input",
        type=Path,
        required=True,
        help="Input corpus file or directory"
    )
    parser.add_argument(
        "-o", "--output",
        type=Path,
        required=True,
        help="Output file (JSON or CSV)"
    )
    parser.add_argument(
        "--output-format",
        choices=["json", "csv"],
        default="json",
        help="Output format (default: json)"
    )
    parser.add_argument(
        "--min-freq",
        type=int,
        default=3,
        help="Minimum frequency (default: 3)"
    )
    parser.add_argument(
        "--max-freq",
        type=int,
        default=1000,
        help="Maximum frequency (default: 1000)"
    )
    parser.add_argument(
        "--min-length",
        type=int,
        default=2,
        help="Minimum word length (default: 2)"
    )
    parser.add_argument(
        "--max-length",
        type=int,
        default=10,
        help="Maximum word length (default: 10)"
    )
    parser.add_argument(
        "--reference-dict",
        type=Path,
        help="Reference dictionary to exclude known words"
    )
    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Enable verbose logging"
    )

    args = parser.parse_args()

    if args.verbose:
        logger.setLevel(logging.DEBUG)

    # Validate input
    if not args.input.exists():
        logger.error(f"Input path does not exist: {args.input}")
        return 1

    # Create output directory
    args.output.parent.mkdir(parents=True, exist_ok=True)

    # Extract neologisms
    extractor = NeologismExtractor(
        min_frequency=args.min_freq,
        max_frequency=args.max_freq,
        min_length=args.min_length,
        max_length=args.max_length,
        use_reference_dict=args.reference_dict is not None,
        reference_dict_path=args.reference_dict,
    )

    neologisms = extractor.extract_from_corpus(args.input, args.format)

    # Write output
    logger.info(f"Writing neologisms to: {args.output}")

    with open(args.output, "w", encoding="utf-8", newline="") as f:
        if args.output_format == "json":
            count = write_neologisms_json(neologisms, f)
        else:
            count = write_neologisms_csv(neologisms, f)

    logger.info(f"Successfully wrote {count:,} neologisms to {args.output}")
    logger.info("Done!")

    return 0


if __name__ == "__main__":
    sys.exit(main())
