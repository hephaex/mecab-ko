# PLAN — mecab-ko Sprint 168 (Track B 결정 — 사용자 confirm 필요)

> 마지막 업데이트: 2026-05-27

## 완료: Sprint 167 — Track B Step 4 실패 → Rollback

### 결과

- 새 CRF dict 사용 시 sample.tsv Token: 100% → **62.2%** (-37.8pp)
- 즉시 rollback (Sprint 138 정책)
- baseline 복원 확인

### 원인

학습 데이터 features 부족 (POS only). mecab feature.def가 활용하는 semantic/reading features 부재 → CRF overfit → general regression.

### Track B 1차 시도 종합

| Sprint | Step | 결과 |
|--------|------|------|
| 164 | 빌드 환경 | ✅ macOS arm64 |
| 165 | 학습 데이터 (UD dev 3016 sentences) | ✅ |
| 166 | 학습 + dict 변환 (62.6초 파이프라인) | ✅ |
| **167** | **Rust 통합** | **❌ -37.8pp** |

## Sprint 168 — 사용자 결정 옵션

### Option A: Self-training (학습 data features 보강)

**작업**: 기존 mecab-ko-dic으로 KLUE/UD train tokenize → 그 features (POS + semantic + reading)를 학습 데이터로 사용.

**장점**: 학습 features 풍부해짐
**단점**: 기존 mecab의 오류를 학습 (self-amplification). 결과적으로 기존 이상 못 함

### Option B: 학습 corpus 확장 + leakage 허용

**작업**: KLUE val (1995) + UD test (2609) 모두 학습에 사용. 별도 hold-out test set 마련.

**장점**: 학습 data 크기 ~7000 sentences로 확장
**단점**: 평가 leakage → 결과 신뢰도 낮음. 신뢰성 있는 hold-out test set 만들기 어려움

### Option C: Sejong 코퍼스 입수 (학술 라이선스)

**작업**: 국립국어원 또는 KAIST에서 Sejong tagged corpus 학술 입수. 원본 mecab-ko-dic 학습 데이터와 동급.

**장점**: 가장 정확한 학습 (mecab features full coverage)
**단점**: 라이선스 절차 (시간 소요), 자동화 불가

### Option D [권고]: Track B 종료

**작업**: CRF retrain 시도 종료. 정확도 sprint 종료 또는 다른 방향 (NIKL Modu 다운로드 / 새 영역).

**장점**: 명확한 결론 (Track B는 학습 데이터 quality가 절대적)
**단점**: +1~5pp lift 기회 상실

## Track B 학습 정리

이번 시도로 얻은 가치:
1. ✅ legacy/ macOS arm64 빌드 가능 (Sprint 164)
2. ✅ 학습 파이프라인 검증 (Sprint 166, 62.6초)
3. ✅ Rust dict-builder 통합 (Sprint 167)
4. ✅ 격리 메커니즘 (별도 dict + 환경 변수) 검증

차후 누군가 Sejong 코퍼스 입수 시:
1. tools/to_mecab_tagged.py (Sprint 165, UD 형식)
2. seed/ 디렉토리 준비 패턴 (Sprint 166)
3. 4단계 파이프라인 (dict-index, cost-train, dict-gen, dict-index) 명세

는 모두 재사용 가능. **인프라 자체는 가치**.

## 누적 진척 (Sprint 122 → 167)

| Metric | Baseline | 현재 (Track B rollback 후) |
|--------|---------|------------------------|
| sample.tsv | 100%/99.9% | 100%/99.9% (보존) |
| KLUE morph practical | ~65.8% | **72.1%** (+6.3pp) |
| KLUE surface canonical_lenient | ~89% | **95.6%** (+6pp) |
| UD Kaist morph practical | — | 68.6% |
| UD GSD morph practical | — | 71.8% |

Track B 시도 후에도 baseline 손상 없음 (격리 효과).

## 결정 프로세스

비가역 작업 → 사용자 confirm 필수. 다음 sprint-run 시 사용자 옵션 명시 시 진행.
