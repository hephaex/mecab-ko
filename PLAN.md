# PLAN — mecab-ko Sprint 159 (사용자 confirm 필요)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 158 — 명시 어구 정규화 + 안전 영역 소진 선언

### Sprint 158 결과
- EXPLICIT_PHRASE_PATTERNS 3 패턴 추가 (인하여→인해, 확인하여→확인해, 참고하시어요→참고하세요)
- KLUE surface canonical_lenient: +10 eojeols (~+0.05pp)
- sample.tsv 무회귀

### Sprint 156~158 안전 영역 누적 효과

| Sprint | 영역 | 효과 |
|--------|------|------|
| 156 | ㄷ 불규칙 + 르 추가 | +30 eojeols |
| 157 | MAG/MAJ practical 동치 | +124 eojeols + 3 silver +0.2pp |
| 158 | 명시 어구 축약 | +10 eojeols |

**총 +164 eojeols + 3 silver morph +0.2pp.**

## 안전 영역 소진 선언

진단 + 시도 결과:
- splitter rule: 영역 소진 (Sprint 154)
- dict cost 확장: 회귀 (Sprint 155)
- CRF matrix 조정: 회귀 (Sprint 138)
- surface normalization: 효과 ≤ +0.05pp/sprint
- PRACTICAL 동치: 추가 후보 부족

**다음 단계: 비가역 대규모 작업만 남음** → 사용자 confirm 필요.

## 누적 진척 (Sprint 122 baseline → Sprint 158)

| Metric | Sprint 122 | Sprint 158 | Δ |
|--------|-----------|-----------|---|
| sample.tsv | 100%/99.9% | 100%/99.9% | — (baseline) |
| KLUE morph practical | ~65.8% | **72.1%** | **+6.3pp** |
| KLUE eojeol practical | ~5000 | 5327 | +327 |
| KLUE surface canonical_lenient | ~89% | **95.6%** | **+6pp** |
| UD Kaist morph practical | — | 68.6% | (new dataset) |
| UD GSD morph practical | — | 71.8% | (new dataset) |

## Sprint 159 — 사용자 confirm 필요

### F: NIKL Modu 도입 (silver dataset 확장)

- Academic license 수동 다운로드
- 구어/SNS 도메인 (5-gate CI 추가)
- coverage 확장 효과 (lift는 아님)
- **사용자 confirm 필요**

### E: Full CRF Retrain (Track B)

- 3-5 sprint 장기 작업
- 학습 데이터 + mecab-cost-train
- 잠재 lift +1~5pp KLUE morph
- Sprint 136에서 인프라 조사 완료
- **사용자 confirm 필요**

### 또는 정확도 외 영역

- 문서 정리 (docs/research 아카이브)
- CLI/API 사용성
- 성능 최적화
- 추가 언어 바인딩
- **사용자 우선순위 필요**

## 결정 프로세스

이전 sprints에서 자동 진행했지만, Sprint 159는 비가역 작업이거나 새 방향 선택 필요.
사용자에게 confirm 요청:
1. F (NIKL Modu): 다운로드 + 도메인 확장
2. E (CRF Retrain): 3-5 sprint 정확도 lift
3. 정확도 외 영역으로 전환

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과**
- sample.tsv baseline 100%/99.9% **회귀 금지**
