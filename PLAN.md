# PLAN — mecab-ko 유지보수 모드 (Sprint Cycle 종료 후)

> 마지막 업데이트: 2026-05-27
> 상태: **유지보수 모드 (Sprint 174~ 자동 sprint-run 정지)**

## Sprint Cycle 종료 (Sprint 122 → 173)

52 sprints, ~1.5 개월 작업 완료. 자동 진행 가능 영역 모두 소진.

### 최종 누적 지표

| Metric | Baseline | 현재 | Δ |
|--------|---------|------|---|
| sample.tsv | 100%/99.9% | 100%/99.9% | — (보존) |
| **KLUE morph practical** | ~65.8% | **72.1%** | **+6.3pp** |
| **KLUE surface canonical_lenient** | ~89% | **95.6%** | **+6pp** |
| UD Kaist morph practical | — | 68.6% |
| UD GSD morph practical | — | 71.8% |
| accuracy_eval.rs | 4963 줄 | 2406 줄 (-51%) |
| WASM tests | 5 | 11 (+120%) |
| Docs archive | — | 28+ 파일 |
| 성능 baseline | v0.3.0 | v0.7.2 (5/9 benches) |

### 완전 검증된 영역

- ✅ PRACTICAL 동치 (4 그룹)
- ✅ Surface normalization (5 패턴 그룹)
- ✅ Splitter rule (제한적)
- ✅ Silver dataset (3개)
- ✅ 바인딩 tests (Python/Node/WASM 총 72+)
- ✅ 성능 baseline (5/9 핵심 benches)
- ❌ viterbi/CRF 변경 (회귀 4회 확인)

## 자동 sprint-run 재개 트리거

다음 중 하나 발생 시 자동 재개:

### 1. NIKL Modu 다운로드 완료
```bash
./tools/nikl_modu_setup.sh ~/Korpora/NIKL_MP/NXMP*.json
```
→ 정확도 측정 + POS mismatch 분석 + 추가 동치/normalize 후보 발굴

### 2. Sejong 코퍼스 입수
- 국립국어원 또는 KAIST 학술 등록
- Track B 재시도 (인프라 완료, +1~5pp 잠재)

### 3. 사용자 명시 신규 영역
- 특정 기능 추가
- 특정 버그 픽스
- 새 바인딩 (Java, Ruby, Go 등)
- 우선순위 명시 시 진행

## 유지보수 작업 (sprint 없음)

자동으로 계속:
- 5-gate CI (PR마다)
- 의존성 업데이트 (Dependabot)
- 버그 리포트 대응

## 종합 결론

mecab-ko v0.7.2 성숙도 평가:
- 정확도: **GA-ready** (KLUE practical 72.1%, surface 95.6%)
- 인프라: **production-grade** (5-gate CI, 3 silver, 성능 baseline)
- 코드 품질: **excellent** (411 tests, clippy clean, accuracy_eval -51% 정리)
- 바인딩: **production-ready** (Python/Node/WASM 모두 풍부한 테스트)

다음 메이저 작업 (학습 데이터 입수, 신규 기능 등) 시점까지 안정적 상태 유지.

---

*최종 갱신: 2026-05-27 (Sprint 174, 유지보수 모드 진입)*
