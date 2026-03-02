#!/usr/bin/env python3
"""
Benchmark result parser and analyzer.

Parses bencher format benchmark output and provides analysis tools.
"""

import json
import re
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Tuple, Optional


class BenchmarkParser:
    """Parse and analyze benchmark results."""

    # Regex pattern for bencher format: "test name ... bench: X ns/iter (+/- Y)"
    BENCH_PATTERN = re.compile(
        r'test\s+(\S+)\s+.*bench:\s+([\d,]+)\s+ns/iter'
    )

    @staticmethod
    def parse_bench_output(content: str) -> Dict[str, int]:
        """
        Parse bencher format benchmark output.

        Args:
            content: Benchmark output text

        Returns:
            Dictionary mapping benchmark name to time in nanoseconds
        """
        results = {}
        for line in content.split('\n'):
            match = BenchmarkParser.BENCH_PATTERN.search(line)
            if match:
                name = match.group(1)
                time_ns = int(match.group(2).replace(',', ''))
                results[name] = time_ns
        return results

    @staticmethod
    def format_time(ns: float) -> str:
        """Format nanoseconds to human-readable format."""
        if ns < 1000:
            return f"{ns:.1f}ns"
        elif ns < 1_000_000:
            return f"{ns / 1000:.1f}µs"
        else:
            return f"{ns / 1_000_000:.1f}ms"

    @staticmethod
    def compare_results(
        base: Dict[str, int], pr: Dict[str, int]
    ) -> Dict[str, Dict]:
        """
        Compare two sets of benchmark results.

        Args:
            base: Baseline benchmark results
            pr: PR benchmark results

        Returns:
            Comparison with diffs and percentages
        """
        comparison = {}
        for name, pr_time in pr.items():
            if name in base:
                base_time = base[name]
                diff_ns = pr_time - base_time
                diff_pct = (diff_ns / base_time * 100) if base_time > 0 else 0

                # Determine status
                if diff_pct > 10:
                    status = "error"
                    symbol = "❌"
                elif diff_pct > 5:
                    status = "warning"
                    symbol = "⚠️"
                elif diff_pct < 0:
                    status = "pass"
                    symbol = "🟢"
                else:
                    status = "pass"
                    symbol = "✅"

                comparison[name] = {
                    "base_ns": base_time,
                    "pr_ns": pr_time,
                    "diff_ns": diff_ns,
                    "diff_pct": round(diff_pct, 1),
                    "status": status,
                    "symbol": symbol,
                    "base_fmt": BenchmarkParser.format_time(base_time),
                    "pr_fmt": BenchmarkParser.format_time(pr_time),
                }

        return comparison

    @staticmethod
    def generate_markdown_table(
        comparison: Dict[str, Dict], title: str = "Benchmark Comparison"
    ) -> str:
        """Generate markdown table from comparison results."""
        lines = [
            f"## {title}\n",
            "| Benchmark | Base | PR | Change | Status |",
            "|-----------|------|----|---------|---------|\n",
        ]

        has_error = False
        has_warning = False
        error_details = []
        warning_details = []

        for name, data in sorted(comparison.items()):
            short_name = name.split("::")[-1]
            diff_sign = "+" if data["diff_pct"] > 0 else ""
            symbol = data["symbol"]

            lines.append(
                f"| {short_name} | {data['base_fmt']} | {data['pr_fmt']} | "
                f"{diff_sign}{data['diff_pct']}% | {symbol} |"
            )

            if data["status"] == "error":
                has_error = True
                error_details.append(f"- **{name}**: +{data['diff_pct']}%")
            elif data["status"] == "warning":
                has_warning = True
                warning_details.append(f"- {name}: +{data['diff_pct']}%")

        # Add summary
        lines.append("\n## Summary\n")
        if has_error:
            lines.append("### ❌ Critical Performance Regression Detected!")
            lines.append("The following benchmarks exceeded 10% regression threshold:\n")
            lines.extend(error_details)
            lines.append("\n**This PR should not be merged without investigation.**\n")
        elif has_warning:
            lines.append("### ⚠️ Performance Regression Warning")
            lines.append("The following benchmarks show 5-10% regression:\n")
            lines.extend(warning_details)
            lines.append("\n**Please verify this is acceptable before merging.**\n")
        else:
            lines.append("### ✅ Performance Check Passed")
            lines.append("No significant performance regressions detected.\n")

        return "\n".join(lines)

    @staticmethod
    def generate_json_results(
        content: str, version: str = "unknown", commit: str = "unknown"
    ) -> Dict:
        """Generate JSON format results."""
        results = BenchmarkParser.parse_bench_output(content)

        return {
            "version": version,
            "commit": commit,
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "platform": "ubuntu-latest",
            "rustc": "stable",
            "results": {
                name: {
                    "time_ns": time_ns,
                    "time_us": time_ns / 1000,
                    "time_ms": time_ns / 1_000_000,
                }
                for name, time_ns in results.items()
            },
        }

    @staticmethod
    def generate_comparison_json(
        base_results: Dict[str, int],
        pr_results: Dict[str, int],
        pr_number: int = 0,
        base_branch: str = "main",
        pr_branch: str = "unknown",
    ) -> Dict:
        """Generate JSON comparison results."""
        comparison = BenchmarkParser.compare_results(base_results, pr_results)

        return {
            "pr_number": pr_number,
            "base_branch": base_branch,
            "pr_branch": pr_branch,
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "base_results": {
                name: {
                    "time_ns": time_ns,
                    "time_us": time_ns / 1000,
                    "time_ms": time_ns / 1_000_000,
                }
                for name, time_ns in base_results.items()
            },
            "pr_results": {
                name: {
                    "time_ns": time_ns,
                    "time_us": time_ns / 1000,
                    "time_ms": time_ns / 1_000_000,
                }
                for name, time_ns in pr_results.items()
            },
            "comparison": comparison,
        }


def main():
    """CLI interface for benchmark parser."""
    if len(sys.argv) < 2:
        print("Usage: benchmark-parser.py <command> [args]")
        print("\nCommands:")
        print("  parse <file>              - Parse benchmark output")
        print("  compare <base> <pr>       - Compare two benchmark results")
        print("  format-table <base> <pr>  - Generate markdown table")
        sys.exit(1)

    command = sys.argv[1]

    if command == "parse":
        if len(sys.argv) < 3:
            print("Usage: benchmark-parser.py parse <file>")
            sys.exit(1)

        with open(sys.argv[2]) as f:
            content = f.read()

        results = BenchmarkParser.parse_bench_output(content)
        output = BenchmarkParser.generate_json_results(content)
        print(json.dumps(output, indent=2))

    elif command == "compare":
        if len(sys.argv) < 4:
            print("Usage: benchmark-parser.py compare <base_file> <pr_file>")
            sys.exit(1)

        with open(sys.argv[2]) as f:
            base_content = f.read()
        with open(sys.argv[3]) as f:
            pr_content = f.read()

        base_results = BenchmarkParser.parse_bench_output(base_content)
        pr_results = BenchmarkParser.parse_bench_output(pr_content)

        output = BenchmarkParser.generate_comparison_json(base_results, pr_results)
        print(json.dumps(output, indent=2))

    elif command == "format-table":
        if len(sys.argv) < 4:
            print("Usage: benchmark-parser.py format-table <base_file> <pr_file>")
            sys.exit(1)

        with open(sys.argv[2]) as f:
            base_content = f.read()
        with open(sys.argv[3]) as f:
            pr_content = f.read()

        base_results = BenchmarkParser.parse_bench_output(base_content)
        pr_results = BenchmarkParser.parse_bench_output(pr_content)
        comparison = BenchmarkParser.compare_results(base_results, pr_results)

        markdown = BenchmarkParser.generate_markdown_table(comparison)
        print(markdown)

    else:
        print(f"Unknown command: {command}")
        sys.exit(1)


if __name__ == "__main__":
    main()
