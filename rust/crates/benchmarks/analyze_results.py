#!/usr/bin/env python3
"""
MeCab-Ko Benchmark Results Analyzer

This script analyzes Criterion benchmark results and generates reports.

Usage:
    python3 analyze_results.py [--input DIR] [--output FILE] [--format {json,csv,md}]

Examples:
    # Analyze latest results
    python3 analyze_results.py

    # Generate Markdown report
    python3 analyze_results.py --format md --output report.md

    # Analyze specific results directory
    python3 analyze_results.py --input benchmark_results_20240115_120000
"""

import argparse
import json
import os
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional


@dataclass
class BenchmarkData:
    """Represents a single benchmark result."""
    name: str
    group: str
    mean_ns: float
    std_dev_ns: float
    median_ns: Optional[float] = None
    throughput: Optional[float] = None


class BenchmarkAnalyzer:
    """Analyzes Criterion benchmark results."""

    def __init__(self, results_dir: Path):
        self.results_dir = results_dir
        self.benchmarks: List[BenchmarkData] = []

    def load_results(self):
        """Load all benchmark results from the directory."""
        criterion_dir = self.results_dir / "criterion"
        if not criterion_dir.exists():
            print(f"Warning: {criterion_dir} not found", file=sys.stderr)
            return

        # Walk through criterion directory structure
        for group_dir in criterion_dir.iterdir():
            if not group_dir.is_dir() or group_dir.name in ["report", ".cargo-criterion"]:
                continue

            # Look for estimates.json in each benchmark
            for bench_dir in group_dir.iterdir():
                if not bench_dir.is_dir():
                    continue

                estimates_file = bench_dir / "base" / "estimates.json"
                if not estimates_file.exists():
                    continue

                try:
                    with open(estimates_file) as f:
                        data = json.load(f)

                    mean_ns = data.get("mean", {}).get("point_estimate", 0)
                    std_dev_ns = data.get("std_dev", {}).get("point_estimate", 0)
                    median_ns = data.get("median", {}).get("point_estimate")

                    bench = BenchmarkData(
                        name=bench_dir.name,
                        group=group_dir.name,
                        mean_ns=mean_ns,
                        std_dev_ns=std_dev_ns,
                        median_ns=median_ns,
                    )
                    self.benchmarks.append(bench)

                except (json.JSONDecodeError, KeyError) as e:
                    print(f"Warning: Failed to parse {estimates_file}: {e}", file=sys.stderr)

        print(f"Loaded {len(self.benchmarks)} benchmark results")

    def generate_summary(self) -> Dict[str, List[BenchmarkData]]:
        """Generate summary grouped by benchmark group."""
        summary: Dict[str, List[BenchmarkData]] = {}

        for bench in self.benchmarks:
            if bench.group not in summary:
                summary[bench.group] = []
            summary[bench.group].append(bench)

        # Sort each group by mean time
        for group in summary.values():
            group.sort(key=lambda b: b.mean_ns)

        return summary

    def to_json(self) -> str:
        """Generate JSON report."""
        summary = self.generate_summary()
        data = {}

        for group, benches in summary.items():
            data[group] = [
                {
                    "name": b.name,
                    "mean_ns": b.mean_ns,
                    "mean_ms": b.mean_ns / 1_000_000,
                    "std_dev_ns": b.std_dev_ns,
                    "median_ns": b.median_ns,
                }
                for b in benches
            ]

        return json.dumps(data, indent=2)

    def to_csv(self) -> str:
        """Generate CSV report."""
        lines = ["Group,Benchmark,Mean (ms),Std Dev (ms),Median (ms)"]

        for bench in sorted(self.benchmarks, key=lambda b: (b.group, b.mean_ns)):
            mean_ms = bench.mean_ns / 1_000_000
            std_dev_ms = bench.std_dev_ns / 1_000_000
            median_ms = bench.median_ns / 1_000_000 if bench.median_ns else ""

            lines.append(f"{bench.group},{bench.name},{mean_ms:.3f},{std_dev_ms:.3f},{median_ms}")

        return "\n".join(lines)

    def to_markdown(self) -> str:
        """Generate Markdown report."""
        summary = self.generate_summary()
        lines = ["# MeCab-Ko Benchmark Results", ""]

        # Overall statistics
        total_benches = len(self.benchmarks)
        total_groups = len(summary)
        lines.extend([
            f"**Total Benchmarks:** {total_benches}",
            f"**Total Groups:** {total_groups}",
            "",
        ])

        # Per-group tables
        for group, benches in sorted(summary.items()):
            lines.extend([
                f"## {group}",
                "",
                "| Benchmark | Mean | Std Dev | Median |",
                "|-----------|------|---------|--------|",
            ])

            for bench in benches:
                mean_ms = bench.mean_ns / 1_000_000
                std_dev_ms = bench.std_dev_ns / 1_000_000
                median_str = f"{bench.median_ns / 1_000_000:.3f} ms" if bench.median_ns else "N/A"

                lines.append(
                    f"| {bench.name} | {mean_ms:.3f} ms | {std_dev_ms:.3f} ms | {median_str} |"
                )

            lines.extend(["", ""])

        # Top 10 fastest
        lines.extend([
            "## Top 10 Fastest Benchmarks",
            "",
            "| Rank | Group | Benchmark | Mean |",
            "|------|-------|-----------|------|",
        ])

        fastest = sorted(self.benchmarks, key=lambda b: b.mean_ns)[:10]
        for i, bench in enumerate(fastest, 1):
            mean_ms = bench.mean_ns / 1_000_000
            lines.append(f"| {i} | {bench.group} | {bench.name} | {mean_ms:.3f} ms |")

        lines.extend(["", ""])

        # Top 10 slowest
        lines.extend([
            "## Top 10 Slowest Benchmarks",
            "",
            "| Rank | Group | Benchmark | Mean |",
            "|------|-------|-----------|------|",
        ])

        slowest = sorted(self.benchmarks, key=lambda b: b.mean_ns, reverse=True)[:10]
        for i, bench in enumerate(slowest, 1):
            mean_ms = bench.mean_ns / 1_000_000
            lines.append(f"| {i} | {bench.group} | {bench.name} | {mean_ms:.3f} ms |")

        return "\n".join(lines)


def find_latest_results_dir(base_dir: Path) -> Optional[Path]:
    """Find the latest benchmark results directory."""
    results_dirs = [d for d in base_dir.iterdir() if d.is_dir() and d.name.startswith("benchmark_results_")]

    if not results_dirs:
        return None

    # Sort by directory name (which includes timestamp)
    return sorted(results_dirs)[-1]


def main():
    parser = argparse.ArgumentParser(description="Analyze MeCab-Ko benchmark results")
    parser.add_argument(
        "--input",
        type=Path,
        help="Input directory containing benchmark results (default: latest)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Output file (default: stdout)",
    )
    parser.add_argument(
        "--format",
        choices=["json", "csv", "md"],
        default="md",
        help="Output format (default: md)",
    )

    args = parser.parse_args()

    # Determine input directory
    if args.input:
        results_dir = args.input
    else:
        # Find latest results directory
        base_dir = Path.cwd()
        results_dir = find_latest_results_dir(base_dir)

        if not results_dir:
            print("Error: No benchmark results found", file=sys.stderr)
            print("Run benchmarks first: ./run_benchmarks.sh", file=sys.stderr)
            sys.exit(1)

    if not results_dir.exists():
        print(f"Error: Directory not found: {results_dir}", file=sys.stderr)
        sys.exit(1)

    print(f"Analyzing results from: {results_dir}", file=sys.stderr)

    # Analyze benchmarks
    analyzer = BenchmarkAnalyzer(results_dir)
    analyzer.load_results()

    if not analyzer.benchmarks:
        print("Warning: No benchmark data found", file=sys.stderr)
        sys.exit(1)

    # Generate report
    if args.format == "json":
        output = analyzer.to_json()
    elif args.format == "csv":
        output = analyzer.to_csv()
    else:  # markdown
        output = analyzer.to_markdown()

    # Write output
    if args.output:
        with open(args.output, "w") as f:
            f.write(output)
        print(f"Report saved to: {args.output}", file=sys.stderr)
    else:
        print(output)


if __name__ == "__main__":
    main()
