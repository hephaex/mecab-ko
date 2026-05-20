# PLAN — mecab-ko Sprint 154 (next, 자동 결정)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 153 E — XSA+ETM 비이슈 확인

- 전문가 권고: XSA+ETM 미처리 추정
- 실측: 38/38 처리됨 (converter decomp fallback)
- 규칙 5 (자동 트랙 선택) 첫 실 적용

## 누적 진척 상황

### 정확도 lift sprint
| Sprint | 효과 |
|--------|------|
| 147 | VV/XSV practical 동치 +0.3pp |
| 150 A | VA+ETM multi-syllable +0.4pp KLUE strict |

### 비이슈 확인 sprint (가치: 학습)
| Sprint | 패턴 | 처리 위치 |
|--------|------|----------|
| 148 D | ETM+ETM "라는" 33건 | splitter L71-73 중복 태그 규칙 |
| 153 E | XSA+ETM 38건 | converter L162-187 decomp fallback |

### 정리 sprint
| Sprint | 효과 |
|--------|------|
| 149 | accuracy_eval -2163줄 + MSRV gate + coverage floor |
| 151 C | setup helper 추출 -563줄 |
| 152 D | Node/WASM continue-on-error 정리 |

### 인프라 작업
| Sprint | 효과 |
|--------|------|
| 152 D | agents.md 규칙 5 추가 (자동 트랙 선택) |

## Sprint 154 — 자동 결정 (전문가 리뷰 기반)

### 진단할 후보 (Sprint 153 학습 반영)

전문가 리뷰 + 빈도 분석 + **반드시 splitter/converter 변환 후 측정**:

- **EP+ETM 86건** (던, 는, 신): ending_rules에 단독 EP+ETM 부재 — 진단 필요
- **XSV+ETM 72건** (던, 헌, 시킬): 미처리 가능성
- **VX+EP 25건** (했, 못했, 왔): 보조용언 + 과거
- **XSA+EP 35건** (했, 스러웠, 허): XSA + 과거
- **새 영역**: 전문가가 식별

### 진단 우선 (Sprint 148 D, 153 E 교훈)

작업 순서:
1. 전문가 리뷰 → Top 권고
2. 진단 테스트 작성 (raw → splitter → converter 모두 확인)
3. 실제 미처리 건수 측정
4. 미처리 ≥ 10건이면 → 구현 + 5-gate
5. 미처리 < 10건이면 → 비이슈 문서화 후 다음 후보

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv / KLUE morph / surface_only / UD Kaist / UD GSD)
- sample.tsv baseline 100%/99.9% **회귀 금지**
