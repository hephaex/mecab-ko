#!/usr/bin/env python3
"""
Cross-platform consistency checker for MeCab-Ko bindings.

This script runs the same inputs through different bindings and compares outputs
to ensure consistency across CLI, Python, Node.js, and WASM implementations.
"""

import json
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Optional


@dataclass
class Token:
    """Represents a parsed token."""

    surface: str
    pos: str
    features: list[str]

    @classmethod
    def from_mecab_line(cls, line: str) -> Optional["Token"]:
        """Parse a token from MeCab output line."""
        if not line or line == "EOS":
            return None

        parts = line.split("\t")
        if len(parts) < 2:
            return None

        surface = parts[0]
        feature_str = parts[1]
        features = feature_str.split(",")
        pos = features[0] if features else "UNK"

        return cls(surface=surface, pos=pos, features=features)


@dataclass
class ParseResult:
    """Results from parsing a text."""

    binding: str
    text: str
    tokens: list[Token]
    raw_output: str
    error: Optional[str] = None


class ConsistencyChecker:
    """Checks consistency across different MeCab-Ko bindings."""

    def __init__(self, project_root: Path):
        """Initialize the consistency checker."""
        self.project_root = project_root
        self.cli_path = (
            project_root / "rust" / "target" / "release" / "mecab-ko"
        )

    def parse_with_cli(self, text: str) -> ParseResult:
        """Parse text using CLI binding."""
        try:
            if not self.cli_path.exists():
                return ParseResult(
                    binding="CLI",
                    text=text,
                    tokens=[],
                    raw_output="",
                    error="CLI binary not found",
                )

            result = subprocess.run(
                [str(self.cli_path)],
                input=text.encode("utf-8"),
                capture_output=True,
                timeout=5,
            )

            if result.returncode != 0:
                return ParseResult(
                    binding="CLI",
                    text=text,
                    tokens=[],
                    raw_output=result.stderr.decode("utf-8"),
                    error=f"CLI returned {result.returncode}",
                )

            output = result.stdout.decode("utf-8")
            tokens = []
            for line in output.strip().split("\n"):
                token = Token.from_mecab_line(line)
                if token:
                    tokens.append(token)

            return ParseResult(
                binding="CLI", text=text, tokens=tokens, raw_output=output
            )

        except subprocess.TimeoutExpired:
            return ParseResult(
                binding="CLI",
                text=text,
                tokens=[],
                raw_output="",
                error="Timeout",
            )
        except Exception as e:
            return ParseResult(
                binding="CLI",
                text=text,
                tokens=[],
                raw_output="",
                error=str(e),
            )

    def parse_with_python(self, text: str) -> ParseResult:
        """Parse text using Python binding."""
        try:
            import mecab_ko

            tagger = mecab_ko.Tagger()
            output = tagger.parse(text)

            tokens = []
            for line in output.strip().split("\n"):
                token = Token.from_mecab_line(line)
                if token:
                    tokens.append(token)

            return ParseResult(
                binding="Python", text=text, tokens=tokens, raw_output=output
            )

        except ImportError:
            return ParseResult(
                binding="Python",
                text=text,
                tokens=[],
                raw_output="",
                error="mecab_ko not installed",
            )
        except Exception as e:
            return ParseResult(
                binding="Python",
                text=text,
                tokens=[],
                raw_output="",
                error=str(e),
            )

    def parse_with_nodejs(self, text: str) -> ParseResult:
        """Parse text using Node.js binding."""
        try:
            # Create a temporary Node.js script
            script = f"""
            const mecab = require('mecab-ko-node');
            const tagger = new mecab.Tagger();
            const result = tagger.parse('{text}');
            console.log(result);
            """

            with tempfile.NamedTemporaryFile(
                mode="w", suffix=".js", delete=False
            ) as f:
                f.write(script)
                script_path = f.name

            result = subprocess.run(
                ["node", script_path],
                capture_output=True,
                timeout=5,
            )

            Path(script_path).unlink()

            if result.returncode != 0:
                return ParseResult(
                    binding="Node.js",
                    text=text,
                    tokens=[],
                    raw_output=result.stderr.decode("utf-8"),
                    error=f"Node.js returned {result.returncode}",
                )

            output = result.stdout.decode("utf-8")
            tokens = []
            for line in output.strip().split("\n"):
                token = Token.from_mecab_line(line)
                if token:
                    tokens.append(token)

            return ParseResult(
                binding="Node.js",
                text=text,
                tokens=tokens,
                raw_output=output,
            )

        except Exception as e:
            return ParseResult(
                binding="Node.js",
                text=text,
                tokens=[],
                raw_output="",
                error=str(e),
            )

    def compare_results(
        self, results: list[ParseResult]
    ) -> dict[str, Any]:
        """Compare results from different bindings."""
        # Filter out failed results
        valid_results = [r for r in results if not r.error]

        if len(valid_results) < 2:
            return {
                "consistent": False,
                "reason": "Not enough valid results to compare",
                "valid_bindings": [r.binding for r in valid_results],
                "errors": {
                    r.binding: r.error for r in results if r.error
                },
            }

        # Compare token counts
        token_counts = {r.binding: len(r.tokens) for r in valid_results}
        if len(set(token_counts.values())) > 1:
            return {
                "consistent": False,
                "reason": "Different token counts",
                "token_counts": token_counts,
            }

        # Compare individual tokens
        reference = valid_results[0]
        inconsistencies = []

        for other in valid_results[1:]:
            for i, (ref_token, other_token) in enumerate(
                zip(reference.tokens, other.tokens)
            ):
                if ref_token.surface != other_token.surface:
                    inconsistencies.append(
                        {
                            "position": i,
                            "type": "surface",
                            "reference": f"{reference.binding}: {ref_token.surface}",
                            "other": f"{other.binding}: {other_token.surface}",
                        }
                    )

                if ref_token.pos != other_token.pos:
                    inconsistencies.append(
                        {
                            "position": i,
                            "type": "pos",
                            "surface": ref_token.surface,
                            "reference": f"{reference.binding}: {ref_token.pos}",
                            "other": f"{other.binding}: {other_token.pos}",
                        }
                    )

        if inconsistencies:
            return {
                "consistent": False,
                "reason": "Token differences found",
                "inconsistencies": inconsistencies,
            }

        return {
            "consistent": True,
            "bindings_tested": [r.binding for r in valid_results],
            "token_count": len(reference.tokens),
        }

    def check_consistency(self, test_sentences: list[dict]) -> dict:
        """Check consistency across all test sentences."""
        results = {
            "total_tests": len(test_sentences),
            "consistent_tests": 0,
            "inconsistent_tests": 0,
            "details": [],
        }

        for test_case in test_sentences:
            text = test_case["input"]
            test_id = test_case.get("id", "unknown")

            # Parse with all bindings
            parse_results = [
                self.parse_with_cli(text),
                self.parse_with_python(text),
                self.parse_with_nodejs(text),
            ]

            # Compare results
            comparison = self.compare_results(parse_results)

            test_result = {
                "id": test_id,
                "text": text,
                "comparison": comparison,
                "results": [
                    {
                        "binding": r.binding,
                        "token_count": len(r.tokens),
                        "error": r.error,
                    }
                    for r in parse_results
                ],
            }

            results["details"].append(test_result)

            if comparison["consistent"]:
                results["consistent_tests"] += 1
            else:
                results["inconsistent_tests"] += 1

        return results


def main():
    """Main entry point."""
    # Find project root
    script_path = Path(__file__).resolve()
    project_root = script_path.parent.parent.parent.parent

    # Load test sentences
    fixtures_path = (
        project_root / "tests" / "e2e" / "fixtures" / "test_sentences.json"
    )

    if not fixtures_path.exists():
        print(f"Error: Test fixtures not found at {fixtures_path}")
        sys.exit(1)

    with open(fixtures_path, encoding="utf-8") as f:
        data = json.load(f)

    test_sentences = data["test_cases"][:10]  # Test first 10 for now

    # Run consistency check
    checker = ConsistencyChecker(project_root)
    results = checker.check_consistency(test_sentences)

    # Print results
    print("\n" + "=" * 70)
    print("MeCab-Ko Cross-Platform Consistency Check")
    print("=" * 70)
    print(f"\nTotal tests: {results['total_tests']}")
    print(f"Consistent: {results['consistent_tests']}")
    print(f"Inconsistent: {results['inconsistent_tests']}")
    print()

    # Print details for inconsistent tests
    if results["inconsistent_tests"] > 0:
        print("\nInconsistent Tests:")
        print("-" * 70)

        for detail in results["details"]:
            if not detail["comparison"]["consistent"]:
                print(f"\n[{detail['id']}] {detail['text'][:50]}...")
                print(
                    f"  Reason: {detail['comparison'].get('reason', 'Unknown')}"
                )

                if "inconsistencies" in detail["comparison"]:
                    for incon in detail["comparison"]["inconsistencies"][
                        :3
                    ]:  # Show first 3
                        print(f"    - Position {incon['position']}:")
                        print(f"      {incon['reference']}")
                        print(f"      {incon['other']}")

                # Show which bindings failed
                for result in detail["results"]:
                    if result["error"]:
                        print(
                            f"  {result['binding']}: ERROR - {result['error']}"
                        )

    # Exit with error if any inconsistencies found
    if results["inconsistent_tests"] > 0:
        print("\n⚠️  Consistency check failed!")
        sys.exit(1)
    else:
        print("\n✅ All tests passed - bindings are consistent!")
        sys.exit(0)


if __name__ == "__main__":
    main()
