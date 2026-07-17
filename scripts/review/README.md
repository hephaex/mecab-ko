# Competitor review automation

Automates the "garu vs mecab-ko" comparative review so it can be regenerated with
fresh, self-collected metrics instead of being hand-written once and going stale.

## Files

- `competitor_review.py` — renders the comparison doc from live metrics + committed F1 JSON.
- `measure_f1.py` — runs the real `mecab evaluate` over committed gold-set TSVs and writes `mecab_f1.json` (honest external F1).
- Output docs → `docs/research/competitor-analysis/garu-vs-mecab-ko.md` + `mecab_f1.json`

## What is automated vs authored

| Part | Source |
|---|---|
| garu repo stats (stars/forks/issues/last push) | GitHub API, live each run |
| garu model/WASM size, F1 numbers | parsed from garu README |
| mecab-ko dict size, WASM size, crate count, speed/memory/test counts | measured from this working tree + root `README.md` |
| mecab-ko external gold-set F1 (UD-GSD/Kaist/KLUE, Sejong-normalized) | measured by `measure_f1.py` → `mecab_f1.json` |
| Architecture trade-offs, recommendations, verdict | authored judgement in `ANALYSIS_*` constants |

The generator refreshes the **data and timestamp**; it never fabricates the
qualitative analysis. To change the analysis, edit the `ANALYSIS_*` constants.

## Usage

```bash
# regenerate the document
python3 scripts/review/competitor_review.py

# print to stdout without writing
python3 scripts/review/competitor_review.py --stdout

# CI guard: exit 2 if the committed doc is out of date (timestamp ignored)
python3 scripts/review/competitor_review.py --check
```

Exit codes: `0` ok · `1` some metric failed to collect (left as `N/A`) · `2` `--check` mismatch.

## Measuring external gold-set F1

`measure_f1.py` needs the compiled dictionary (`data/dict-output/`, gitignored) and
the release CLI (`rust/target/release/mecab`). Both exist after a local dict build.

```bash
# build the CLI if needed, then measure F1 over the committed gold sets
python3 scripts/review/measure_f1.py --build

# measure using an already-built CLI + dict
python3 scripts/review/measure_f1.py

# print JSON without writing
python3 scripts/review/measure_f1.py --stdout
```

It writes `mecab_f1.json` (numbers only — safe to commit; the underlying corpora are
not). `competitor_review.py` then reads that JSON. Refresh flow:

```bash
python3 scripts/review/measure_f1.py       # refresh F1 numbers (needs dict)
python3 scripts/review/competitor_review.py # re-render the doc
git add docs/research/competitor-analysis
```

Honesty note: UD-GSD / UD-Kaist / KLUE-DP are *silver* datasets (native tags
lossily converted to Sejong), so even a perfect analyzer cannot reach F1 1.0, and
they are not the datasets garu reports on. Treat the numbers as directional, not a
like-for-like head-to-head. See the caveat block in `mecab_f1.json`.

## Scheduled regeneration

`.github/workflows/scheduled.yml` runs the generator weekly (Sun 02:00 UTC) and on
manual `workflow_dispatch`, committing the refreshed doc only when it changed.
