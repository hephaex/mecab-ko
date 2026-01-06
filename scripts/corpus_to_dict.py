#!/usr/bin/env python3
"""
MeCab-Ko Corpus to Dictionary Converter

Converts various Korean corpus formats to MeCab dictionary format.
Supports:
- 모두의 말뭉치 (Modu Corpus) JSON format
- 세종 말뭉치 (Sejong Corpus) XML format
- CoNLL-U format
"""

from __future__ import annotations

import argparse
import csv
import json
import logging
import sys
import xml.etree.ElementTree as ET
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterator, TextIO

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


@dataclass
class DictEntry:
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

    def to_csv_row(self) -> list[str]:
        """Convert to MeCab CSV format."""
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


@dataclass
class CorpusStats:
    """Statistics for corpus processing."""

    total_tokens: int = 0
    unique_surfaces: int = 0
    pos_distribution: dict[str, int] = field(default_factory=Counter)
    length_distribution: dict[int, int] = field(default_factory=Counter)
    frequency: dict[tuple[str, str], int] = field(default_factory=Counter)

    def update(self, surface: str, pos: str) -> None:
        """Update statistics with new token."""
        self.total_tokens += 1
        self.pos_distribution[pos] += 1
        self.length_distribution[len(surface)] += 1
        self.frequency[(surface, pos)] += 1

    def finalize(self) -> None:
        """Finalize statistics calculation."""
        self.unique_surfaces = len(self.frequency)

    def print_summary(self) -> None:
        """Print statistics summary."""
        logger.info("=" * 60)
        logger.info("Corpus Statistics Summary")
        logger.info("=" * 60)
        logger.info(f"Total tokens: {self.total_tokens:,}")
        logger.info(f"Unique (surface, POS) pairs: {self.unique_surfaces:,}")
        logger.info("")
        logger.info("POS Distribution:")
        for pos, count in sorted(
            self.pos_distribution.items(),
            key=lambda x: x[1],
            reverse=True
        )[:20]:
            percentage = (count / self.total_tokens) * 100
            logger.info(f"  {pos:20s}: {count:8,} ({percentage:5.2f}%)")
        logger.info("")
        logger.info("Length Distribution:")
        for length, count in sorted(self.length_distribution.items())[:10]:
            logger.info(f"  {length:2d} chars: {count:8,}")
        logger.info("=" * 60)


class CorpusParser:
    """Base class for corpus parsers."""

    def __init__(self, min_frequency: int = 1) -> None:
        self.min_frequency = min_frequency
        self.stats = CorpusStats()

    def parse(self, input_path: Path) -> Iterator[DictEntry]:
        """Parse corpus and yield dictionary entries."""
        raise NotImplementedError

    def calculate_cost(self, frequency: int) -> int:
        """Calculate cost based on frequency (higher frequency = lower cost)."""
        # Cost formula: base_cost - log(frequency) * scale
        # This is a simplified version; actual MeCab uses more complex formula
        base_cost = 10000
        scale = 1000
        import math
        return max(1, int(base_cost - math.log(frequency + 1) * scale))


class ModuCorpusParser(CorpusParser):
    """Parser for 모두의 말뭉치 JSON format."""

    # Map 모두의 말뭉치 POS tags to MeCab-ko tags
    POS_MAP = {
        # 체언 (Nominals)
        "NNG": "NNG",  # 일반 명사
        "NNP": "NNP",  # 고유 명사
        "NNB": "NNB",  # 의존 명사
        "NP": "NP",    # 대명사
        "NR": "NR",    # 수사

        # 용언 (Predicates)
        "VV": "VV",    # 동사
        "VA": "VA",    # 형용사
        "VX": "VX",    # 보조 용언
        "VCP": "VCP",  # 긍정 지정사
        "VCN": "VCN",  # 부정 지정사

        # 관형사, 부사
        "MM": "MM",    # 관형사
        "MAG": "MAG",  # 일반 부사
        "MAJ": "MAJ",  # 접속 부사

        # 조사
        "JKS": "JKS",  # 주격 조사
        "JKC": "JKC",  # 보격 조사
        "JKG": "JKG",  # 관형격 조사
        "JKO": "JKO",  # 목적격 조사
        "JKB": "JKB",  # 부사격 조사
        "JKV": "JKV",  # 호격 조사
        "JKQ": "JKQ",  # 인용격 조사
        "JX": "JX",    # 보조사
        "JC": "JC",    # 접속 조사

        # 어미
        "EP": "EP",    # 선어말 어미
        "EF": "EF",    # 종결 어미
        "EC": "EC",    # 연결 어미
        "ETN": "ETN",  # 명사형 전성 어미
        "ETM": "ETM",  # 관형형 전성 어미

        # 접두사, 접미사
        "XPN": "XPN",  # 체언 접두사
        "XSN": "XSN",  # 명사 파생 접미사
        "XSV": "XSV",  # 동사 파생 접미사
        "XSA": "XSA",  # 형용사 파생 접미사
        "XR": "XR",    # 어근

        # 기호
        "SF": "SF",    # 마침표, 물음표, 느낌표
        "SP": "SP",    # 쉼표, 가운뎃점, 콜론, 빗금
        "SS": "SS",    # 따옴표, 괄호표, 줄표
        "SE": "SE",    # 줄임표
        "SO": "SO",    # 붙임표
        "SW": "SW",    # 기타 기호

        # 외국어, 한자, 기타
        "SL": "SL",    # 외국어
        "SH": "SH",    # 한자
        "SN": "SN",    # 숫자
        "IC": "IC",    # 감탄사
    }

    def parse(self, input_path: Path) -> Iterator[DictEntry]:
        """Parse Modu corpus JSON files."""
        logger.info(f"Parsing Modu corpus: {input_path}")

        # First pass: collect frequency statistics
        logger.info("First pass: collecting frequency statistics...")
        self._collect_frequencies(input_path)

        # Second pass: generate dictionary entries
        logger.info("Second pass: generating dictionary entries...")
        entries_generated = 0

        for surface, pos in sorted(self.stats.frequency.keys()):
            freq = self.stats.frequency[(surface, pos)]

            if freq < self.min_frequency:
                continue

            # Map POS tag
            mecab_pos = self.POS_MAP.get(pos, pos)

            # Calculate cost based on frequency
            cost = self.calculate_cost(freq)

            # Create entry (using simple IDs for now)
            entry = DictEntry(
                surface=surface,
                left_id=0,  # To be filled by mecab-dict-index
                right_id=0,
                cost=cost,
                pos=mecab_pos,
                base_form=surface,
                reading="*",
                pronunciation="*"
            )

            entries_generated += 1
            yield entry

        logger.info(f"Generated {entries_generated:,} dictionary entries")
        self.stats.finalize()
        self.stats.print_summary()

    def _collect_frequencies(self, input_path: Path) -> None:
        """Collect frequency statistics from corpus."""
        if input_path.is_file():
            self._process_json_file(input_path)
        else:
            # Process all JSON files in directory
            for json_file in input_path.glob("**/*.json"):
                self._process_json_file(json_file)

    def _process_json_file(self, file_path: Path) -> None:
        """Process a single JSON file."""
        try:
            with open(file_path, encoding="utf-8") as f:
                data = json.load(f)

            # Handle different JSON structures
            if isinstance(data, dict):
                self._process_json_data(data)
            elif isinstance(data, list):
                for item in data:
                    self._process_json_data(item)
        except json.JSONDecodeError as e:
            logger.warning(f"Failed to parse JSON file {file_path}: {e}")
        except Exception as e:
            logger.error(f"Error processing {file_path}: {e}")

    def _process_json_data(self, data: dict[str, Any]) -> None:
        """Process JSON data structure."""
        # Handle various Modu corpus JSON structures
        if "document" in data:
            for doc in data["document"]:
                self._process_document(doc)
        elif "sentence" in data:
            for sent in data["sentence"]:
                self._process_sentence(sent)
        elif "word" in data:
            self._process_sentence(data)

    def _process_document(self, doc: dict[str, Any]) -> None:
        """Process document structure."""
        if "paragraph" in doc:
            for para in doc["paragraph"]:
                if "sentence" in para:
                    for sent in para["sentence"]:
                        self._process_sentence(sent)
        elif "sentence" in doc:
            for sent in doc["sentence"]:
                self._process_sentence(sent)

    def _process_sentence(self, sent: dict[str, Any]) -> None:
        """Process sentence and extract morphemes."""
        if "word" in sent:
            for word in sent["word"]:
                self._process_word(word)
        elif "morpheme" in sent:
            for morph in sent["morpheme"]:
                self._process_morpheme(morph)

    def _process_word(self, word: dict[str, Any]) -> None:
        """Process word and extract morphemes."""
        if "morpheme" in word:
            for morph in word["morpheme"]:
                self._process_morpheme(morph)

    def _process_morpheme(self, morph: dict[str, Any]) -> None:
        """Process individual morpheme."""
        surface = morph.get("form", morph.get("surface", ""))
        pos = morph.get("label", morph.get("pos", ""))

        if surface and pos:
            self.stats.update(surface, pos)


class SejongCorpusParser(CorpusParser):
    """Parser for 세종 말뭉치 XML format."""

    def parse(self, input_path: Path) -> Iterator[DictEntry]:
        """Parse Sejong corpus XML files."""
        logger.info(f"Parsing Sejong corpus: {input_path}")

        # First pass: collect frequencies
        logger.info("First pass: collecting frequency statistics...")
        self._collect_frequencies(input_path)

        # Second pass: generate entries
        logger.info("Second pass: generating dictionary entries...")
        entries_generated = 0

        for surface, pos in sorted(self.stats.frequency.keys()):
            freq = self.stats.frequency[(surface, pos)]

            if freq < self.min_frequency:
                continue

            cost = self.calculate_cost(freq)

            entry = DictEntry(
                surface=surface,
                left_id=0,
                right_id=0,
                cost=cost,
                pos=pos,
                base_form=surface,
                reading="*",
                pronunciation="*"
            )

            entries_generated += 1
            yield entry

        logger.info(f"Generated {entries_generated:,} dictionary entries")
        self.stats.finalize()
        self.stats.print_summary()

    def _collect_frequencies(self, input_path: Path) -> None:
        """Collect frequency statistics from corpus."""
        if input_path.is_file():
            self._process_xml_file(input_path)
        else:
            for xml_file in input_path.glob("**/*.xml"):
                self._process_xml_file(xml_file)

    def _process_xml_file(self, file_path: Path) -> None:
        """Process a single XML file."""
        try:
            tree = ET.parse(file_path)
            root = tree.getroot()

            # Find all morpheme elements
            for morph in root.findall(".//morph"):
                surface = morph.text
                pos = morph.get("tag", morph.get("pos", ""))

                if surface and pos:
                    self.stats.update(surface, pos)
        except ET.ParseError as e:
            logger.warning(f"Failed to parse XML file {file_path}: {e}")
        except Exception as e:
            logger.error(f"Error processing {file_path}: {e}")


class CoNLLUParser(CorpusParser):
    """Parser for CoNLL-U format."""

    def parse(self, input_path: Path) -> Iterator[DictEntry]:
        """Parse CoNLL-U format files."""
        logger.info(f"Parsing CoNLL-U corpus: {input_path}")

        # First pass: collect frequencies
        logger.info("First pass: collecting frequency statistics...")
        self._collect_frequencies(input_path)

        # Second pass: generate entries
        logger.info("Second pass: generating dictionary entries...")
        entries_generated = 0

        for surface, pos in sorted(self.stats.frequency.keys()):
            freq = self.stats.frequency[(surface, pos)]

            if freq < self.min_frequency:
                continue

            cost = self.calculate_cost(freq)

            entry = DictEntry(
                surface=surface,
                left_id=0,
                right_id=0,
                cost=cost,
                pos=pos,
                base_form=surface,
                reading="*",
                pronunciation="*"
            )

            entries_generated += 1
            yield entry

        logger.info(f"Generated {entries_generated:,} dictionary entries")
        self.stats.finalize()
        self.stats.print_summary()

    def _collect_frequencies(self, input_path: Path) -> None:
        """Collect frequency statistics from corpus."""
        if input_path.is_file():
            self._process_conllu_file(input_path)
        else:
            for conllu_file in input_path.glob("**/*.conllu"):
                self._process_conllu_file(conllu_file)

    def _process_conllu_file(self, file_path: Path) -> None:
        """Process a single CoNLL-U file."""
        try:
            with open(file_path, encoding="utf-8") as f:
                for line in f:
                    line = line.strip()

                    # Skip comments and empty lines
                    if not line or line.startswith("#"):
                        continue

                    # Parse token line
                    parts = line.split("\t")
                    if len(parts) >= 4:
                        # ID, FORM, LEMMA, UPOS, ...
                        surface = parts[1]
                        pos = parts[3]

                        if surface and pos and "-" not in parts[0]:
                            self.stats.update(surface, pos)
        except Exception as e:
            logger.error(f"Error processing {file_path}: {e}")


def write_mecab_dict(
    entries: Iterator[DictEntry],
    output_file: TextIO
) -> int:
    """Write dictionary entries to MeCab CSV format."""
    writer = csv.writer(output_file, lineterminator="\n")
    count = 0

    for entry in entries:
        writer.writerow(entry.to_csv_row())
        count += 1

    return count


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Convert Korean corpus to MeCab dictionary format",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Convert Modu corpus to MeCab format
  %(prog)s -f modu -i corpus/modu/ -o dict/modu.csv

  # Convert Sejong corpus with minimum frequency filter
  %(prog)s -f sejong -i corpus/sejong.xml -o dict/sejong.csv --min-freq 3

  # Convert CoNLL-U corpus
  %(prog)s -f conllu -i corpus/data.conllu -o dict/output.csv
        """
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
        help="Output MeCab dictionary CSV file"
    )
    parser.add_argument(
        "--min-freq",
        type=int,
        default=1,
        help="Minimum frequency threshold (default: 1)"
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

    # Create output directory if needed
    args.output.parent.mkdir(parents=True, exist_ok=True)

    # Select parser
    parser_cls: type[CorpusParser]
    if args.format == "modu":
        parser_cls = ModuCorpusParser
    elif args.format == "sejong":
        parser_cls = SejongCorpusParser
    elif args.format == "conllu":
        parser_cls = CoNLLUParser
    else:
        logger.error(f"Unknown format: {args.format}")
        return 1

    # Parse and convert
    corpus_parser = parser_cls(min_frequency=args.min_freq)
    entries = corpus_parser.parse(args.input)

    # Write output
    logger.info(f"Writing dictionary to: {args.output}")
    with open(args.output, "w", encoding="utf-8", newline="") as f:
        count = write_mecab_dict(entries, f)

    logger.info(f"Successfully wrote {count:,} entries to {args.output}")
    logger.info("Done!")

    return 0


if __name__ == "__main__":
    sys.exit(main())
