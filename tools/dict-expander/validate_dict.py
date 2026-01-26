#!/usr/bin/env python3
"""
MeCab 사전 검증 도구

IT 용어 사전의 품질을 검증하고 통계를 생성합니다.
"""

import csv
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any
import json


@dataclass
class ValidationResult:
    """검증 결과."""

    file_path: Path
    total_entries: int = 0
    duplicates: list[str] = field(default_factory=list)
    invalid_pos: list[tuple[str, str]] = field(default_factory=list)
    empty_readings: list[str] = field(default_factory=list)
    format_errors: list[tuple[int, str]] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)

    @property
    def is_valid(self) -> bool:
        """검증 통과 여부."""
        return (
            not self.format_errors
            and not self.invalid_pos
        )

    @property
    def has_warnings(self) -> bool:
        """경고 존재 여부."""
        return bool(self.duplicates or self.empty_readings or self.warnings)


class MeCabDictValidator:
    """MeCab 사전 검증기."""

    # MeCab 한국어 품사 태그
    VALID_POS_TAGS = {
        # 체언
        "NNG",  # 일반 명사
        "NNP",  # 고유 명사
        "NNB",  # 의존 명사
        "NP",   # 대명사
        "NR",   # 수사
        # 용언
        "VV",   # 동사
        "VA",   # 형용사
        "VX",   # 보조 용언
        "VCP",  # 긍정 지정사
        "VCN",  # 부정 지정사
        # 관형사
        "MM",   # 관형사
        # 부사
        "MAG",  # 일반 부사
        "MAJ",  # 접속 부사
        # 감탄사
        "IC",   # 감탄사
        # 조사
        "JKS",  # 주격 조사
        "JKC",  # 보격 조사
        "JKG",  # 관형격 조사
        "JKO",  # 목적격 조사
        "JKB",  # 부사격 조사
        "JKV",  # 호격 조사
        "JKQ",  # 인용격 조사
        "JX",   # 보조사
        "JC",   # 접속 조사
        # 어미
        "EP",   # 선어말 어미
        "EF",   # 종결 어미
        "EC",   # 연결 어미
        "ETN",  # 명사형 전성 어미
        "ETM",  # 관형형 전성 어미
        # 접두사
        "XPN",  # 체언 접두사
        # 접미사
        "XSN",  # 명사 파생 접미사
        "XSV",  # 동사 파생 접미사
        "XSA",  # 형용사 파생 접미사
        # 어근
        "XR",   # 어근
        # 부호
        "SF",   # 마침표, 물음표, 느낌표
        "SP",   # 쉼표, 가운뎃점, 콜론, 빗금
        "SS",   # 따옴표, 괄호표, 줄표
        "SE",   # 줄임표
        "SO",   # 붙임표
        "SW",   # 기타 기호
        # 외국어, 한자, 기타
        "SL",   # 외국어
        "SH",   # 한자
        "SN",   # 숫자
        "NA",   # 분석 불능
    }

    # IT 용어에 적합한 품사
    IT_TERM_POS = {"NNG", "NNP", "SL"}

    def __init__(self, strict_mode: bool = False):
        self.strict_mode = strict_mode

    def validate_file(self, file_path: Path) -> ValidationResult:
        """CSV 파일 검증."""
        result = ValidationResult(file_path=file_path)

        if not file_path.exists():
            result.format_errors.append((0, f"File not found: {file_path}"))
            return result

        surface_counter = Counter()

        try:
            with file_path.open("r", encoding="utf-8") as f:
                reader = csv.reader(f)
                for line_num, row in enumerate(reader, start=1):
                    # 포맷 검증
                    if len(row) < 13:
                        result.format_errors.append(
                            (line_num, f"Expected 13 fields, got {len(row)}")
                        )
                        continue

                    result.total_entries += 1

                    surface = row[0]
                    cost = row[3]
                    pos = row[4]
                    lemma = row[10]
                    reading = row[11]
                    pronunciation = row[12]

                    # 표면형 중복 체크
                    surface_counter[surface] += 1

                    # 비용 검증
                    try:
                        cost_value = int(cost)
                        if cost_value > 0:
                            result.warnings.append(
                                f"Line {line_num}: Positive cost ({cost_value}) for '{surface}'"
                            )
                    except ValueError:
                        result.format_errors.append(
                            (line_num, f"Invalid cost value: '{cost}'")
                        )

                    # 품사 검증
                    if pos not in self.VALID_POS_TAGS:
                        result.invalid_pos.append((surface, pos))

                    # IT 용어 품사 권장사항
                    if self.strict_mode and pos not in self.IT_TERM_POS:
                        result.warnings.append(
                            f"Line {line_num}: Unusual POS '{pos}' for IT term '{surface}'"
                        )

                    # 읽기 필드 검증
                    if not reading or reading == "*":
                        result.empty_readings.append(surface)

                    # 표면형과 원형 일관성
                    if lemma != surface and lemma != "*":
                        result.warnings.append(
                            f"Line {line_num}: Lemma '{lemma}' differs from surface '{surface}'"
                        )

        except Exception as e:
            result.format_errors.append((0, f"Error reading file: {e}"))

        # 중복 항목 수집
        result.duplicates = [
            surface for surface, count in surface_counter.items() if count > 1
        ]

        return result

    def validate_directory(self, dir_path: Path) -> dict[str, ValidationResult]:
        """디렉토리 내 모든 CSV 파일 검증."""
        results = {}

        csv_files = sorted(dir_path.glob("*.csv"))
        if not csv_files:
            print(f"No CSV files found in {dir_path}")
            return results

        for csv_file in csv_files:
            result = self.validate_file(csv_file)
            results[csv_file.stem] = result

        return results

    def generate_report(
        self, results: dict[str, ValidationResult]
    ) -> dict[str, Any]:
        """검증 리포트 생성."""
        report = {
            "summary": {
                "total_files": len(results),
                "valid_files": sum(1 for r in results.values() if r.is_valid),
                "files_with_warnings": sum(1 for r in results.values() if r.has_warnings),
                "total_entries": sum(r.total_entries for r in results.values()),
                "total_duplicates": sum(len(r.duplicates) for r in results.values()),
                "total_format_errors": sum(
                    len(r.format_errors) for r in results.values()
                ),
            },
            "files": {},
        }

        for name, result in results.items():
            file_report = {
                "path": str(result.file_path),
                "entries": result.total_entries,
                "valid": result.is_valid,
                "has_warnings": result.has_warnings,
                "issues": {
                    "duplicates": len(result.duplicates),
                    "invalid_pos": len(result.invalid_pos),
                    "empty_readings": len(result.empty_readings),
                    "format_errors": len(result.format_errors),
                    "warnings": len(result.warnings),
                },
            }

            if result.duplicates:
                file_report["duplicate_list"] = result.duplicates[:10]  # 상위 10개
            if result.invalid_pos:
                file_report["invalid_pos_list"] = result.invalid_pos[:10]
            if result.format_errors:
                file_report["format_error_list"] = [
                    {"line": line, "error": error}
                    for line, error in result.format_errors[:10]
                ]

            report["files"][name] = file_report

        return report

    def print_report(self, results: dict[str, ValidationResult]) -> None:
        """검증 결과 출력."""
        print("\n" + "=" * 70)
        print("MeCab Dictionary Validation Report")
        print("=" * 70)

        total_entries = sum(r.total_entries for r in results.values())
        valid_files = sum(1 for r in results.values() if r.is_valid)
        files_with_warnings = sum(1 for r in results.values() if r.has_warnings)

        print(f"\nOverall Summary:")
        print(f"  Total files: {len(results)}")
        print(f"  Valid files: {valid_files}")
        print(f"  Files with warnings: {files_with_warnings}")
        print(f"  Total entries: {total_entries}")

        # 파일별 상세 결과
        print(f"\nPer-file Results:")
        for name, result in results.items():
            status_icon = "✓" if result.is_valid else "✗"
            warning_icon = "⚠" if result.has_warnings else " "

            print(f"\n  [{status_icon}] {name} ({result.total_entries} entries) {warning_icon}")

            if result.format_errors:
                print(f"    Format errors: {len(result.format_errors)}")
                for line, error in result.format_errors[:3]:
                    print(f"      Line {line}: {error}")

            if result.invalid_pos:
                print(f"    Invalid POS tags: {len(result.invalid_pos)}")
                for surface, pos in result.invalid_pos[:3]:
                    print(f"      '{surface}': {pos}")

            if result.duplicates:
                print(f"    Duplicates: {len(result.duplicates)}")
                for surface in result.duplicates[:3]:
                    print(f"      {surface}")

            if result.empty_readings:
                print(f"    Empty readings: {len(result.empty_readings)}")

            if result.warnings:
                print(f"    Warnings: {len(result.warnings)}")
                for warning in result.warnings[:3]:
                    print(f"      {warning}")

        print("\n" + "=" * 70)

    def remove_duplicates(
        self, file_path: Path, output_path: Path | None = None
    ) -> int:
        """중복 항목 제거."""
        if output_path is None:
            output_path = file_path.with_suffix(".dedup.csv")

        seen = set()
        kept = []
        removed = 0

        with file_path.open("r", encoding="utf-8") as f:
            reader = csv.reader(f)
            for row in reader:
                if not row:
                    continue

                surface = row[0]
                if surface in seen:
                    removed += 1
                    continue

                seen.add(surface)
                kept.append(row)

        # 출력 파일 작성
        with output_path.open("w", encoding="utf-8") as f:
            writer = csv.writer(f)
            writer.writerows(kept)

        return removed


def main() -> None:
    """메인 함수."""
    import argparse

    parser = argparse.ArgumentParser(
        description="MeCab 사전 CSV 파일 검증 및 품질 관리"
    )
    parser.add_argument(
        "input",
        type=Path,
        help="검증할 CSV 파일 또는 디렉토리",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="엄격한 검증 모드 (IT 용어 품사 제한)",
    )
    parser.add_argument(
        "--remove-duplicates",
        action="store_true",
        help="중복 항목 제거",
    )
    parser.add_argument(
        "--output-report",
        type=Path,
        help="JSON 리포트 출력 경로",
    )

    args = parser.parse_args()

    validator = MeCabDictValidator(strict_mode=args.strict)

    # 검증 실행
    if args.input.is_dir():
        results = validator.validate_directory(args.input)
    else:
        result = validator.validate_file(args.input)
        results = {args.input.stem: result}

    # 결과 출력
    validator.print_report(results)

    # JSON 리포트 저장
    if args.output_report:
        report = validator.generate_report(results)
        with args.output_report.open("w", encoding="utf-8") as f:
            json.dump(report, f, ensure_ascii=False, indent=2)
        print(f"\nDetailed report saved to: {args.output_report}")

    # 중복 제거
    if args.remove_duplicates:
        print("\nRemoving duplicates...")
        for name, result in results.items():
            if result.duplicates:
                removed = validator.remove_duplicates(result.file_path)
                print(f"  {name}: Removed {removed} duplicate entries")

    # 종료 코드
    all_valid = all(r.is_valid for r in results.values())
    return 0 if all_valid else 1


if __name__ == "__main__":
    exit(main())
