# PROGRESS — mecab-ko Sprint 139 (UD Korean-Kaist 통합)

> 마지막 업데이트: 2026-05-19

## Sprint 139 P2 — UD Korean-Kaist Silver Baseline 통합

| Task | 상태 | 비고 |
|------|------|------|
| S139-P2-1: UD Korean-Kaist 다운로드 + 형식 확인 | ✅ 완료 | `data/raw/ud_kaist/` test(3.3MB) + dev(3.0MB) |
| S139-P2-2: KAIST XPOS → Sejong tag converter | ✅ 완료 | `tools/convert_ud_kaist.py` (50 tag mapping) |
| S139-P2-3: TSV 변환 + 평가 통합 | ✅ 완료 | 3,124 sentences 변환 (71.8% 변환률), `test_ud_kaist_dual_metric` 추가 |
| S139-P2-4: 기준 측정 + 보고서 | ✅ 완료 | `docs/research/accuracy/2026-05-19_sprint139_ud_kaist.md` |

## 핵심 발견

### lemma + XPOS 결합이 UPOS보다 우월

UD CoNLL-U는 column 4(UPOS, 보편 태그)와 column 5(XPOS, 언어별 KAIST 태그) 둘 다 제공.
- UPOS만 사용 시 NOUN→NNG/NNP/NNB lossy
- **XPOS 사용 시 ncn→NNG, jca→JKB 등 morpheme 단위 직접 매핑 가능**
- lemma column ("조약+에")에서 morpheme 분해 정보 추출

### Silver 변환 결과

- 입력: 4,353 sentences (test+dev)
- 변환: 3,124 sentences (71.8%)
- Skip: 1,229 sentences (unknown tag, lemma/xpos mismatch)
- 보수적 skip이 정확도 측면에서 안전

### Baseline 측정 (test split, 1,638 sentences)

| Metric | UD Kaist | KLUE DP | 차이 |
|--------|---------|---------|-----|
| Morph strict | 66.3% | 66.8% | -0.5pp (거의 동일) |
| Morph practical | 68.0% | 71.6% | -3.6pp (lift 폭 차이) |
| Per-eojeol strict | 20.7% | 20.7% | 0 |
| Per-eojeol practical | 21.8% | 23.5% | -1.7pp |

**해석**:
- morph strict 거의 동일 → mecab 기본 동작 일관
- practical lift 차이 → UD KAIST jcc(보격)→JKC가 mecab JKS(주격)와 차이. KLUE는 SP/SC/NNB/NNG 등 흡수가 큼.
- **도메인 다양화 효과 확인**: 두 데이터셋이 보완적 오류 발견

## 측정값 (변경 없음 — 평가 데이터 추가만)

| 메트릭 | Sprint 138 | Sprint 139 | Δ |
|--------|-----------|-----------|---|
| KLUE morph strict | 66.8% | 66.8% | — |
| KLUE morph practical | 71.6% | 71.6% | — |
| KLUE eo practical | 23.5% | 23.5% | — |
| Surface canonical_lenient | 95.5% | 95.5% | — |
| Sample.tsv Token | 100.0% | 100.0% | — |
| Sample.tsv Sentence | 99.9% | 99.9% | — |
| **UD Kaist morph strict** | (신규) | **66.3%** | (신규) |
| **UD Kaist morph practical** | (신규) | **68.0%** | (신규) |

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: all pass / 0 fail
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `test_ud_kaist_dual_metric`: PASS (strict 66.3% / practical 68.0%)

## 변경 파일

- `data/raw/ud_kaist/ko_kaist-ud-test.conllu` (downloaded, 3.3MB)
- `data/raw/ud_kaist/ko_kaist-ud-dev.conllu` (downloaded, 3.0MB)
- `data/eval/ud_kaist_test.tsv` (신규 1,638 lines)
- `data/eval/ud_kaist_dev.tsv` (신규 1,486 lines)
- `tools/convert_ud_kaist.py` (신규 CoNLL-U → TSV 변환기)
- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`: `test_ud_kaist_dual_metric` 추가 (~75줄)
- `docs/research/accuracy/2026-05-19_sprint139_ud_kaist.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 140 후보

1. UD Kaist SPLIT_DIFFERENT 분석 (Sprint 137과 동일 방법, 다른 도메인)
2. JKC ↔ JKS practical 동치 추가 검토 (UD-mecab convention 차이)
3. accuracy-gate CI에 UD Kaist 추가 (3 게이트)
4. Track C: dict-builder CSV 버그 수정 (선행)
5. Track B: full CRF retrain (escalation)
