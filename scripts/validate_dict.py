#!/usr/bin/env python3
"""MeCab-Ko 도메인 사전 CSV 검증 스크립트.

Usage:
    python scripts/validate_dict.py data/domain-dic/news/news-nnp.csv
    python scripts/validate_dict.py data/domain-dic/**/*.csv
"""

import csv
import sys
from pathlib import Path

VALID_POS = {"NNP", "NNG", "VV", "VA", "MAG", "IC", "XR", "XSN", "XSV", "XSA"}
VALID_BUNRYU = {"*", "인명", "지명", "단체명", "브랜드", "국가명", "매체명"}
MAX_COST = 0
MIN_COST = -32000
EXPECTED_FIELDS = 12


def validate_file(filepath: str) -> list[str]:
    errors = []
    path = Path(filepath)

    if not path.exists():
        return [f"File not found: {filepath}"]

    if path.suffix != ".csv":
        return [f"Not a CSV file: {filepath}"]

    seen_surfaces = set()
    line_num = 0
    entry_count = 0

    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line_num += 1
            line = line.strip()

            if not line or line.startswith("#"):
                continue

            entry_count += 1
            fields = line.split(",")

            if len(fields) != EXPECTED_FIELDS:
                errors.append(
                    f"L{line_num}: Expected {EXPECTED_FIELDS} fields, got {len(fields)}: {line[:60]}"
                )
                continue

            surface = fields[0]
            left_id = fields[1]
            right_id = fields[2]
            cost_str = fields[3]
            pos = fields[4]
            bunryu = fields[5]

            if not surface:
                errors.append(f"L{line_num}: Empty surface")

            if surface in seen_surfaces:
                errors.append(f"L{line_num}: Duplicate surface: {surface}")
            seen_surfaces.add(surface)

            try:
                cost = int(cost_str)
                if cost < MIN_COST or cost > MAX_COST:
                    errors.append(
                        f"L{line_num}: Cost {cost} out of range [{MIN_COST}, {MAX_COST}]: {surface}"
                    )
            except ValueError:
                errors.append(f"L{line_num}: Invalid cost: {cost_str}")

            if pos not in VALID_POS:
                errors.append(f"L{line_num}: Unknown POS: {pos} for {surface}")

    if entry_count == 0:
        errors.append("No entries found (empty file)")

    return errors


def main():
    if len(sys.argv) < 2:
        print("Usage: python validate_dict.py <csv_file> [csv_file ...]")
        sys.exit(1)

    total_errors = 0

    for filepath in sys.argv[1:]:
        errors = validate_file(filepath)
        if errors:
            print(f"\n❌ {filepath}: {len(errors)} error(s)")
            for err in errors[:20]:
                print(f"  {err}")
            if len(errors) > 20:
                print(f"  ... and {len(errors) - 20} more")
            total_errors += len(errors)
        else:
            print(f"✅ {filepath}: OK")

    if total_errors > 0:
        print(f"\nTotal: {total_errors} error(s)")
        sys.exit(1)
    else:
        print("\nAll files valid.")
        sys.exit(0)


if __name__ == "__main__":
    main()
