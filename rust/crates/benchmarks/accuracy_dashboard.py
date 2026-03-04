#!/usr/bin/env python3
"""
MeCab-Ko Accuracy Dashboard

This script visualizes accuracy trends and generates reports.

Usage:
    python3 accuracy_dashboard.py [--format {text,html,json}] [--output FILE]

Examples:
    # Display text dashboard
    python3 accuracy_dashboard.py

    # Generate HTML report
    python3 accuracy_dashboard.py --format html --output accuracy_report.html
"""

import argparse
import json
import os
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import List, Optional


@dataclass
class AccuracyRecord:
    """Represents a single accuracy measurement."""
    date: str
    version: str
    sprint: str
    token_accuracy: float
    sentence_accuracy: float
    notes: str


class AccuracyDashboard:
    """Displays accuracy trends and metrics."""

    def __init__(self, history_file: Path):
        self.history_file = history_file
        self.records: List[AccuracyRecord] = []

    def load_history(self) -> bool:
        """Load accuracy history from JSON file."""
        if not self.history_file.exists():
            print(f"Warning: {self.history_file} not found")
            return False

        with open(self.history_file) as f:
            data = json.load(f)

        for entry in data.get("history", []):
            self.records.append(AccuracyRecord(
                date=entry.get("date", ""),
                version=entry.get("version", ""),
                sprint=entry.get("sprint", ""),
                token_accuracy=entry.get("token_accuracy", 0.0),
                sentence_accuracy=entry.get("sentence_accuracy", 0.0),
                notes=entry.get("notes", "")
            ))

        return True

    def generate_text_dashboard(self) -> str:
        """Generate ASCII text dashboard."""
        if not self.records:
            return "No accuracy data available."

        lines = []
        width = 70

        # Header
        lines.append("=" * width)
        lines.append(" MeCab-Ko Accuracy Dashboard ".center(width))
        lines.append("=" * width)
        lines.append("")

        # Current status
        latest = self.records[-1]
        lines.append(f"Current Accuracy: {latest.token_accuracy:.1f}%")
        lines.append(f"Version: {latest.version} | Sprint: {latest.sprint}")
        lines.append("")

        # Trend chart (ASCII)
        lines.append("-" * width)
        lines.append(" Accuracy Trend ".center(width))
        lines.append("-" * width)

        max_acc = max(r.token_accuracy for r in self.records)
        chart_height = 10
        chart_width = min(len(self.records), 20)

        # Generate ASCII chart
        for row in range(chart_height, -1, -1):
            threshold = (row / chart_height) * max_acc
            line = f"{threshold:5.1f}% |"
            for i, record in enumerate(self.records[-chart_width:]):
                if record.token_accuracy >= threshold:
                    line += " ##"
                else:
                    line += "   "
            lines.append(line)

        # X-axis
        lines.append("       " + "-" * (chart_width * 3 + 1))

        # History table
        lines.append("")
        lines.append("-" * width)
        lines.append(" History ".center(width))
        lines.append("-" * width)
        lines.append(f"{'Date':<12} {'Version':<8} {'Token%':>8} {'Sent%':>8}  Notes")
        lines.append("-" * width)

        for record in self.records:
            lines.append(
                f"{record.date:<12} {record.version:<8} "
                f"{record.token_accuracy:>7.1f}% {record.sentence_accuracy:>7.1f}%  "
                f"{record.notes[:25]}"
            )

        # Statistics
        lines.append("")
        lines.append("-" * width)
        lines.append(" Statistics ".center(width))
        lines.append("-" * width)

        first = self.records[0]
        total_improvement = latest.token_accuracy - first.token_accuracy
        avg_improvement = total_improvement / max(len(self.records) - 1, 1)

        lines.append(f"Total Improvement: +{total_improvement:.1f}%")
        lines.append(f"Average per Sprint: +{avg_improvement:.1f}%")
        lines.append(f"Measurements: {len(self.records)}")
        lines.append("")
        lines.append("=" * width)

        return "\n".join(lines)

    def generate_html_dashboard(self) -> str:
        """Generate HTML dashboard with Chart.js visualization."""
        if not self.records:
            return "<html><body><p>No accuracy data available.</p></body></html>"

        dates = [r.date for r in self.records]
        token_acc = [r.token_accuracy for r in self.records]
        sent_acc = [r.sentence_accuracy for r in self.records]
        latest = self.records[-1]

        html = f"""<!DOCTYPE html>
<html lang="ko">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MeCab-Ko Accuracy Dashboard</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        .card {{
            background: white;
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .metrics {{
            display: flex;
            justify-content: space-around;
            text-align: center;
        }}
        .metric {{
            padding: 20px;
        }}
        .metric-value {{
            font-size: 48px;
            font-weight: bold;
            color: #2563eb;
        }}
        .metric-label {{
            color: #666;
            margin-top: 5px;
        }}
        .chart-container {{
            position: relative;
            height: 400px;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
        }}
        th, td {{
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }}
        th {{
            background-color: #f8f9fa;
        }}
        .improvement {{
            color: #16a34a;
            font-weight: bold;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>MeCab-Ko Accuracy Dashboard</h1>
        <p>Version {latest.version} | {latest.sprint}</p>
    </div>

    <div class="card">
        <div class="metrics">
            <div class="metric">
                <div class="metric-value">{latest.token_accuracy:.1f}%</div>
                <div class="metric-label">Token Accuracy</div>
            </div>
            <div class="metric">
                <div class="metric-value">{latest.sentence_accuracy:.1f}%</div>
                <div class="metric-label">Sentence Accuracy</div>
            </div>
            <div class="metric">
                <div class="metric-value improvement">+{latest.token_accuracy - self.records[0].token_accuracy:.1f}%</div>
                <div class="metric-label">Total Improvement</div>
            </div>
        </div>
    </div>

    <div class="card">
        <h2>Accuracy Trend</h2>
        <div class="chart-container">
            <canvas id="accuracyChart"></canvas>
        </div>
    </div>

    <div class="card">
        <h2>History</h2>
        <table>
            <thead>
                <tr>
                    <th>Date</th>
                    <th>Version</th>
                    <th>Sprint</th>
                    <th>Token Accuracy</th>
                    <th>Sentence Accuracy</th>
                    <th>Notes</th>
                </tr>
            </thead>
            <tbody>
"""

        for record in reversed(self.records):
            html += f"""                <tr>
                    <td>{record.date}</td>
                    <td>{record.version}</td>
                    <td>{record.sprint}</td>
                    <td>{record.token_accuracy:.1f}%</td>
                    <td>{record.sentence_accuracy:.1f}%</td>
                    <td>{record.notes}</td>
                </tr>
"""

        html += f"""            </tbody>
        </table>
    </div>

    <script>
        const ctx = document.getElementById('accuracyChart').getContext('2d');
        new Chart(ctx, {{
            type: 'line',
            data: {{
                labels: {json.dumps(dates)},
                datasets: [{{
                    label: 'Token Accuracy (%)',
                    data: {json.dumps(token_acc)},
                    borderColor: '#2563eb',
                    backgroundColor: 'rgba(37, 99, 235, 0.1)',
                    fill: true,
                    tension: 0.4
                }}, {{
                    label: 'Sentence Accuracy (%)',
                    data: {json.dumps(sent_acc)},
                    borderColor: '#16a34a',
                    backgroundColor: 'rgba(22, 163, 74, 0.1)',
                    fill: true,
                    tension: 0.4
                }}]
            }},
            options: {{
                responsive: true,
                maintainAspectRatio: false,
                scales: {{
                    y: {{
                        beginAtZero: true,
                        max: 100,
                        title: {{
                            display: true,
                            text: 'Accuracy (%)'
                        }}
                    }}
                }},
                plugins: {{
                    legend: {{
                        position: 'top'
                    }}
                }}
            }}
        }});
    </script>
</body>
</html>
"""
        return html

    def generate_json_report(self) -> str:
        """Generate JSON report."""
        if not self.records:
            return json.dumps({"error": "No data"}, indent=2)

        latest = self.records[-1]
        first = self.records[0]

        report = {
            "generated_at": datetime.now().isoformat(),
            "current": {
                "token_accuracy": latest.token_accuracy,
                "sentence_accuracy": latest.sentence_accuracy,
                "version": latest.version,
                "sprint": latest.sprint
            },
            "improvement": {
                "total": latest.token_accuracy - first.token_accuracy,
                "per_sprint": (latest.token_accuracy - first.token_accuracy) / max(len(self.records) - 1, 1)
            },
            "history": [
                {
                    "date": r.date,
                    "version": r.version,
                    "sprint": r.sprint,
                    "token_accuracy": r.token_accuracy,
                    "sentence_accuracy": r.sentence_accuracy,
                    "notes": r.notes
                }
                for r in self.records
            ]
        }

        return json.dumps(report, indent=2, ensure_ascii=False)


def main():
    parser = argparse.ArgumentParser(description="MeCab-Ko Accuracy Dashboard")
    parser.add_argument(
        "--format", "-f",
        choices=["text", "html", "json"],
        default="text",
        help="Output format (default: text)"
    )
    parser.add_argument(
        "--output", "-o",
        type=str,
        help="Output file (default: stdout)"
    )
    parser.add_argument(
        "--input", "-i",
        type=str,
        help="History JSON file"
    )

    args = parser.parse_args()

    # Find history file
    script_dir = Path(__file__).parent
    if args.input:
        history_file = Path(args.input)
    else:
        history_file = script_dir / "accuracy_history.json"

    # Load and generate
    dashboard = AccuracyDashboard(history_file)
    if not dashboard.load_history():
        return 1

    if args.format == "text":
        output = dashboard.generate_text_dashboard()
    elif args.format == "html":
        output = dashboard.generate_html_dashboard()
    else:
        output = dashboard.generate_json_report()

    # Output
    if args.output:
        with open(args.output, "w") as f:
            f.write(output)
        print(f"Report saved to {args.output}")
    else:
        print(output)

    return 0


if __name__ == "__main__":
    exit(main())
