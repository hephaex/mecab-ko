#!/usr/bin/env python3
"""Measure mecab-ko F1 on committed external gold sets and emit a results JSON.

This exists so the competitor review reports an *honest, externally-measured* F1
instead of the self-generated "Token Accuracy 100%". It runs the real CLI
`mecab evaluate` over the gold-set TSVs checked into `data/eval/` and records
Precision / Recall / F1 / Token-Accuracy for each dataset.

Two comparison modes are recorded per dataset:
  * strict — exact (surface, POS) match. Punished hard by tag-scheme differences.
  * sejong — Sejong-normalized comparison (jamo normalize + compound split),
             the mode this repo uses for cross-scheme (silver) evaluation.

Important honesty notes captured in the JSON:
  * UD-GSD / UD-Kaist / KLUE-DP are *silver* datasets: their native tag schemes
    are lossily converted to Sejong, so even a perfect analyzer cannot hit 1.0.
  * These are NOT the same datasets garu reports on (garu: its own v15k gold +
    NIKL MP, which is license-restricted and not committed here). Numbers are
    therefore directionally informative, not a like-for-like head-to-head.

Output: docs/research/competitor-analysis/mecab_f1.json  (numbers only — safe to commit)

Usage:
    python3 scripts/review/measure_f1.py            # measure + write JSON
    python3 scripts/review/measure_f1.py --stdout    # print JSON, do not write
    python3 scripts/review/measure_f1.py --build      # cargo build the CLI first
"""
from __future__ import annotations

import argparse
import datetime as _dt
import json
import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
CLI_BIN = REPO_ROOT / "rust" / "target" / "release" / "mecab"
DICDIR = REPO_ROOT / "data" / "dict-output"
OUTPUT = REPO_ROOT / "docs" / "research" / "competitor-analysis" / "mecab_f1.json"

# label -> (tsv filename, kind). kind "gold" = external gold/silver; "self" = synthetic baseline.
DATASETS = [
    ("UD-GSD",   "ud_gsd_test.tsv",  "gold", "UD Korean-GSD test (Google news/web), Sejong로 silver 변환"),
    ("UD-Kaist", "ud_kaist_test.tsv", "gold", "UD Korean-Kaist test (학술/뉴스), Sejong로 silver 변환"),
    ("KLUE-DP",  "klue_dp_val.tsv",  "gold", "KLUE DP val (Airbnb 후기/뉴스), Sejong로 silver 변환"),
    ("sample",   "sample.tsv",       "self", "합성 baseline (자체 생성) — 상한 sanity check"),
]
MODES = ["strict", "sejong"]

_METRICS = {
    "sentences": r"테스트 문장:\s*([0-9]+)",
    "token_accuracy": r"Token Accuracy:\s*([0-9.]+)%",
    "precision": r"Precision:\s*([0-9.]+)",
    "recall": r"Recall:\s*([0-9.]+)",
    "f1": r"F1 Score:\s*([0-9.]+)",
}


def build_cli() -> None:
    subprocess.run(
        ["cargo", "build", "--release", "-p", "mecab-ko-cli",
         "--manifest-path", str(REPO_ROOT / "rust" / "Cargo.toml")],
        check=True,
    )


def run_eval(tsv: Path, mode: str) -> dict:
    cmd = [str(CLI_BIN), "evaluate", "--input", str(tsv), "--dicdir", str(DICDIR)]
    if mode == "sejong":
        cmd.append("--sejong")
    out = subprocess.run(cmd, capture_output=True, text=True)
    blob = out.stdout + "\n" + out.stderr
    res: dict = {}
    for key, pat in _METRICS.items():
        m = re.search(pat, blob)
        if not m:
            res[key] = None
            continue
        val = float(m.group(1))
        res[key] = int(val) if key == "sentences" else (val / 100.0 if key == "token_accuracy" else val)
    return res


def measure() -> dict:
    if not CLI_BIN.exists():
        raise SystemExit(f"CLI binary not found: {CLI_BIN}\n  run with --build or `cargo build --release -p mecab-ko-cli`")
    if not DICDIR.exists():
        raise SystemExit(f"compiled dict not found: {DICDIR}")

    results = {}
    for label, fname, kind, note in DATASETS:
        tsv = REPO_ROOT / "data" / "eval" / fname
        if not tsv.exists():
            results[label] = {"kind": kind, "note": note, "missing": True}
            continue
        entry = {"kind": kind, "note": note, "tsv": fname}
        for mode in MODES:
            entry[mode] = run_eval(tsv, mode)
        results[label] = entry

    # honest headline: mean sejong F1 over external gold sets only
    gold_f1 = [
        r["sejong"]["f1"]
        for r in results.values()
        if r.get("kind") == "gold" and not r.get("missing") and r.get("sejong", {}).get("f1") is not None
    ]
    summary = {
        "external_gold_sejong_f1_mean": round(sum(gold_f1) / len(gold_f1), 3) if gold_f1 else None,
        "external_gold_sejong_f1_range": [round(min(gold_f1), 3), round(max(gold_f1), 3)] if gold_f1 else None,
        "datasets_measured": len(gold_f1),
    }
    return {
        "measured_at": _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%d %H:%M UTC"),
        "dicdir": str(DICDIR.relative_to(REPO_ROOT)),
        "modes": {
            "strict": "exact (surface, POS) 일치 — 태그 스킴 차이에 취약",
            "sejong": "Sejong 정규화(자모 정규화+복합 태그 분리) 비교 — cross-scheme 평가용",
        },
        "caveat": (
            "UD-GSD/UD-Kaist/KLUE-DP는 native 태그를 Sejong으로 lossy 변환한 silver 데이터셋이라 "
            "완벽한 분석기도 1.0에 도달할 수 없다. garu가 보고한 v15k/NIKL MP와는 다른 데이터셋이므로 "
            "직접적 head-to-head가 아니라 방향성 지표로만 해석할 것."
        ),
        "summary": summary,
        "datasets": results,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--stdout", action="store_true", help="print JSON, do not write")
    ap.add_argument("--build", action="store_true", help="cargo build the CLI first")
    args = ap.parse_args()

    if args.build:
        build_cli()

    data = measure()
    text = json.dumps(data, ensure_ascii=False, indent=2)
    if args.stdout:
        print(text)
        return 0
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(text + "\n", "utf-8")
    s = data["summary"]
    print(f"wrote {OUTPUT.relative_to(REPO_ROOT)}")
    print(f"external gold sejong F1: mean={s['external_gold_sejong_f1_mean']} range={s['external_gold_sejong_f1_range']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
