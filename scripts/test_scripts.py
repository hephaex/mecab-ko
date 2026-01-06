#!/usr/bin/env python3
"""
Simple tests for MeCab-Ko corpus processing scripts.

Run with: python3 test_scripts.py
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def create_sample_corpus(corpus_dir: Path) -> None:
    """Create a minimal sample corpus for testing."""
    sample_data = {
        "document": [
            {
                "id": "TEST001",
                "sentence": [
                    {
                        "id": 1,
                        "form": "테스트 문장입니다.",
                        "word": [
                            {
                                "form": "테스트",
                                "morpheme": [
                                    {"form": "테스트", "label": "NNG"}
                                ]
                            },
                            {
                                "form": "문장입니다",
                                "morpheme": [
                                    {"form": "문장", "label": "NNG"},
                                    {"form": "이", "label": "VCP"},
                                    {"form": "ㅂ니다", "label": "EF"}
                                ]
                            },
                            {
                                "form": ".",
                                "morpheme": [
                                    {"form": ".", "label": "SF"}
                                ]
                            }
                        ]
                    }
                ]
            }
        ]
    }

    corpus_file = corpus_dir / "sample.json"
    with open(corpus_file, "w", encoding="utf-8") as f:
        json.dump(sample_data, f, ensure_ascii=False, indent=2)

    print(f"✓ Created sample corpus: {corpus_file}")


def test_corpus_to_dict(script_dir: Path, corpus_dir: Path, output_dir: Path) -> bool:
    """Test corpus_to_dict.py script."""
    print("\n[TEST] corpus_to_dict.py")
    print("-" * 60)

    output_file = output_dir / "test_dict.csv"
    script = script_dir / "corpus_to_dict.py"

    cmd = [
        str(script),
        "-f", "modu",
        "-i", str(corpus_dir),
        "-o", str(output_file),
        "--min-freq", "1"
    ]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=30
        )

        if result.returncode != 0:
            print(f"✗ Failed with return code {result.returncode}")
            print(f"STDERR: {result.stderr}")
            return False

        if not output_file.exists():
            print("✗ Output file not created")
            return False

        # Check output has entries
        with open(output_file, encoding="utf-8") as f:
            lines = f.readlines()

        if len(lines) < 3:  # Should have at least a few entries
            print(f"✗ Insufficient entries: {len(lines)}")
            return False

        print(f"✓ Generated {len(lines)} dictionary entries")
        print(f"✓ Output file: {output_file}")
        return True

    except subprocess.TimeoutExpired:
        print("✗ Script timed out")
        return False
    except Exception as e:
        print(f"✗ Exception: {e}")
        return False


def test_extract_neologisms(
    script_dir: Path,
    corpus_dir: Path,
    output_dir: Path
) -> bool:
    """Test extract_neologisms.py script."""
    print("\n[TEST] extract_neologisms.py")
    print("-" * 60)

    output_file = output_dir / "test_neologisms.json"
    script = script_dir / "extract_neologisms.py"

    cmd = [
        str(script),
        "-f", "modu",
        "-i", str(corpus_dir),
        "-o", str(output_file),
        "--min-freq", "1",
        "--max-freq", "10"
    ]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=30
        )

        if result.returncode != 0:
            print(f"✗ Failed with return code {result.returncode}")
            print(f"STDERR: {result.stderr}")
            return False

        if not output_file.exists():
            print("✗ Output file not created")
            return False

        # Check JSON structure
        with open(output_file, encoding="utf-8") as f:
            data = json.load(f)

        if "neologisms" not in data or "metadata" not in data:
            print("✗ Invalid JSON structure")
            return False

        neo_count = len(data["neologisms"])
        print(f"✓ Found {neo_count} neologism candidates")
        print(f"✓ Output file: {output_file}")
        return True

    except subprocess.TimeoutExpired:
        print("✗ Script timed out")
        return False
    except Exception as e:
        print(f"✗ Exception: {e}")
        return False


def test_merge_dictionaries(script_dir: Path, output_dir: Path) -> bool:
    """Test merge_dictionaries.py script."""
    print("\n[TEST] merge_dictionaries.py")
    print("-" * 60)

    # Create two sample dictionaries
    dict1 = output_dir / "dict1.csv"
    dict2 = output_dir / "dict2.csv"
    merged = output_dir / "merged.csv"

    # Sample data
    with open(dict1, "w", encoding="utf-8") as f:
        f.write("단어,0,0,1000,NNG,*,*,*,*,*,단어,*,*\n")
        f.write("테스트,0,0,2000,NNG,*,*,*,*,*,테스트,*,*\n")

    with open(dict2, "w", encoding="utf-8") as f:
        f.write("테스트,0,0,1500,NNG,*,*,*,*,*,테스트,*,*\n")  # Duplicate
        f.write("병합,0,0,3000,NNG,*,*,*,*,*,병합,*,*\n")

    script = script_dir / "merge_dictionaries.py"

    cmd = [
        str(script),
        "-i", str(dict1), str(dict2),
        "-o", str(merged),
        "--strategy", "min_cost"
    ]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=30
        )

        if result.returncode != 0:
            print(f"✗ Failed with return code {result.returncode}")
            print(f"STDERR: {result.stderr}")
            return False

        if not merged.exists():
            print("✗ Merged file not created")
            return False

        # Check result
        with open(merged, encoding="utf-8") as f:
            lines = f.readlines()

        if len(lines) != 3:  # Should have 3 unique entries
            print(f"✗ Expected 3 entries, got {len(lines)}")
            return False

        print(f"✓ Successfully merged {len(lines)} entries")
        print(f"✓ Output file: {merged}")
        return True

    except subprocess.TimeoutExpired:
        print("✗ Script timed out")
        return False
    except Exception as e:
        print(f"✗ Exception: {e}")
        return False


def test_analyze(script_dir: Path, output_dir: Path) -> bool:
    """Test merge_dictionaries.py --analyze."""
    print("\n[TEST] Dictionary Analysis")
    print("-" * 60)

    # Use the merged dictionary from previous test
    dict_file = output_dir / "merged.csv"

    if not dict_file.exists():
        print("✗ No dictionary file to analyze")
        return False

    script = script_dir / "merge_dictionaries.py"

    cmd = [
        str(script),
        "--analyze", str(dict_file)
    ]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=30
        )

        if result.returncode != 0:
            print(f"✗ Failed with return code {result.returncode}")
            print(f"STDERR: {result.stderr}")
            return False

        if "Dictionary Analysis" not in result.stderr:
            print("✗ Analysis output not found")
            return False

        print("✓ Analysis completed successfully")
        return True

    except subprocess.TimeoutExpired:
        print("✗ Script timed out")
        return False
    except Exception as e:
        print(f"✗ Exception: {e}")
        return False


def main() -> int:
    """Run all tests."""
    print("=" * 60)
    print("MeCab-Ko Corpus Processing Scripts - Test Suite")
    print("=" * 60)

    script_dir = Path(__file__).parent
    print(f"Script directory: {script_dir}")

    # Create temporary directories
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        corpus_dir = tmp_path / "corpus"
        output_dir = tmp_path / "output"

        corpus_dir.mkdir()
        output_dir.mkdir()

        print(f"Temporary directory: {tmp_path}")

        # Create sample corpus
        create_sample_corpus(corpus_dir)

        # Run tests
        tests = [
            ("corpus_to_dict", lambda: test_corpus_to_dict(
                script_dir, corpus_dir, output_dir)),
            ("extract_neologisms", lambda: test_extract_neologisms(
                script_dir, corpus_dir, output_dir)),
            ("merge_dictionaries", lambda: test_merge_dictionaries(
                script_dir, output_dir)),
            ("analyze", lambda: test_analyze(script_dir, output_dir)),
        ]

        results = {}
        for test_name, test_func in tests:
            try:
                results[test_name] = test_func()
            except Exception as e:
                print(f"\n✗ Test {test_name} crashed: {e}")
                results[test_name] = False

        # Print summary
        print("\n" + "=" * 60)
        print("Test Summary")
        print("=" * 60)

        total = len(results)
        passed = sum(1 for r in results.values() if r)
        failed = total - passed

        for test_name, result in results.items():
            status = "✓ PASS" if result else "✗ FAIL"
            print(f"{status:8s} - {test_name}")

        print("-" * 60)
        print(f"Total: {total} | Passed: {passed} | Failed: {failed}")
        print("=" * 60)

        if failed > 0:
            print("\n⚠ Some tests failed. Please check the output above.")
            return 1
        else:
            print("\n✓ All tests passed!")
            return 0


if __name__ == "__main__":
    sys.exit(main())
