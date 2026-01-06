#!/usr/bin/env python3
"""Basic tests for dict-expander tools.

Quick verification that all modules can be imported and basic functionality works.
"""

import sys
from pathlib import Path

# Add current directory to path
sys.path.insert(0, str(Path(__file__).parent))


def test_imports():
    """Test that all modules can be imported."""
    print("Testing module imports...")

    try:
        from utils.mecab_format import MecabEntry, format_mecab_line, parse_mecab_line
        print("  ✓ utils.mecab_format")

        from utils.korean_utils import (
            get_jongseong_marker,
            has_final_consonant,
            is_hangul,
            decompose_hangul,
            compose_hangul,
        )
        print("  ✓ utils.korean_utils")

        from validators.deduplicator import Deduplicator, deduplicate_entries
        print("  ✓ validators.deduplicator")

        from validators.pos_inference import POSInferencer, infer_pos_tag
        print("  ✓ validators.pos_inference")

        from validators.quality_checker import QualityChecker, ValidationResult
        print("  ✓ validators.quality_checker")

        from data_sources.wikipedia_fetcher import WikipediaFetcher
        print("  ✓ data_sources.wikipedia_fetcher")

        from data_sources.public_data_fetcher import PublicDataFetcher
        print("  ✓ data_sources.public_data_fetcher")

        print("\n✓ All imports successful!\n")
        return True

    except Exception as e:
        print(f"\n✗ Import failed: {e}\n")
        return False


def test_mecab_format():
    """Test MeCab format utilities."""
    print("Testing MeCab format utilities...")

    from utils.mecab_format import MecabEntry
    from utils.korean_utils import get_jongseong_marker

    try:
        # Create entry
        entry = MecabEntry(
            surface="서울",
            pos="NNP",
            semantic="지명",
            has_jongseong=get_jongseong_marker("서울"),
            reading="서울",
        )

        # Convert to CSV
        csv_line = entry.to_csv_line()
        expected = "서울,0,0,0,NNP,지명,T,서울,*,*,*,*"

        assert csv_line == expected, f"Expected: {expected}, Got: {csv_line}"
        print(f"  ✓ Created entry: {csv_line}")

        # Parse CSV
        parsed = MecabEntry.from_csv_line(csv_line)
        assert parsed.surface == "서울"
        assert parsed.pos == "NNP"
        assert parsed.has_jongseong == "T"
        print(f"  ✓ Parsed entry successfully")

        print("\n✓ MeCab format tests passed!\n")
        return True

    except Exception as e:
        print(f"\n✗ MeCab format test failed: {e}\n")
        return False


def test_korean_utils():
    """Test Korean text utilities."""
    print("Testing Korean text utilities...")

    from utils.korean_utils import (
        is_hangul,
        has_final_consonant,
        decompose_hangul,
        compose_hangul,
        get_jongseong_marker,
    )

    try:
        # Test is_hangul
        assert is_hangul("가") == True
        assert is_hangul("A") == False
        print("  ✓ is_hangul works")

        # Test final consonant detection
        assert has_final_consonant("한글") == True
        assert has_final_consonant("나무") == False
        print("  ✓ has_final_consonant works")

        # Test jongseong marker
        assert get_jongseong_marker("서울") == "T"  # 울 ends with ㄹ
        assert get_jongseong_marker("나무") == "F"  # 무 has no final consonant
        print("  ✓ get_jongseong_marker works")

        # Test decompose/compose
        cho, jung, jong = decompose_hangul("한")
        assert cho == "ㅎ"
        assert jung == "ㅏ"
        assert jong == "ㄴ"
        print("  ✓ decompose_hangul works")

        recomposed = compose_hangul("ㅎ", "ㅏ", "ㄴ")
        assert recomposed == "한"
        print("  ✓ compose_hangul works")

        print("\n✓ Korean utils tests passed!\n")
        return True

    except Exception as e:
        print(f"\n✗ Korean utils test failed: {e}\n")
        return False


def test_validators():
    """Test validation utilities."""
    print("Testing validators...")

    from utils.mecab_format import MecabEntry
    from utils.korean_utils import get_jongseong_marker
    from validators.deduplicator import deduplicate_entries
    from validators.pos_inference import infer_pos_tag
    from validators.quality_checker import QualityChecker

    try:
        # Test POS inference
        pos = infer_pos_tag("서울")
        assert pos in ["NNG", "NNP"]
        print(f"  ✓ POS inference: '서울' -> {pos}")

        # Test quality checker
        checker = QualityChecker()
        entry = MecabEntry(
            surface="테스트",
            pos="NNG",
            has_jongseong=get_jongseong_marker("테스트"),
            reading="테스트",
        )
        result = checker.validate_entry(entry)
        assert result.is_valid == True
        print(f"  ✓ Quality validation passed")

        # Test deduplication
        entries = [
            MecabEntry("서울", "NNP", "T", "서울"),
            MecabEntry("서울", "NNP", "T", "서울"),
            MecabEntry("부산", "NNP", "F", "부산"),
        ]
        unique, stats = deduplicate_entries(entries)
        assert len(unique) == 2
        print(f"  ✓ Deduplication: {len(entries)} -> {len(unique)} entries")

        print("\n✓ Validator tests passed!\n")
        return True

    except Exception as e:
        print(f"\n✗ Validator test failed: {e}\n")
        return False


def main():
    """Run all tests."""
    print("=" * 60)
    print("MeCab-Ko Dict-Expander - Basic Tests")
    print("=" * 60)
    print()

    results = []

    results.append(("Imports", test_imports()))
    results.append(("MeCab Format", test_mecab_format()))
    results.append(("Korean Utils", test_korean_utils()))
    results.append(("Validators", test_validators()))

    print("=" * 60)
    print("Test Summary")
    print("=" * 60)

    for name, passed in results:
        status = "✓ PASS" if passed else "✗ FAIL"
        print(f"{status}: {name}")

    all_passed = all(passed for _, passed in results)

    print()
    if all_passed:
        print("✓ All tests passed!")
        return 0
    else:
        print("✗ Some tests failed")
        return 1


if __name__ == "__main__":
    sys.exit(main())
