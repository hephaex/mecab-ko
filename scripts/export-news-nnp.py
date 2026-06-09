#!/usr/bin/env python3
"""Baram NNP → mecab-ko 12필드 CSV 변환 + 통계 생성.

CT250 PG user_dictionary에서 verified=true, length>=2 NNP를
mecab-ko 12필드 CSV로 변환하여 data/domain-dic/news/ 에 저장.

cost는 카테고리 + evaluation_score 기반으로 산정.

Usage:
    python3 scripts/export-news-nnp.py              # 기본
    python3 scripts/export-news-nnp.py --dry-run    # 미리보기
"""

import argparse
import json
import os
import sys
from datetime import datetime, timezone

import psycopg2
import psycopg2.extras

PG_HOST = os.getenv("PG_HOST", "172.20.10.200")
PG_PORT = int(os.getenv("PG_PORT", "5432"))
PG_USER = os.getenv("PG_USER", "baram")
PG_PASS = os.getenv("PG_PASS", os.getenv("PG_PASSWORD", ""))
PG_DB = os.getenv("PG_DB", "baram")

OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "..", "data", "domain-dic", "news")
CSV_FILE = os.path.join(OUTPUT_DIR, "news-nnp.csv")
STATS_FILE = os.path.join(OUTPUT_DIR, "news-nnp-stats.json")

MIN_LEN = 2

CATEGORY_MAP = {
    "인물": "인명",
    "기관": "단체명",
    "지역": "지명",
    "브랜드": "브랜드",
    "국가": "국가명",
    "이벤트": "기타",
    "기타": "기타",
    "팀": "단체명",
    "팀명": "단체명",
    "팀명(기타)": "단체명",
    "기업": "단체명",
    "직위": "기타",
    "법안": "기타",
    "언론사": "매체명",
}

CATEGORY_BASE_COST = {
    "지역": -6000,
    "국가": -6000,
    "기관": -5000,
    "기업": -5000,
    "인물": -4000,
    "브랜드": -4000,
    "언론사": -3500,
    "이벤트": -3000,
    "팀": -3500,
    "팀명": -3500,
    "팀명(기타)": -3500,
    "기타": -3000,
    "직위": -2500,
    "법안": -2500,
}

SCORE_BONUS = {5: -1000, 4: -500, 3: 0, 2: 500}


def calc_cost(category, eval_score):
    base = CATEGORY_BASE_COST.get(category, -3000)
    bonus = SCORE_BONUS.get(eval_score, -500)
    return max(-10000, min(0, base + bonus))


def get_conn():
    conn = psycopg2.connect(
        host=PG_HOST, port=PG_PORT, user=PG_USER,
        password=PG_PASS, dbname=PG_DB,
        connect_timeout=10,
    )
    conn.set_client_encoding("UTF8")
    return conn


def fetch_nnps(conn):
    cur = conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor)
    cur.execute("""
        SELECT surface, category, evaluation_score
        FROM user_dictionary
        WHERE verified = true
          AND length(surface) >= %s
          AND surface NOT LIKE '%%,%%'
        ORDER BY surface
    """, (MIN_LEN,))
    rows = cur.fetchall()
    cur.close()
    return rows


def to_12field(surface, category, eval_score):
    bunryu = CATEGORY_MAP.get(category, "기타")
    cost = calc_cost(category, eval_score)
    return f"{surface},0,0,{cost},NNP,{bunryu},*,*,{surface},{surface},{surface},*"


def write_csv(rows, path):
    now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    lines = [
        "# Baram 뉴스 NNP 사전",
        "# 출처: Baram 뉴스 크롤링 파이프라인 (CT250 PostgreSQL user_dictionary)",
        "# 검증: LLM verified=true (Solar-Open2 / EXAONE 3.5)",
        "# 형식: 표면형,좌ID,우ID,비용,품사,분류,종류,활용형,조합읽기,원형,읽기,타입",
        "# 라이선스: CC-BY-SA 4.0",
        f"# 갱신: {now}",
        f"# 항목: {len(rows)}건 (verified=true, 길이≥{MIN_LEN})",
        "#",
    ]
    for r in rows:
        lines.append(to_12field(r["surface"], r["category"], r["evaluation_score"]))

    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")

    return len(rows)


def write_stats(rows, path):
    cat_counts = {}
    for r in rows:
        cat = r["category"] or "기타"
        cat_counts[cat] = cat_counts.get(cat, 0) + 1

    stats = {
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "total_entries": len(rows),
        "min_length": MIN_LEN,
        "categories": dict(sorted(cat_counts.items(), key=lambda x: -x[1])),
    }

    with open(path, "w", encoding="utf-8") as f:
        json.dump(stats, f, ensure_ascii=False, indent=2)
        f.write("\n")

    return stats


def main():
    parser = argparse.ArgumentParser(description="Export Baram NNP to mecab-ko CSV")
    parser.add_argument("--dry-run", action="store_true", help="Preview only")
    args = parser.parse_args()

    conn = get_conn()
    rows = fetch_nnps(conn)
    conn.close()

    print(f"Fetched {len(rows)} verified NNPs (len>={MIN_LEN})")

    if args.dry_run:
        for r in rows[:10]:
            print(f"  {to_12field(r['surface'], r['category'], r['evaluation_score'])}")
        if len(rows) > 10:
            print(f"  ... ({len(rows) - 10} more)")
        return

    os.makedirs(OUTPUT_DIR, exist_ok=True)
    count = write_csv(rows, CSV_FILE)
    stats = write_stats(rows, STATS_FILE)

    print(f"Wrote {count} entries to {CSV_FILE}")
    print(f"Stats: {json.dumps(stats['categories'], ensure_ascii=False)}")


if __name__ == "__main__":
    main()
