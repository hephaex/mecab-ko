# PLAN — mecab-ko Sprint 127

> 마지막 업데이트: 2026-05-11

## 현재 스프린트: Sprint 127 P1 — 복합명사 분할 정책 분석

### 배경

Sprint 126 P1까지 측정 결과:
- strict 65.8% / lenient 69.3% / practical 70.3% (KLUE DP morpheme)
- 진단 추정(85-90%)과 측정(70.3%) 사이 ~14pp 갭
- NNG/NNP/NNB convention(SL/NNP, NNB/NNG)은 dual equivalence로 흡수
- 잔여 갭의 약 절반은 **복합명사 분할 정책 차이** 추정 (~20%)
- 나머지 절반은 진짜 분석 오류 (Sprint 127 P2 후보)

### 목표

복합명사 분할 정책 차이가 KLUE DP 평가 실패에서 차지하는 비율을 정량화하고,
slice-level matching 메트릭의 적합성을 검토한다.

### 작업 목록

- [ ] **S127P1-01** (analysis): 복합명사 분할 차이 케이스 자동 추출 테스트 추가
  - eojeol 단위로 KLUE 골드 vs mecab 분석 비교
  - "단일 token vs 다중 token" 분할 차이 패턴 탐지
  - 케이스별 sample 출력 (각 패턴 top 20)

- [ ] **S127P1-02** (analysis): 패턴 분류
  - "팝스타→팝+스타" 식 KLUE 단일 / mecab 분할
  - 역방향: KLUE 분할 / mecab 단일 (있으면)
  - 보조 분류: 접두/접미 (한+학생, 검사+장)

- [ ] **S127P1-03** (analysis): Slice-level matching 메트릭 적합성 검토
  - 정의: 어절 내 surface가 일치하면 분할 방식 무관 (양쪽 인정)
  - Trade-off: NNG/NNG vs NNG single은 의미 손실 거의 없음 (downstream OK)
  - 단점: 진짜 분할 오류도 흡수 가능
  - 진단 후 `evaluate_dataset_slice_lenient` 구현 여부 결정

- [ ] **S127P1-04** (docs): 보고서 작성
  - `docs/research/accuracy/2026-05-11_klue_dp_compound_noun.md`
  - 케이스 통계, 분류, slice-level 적합성 결론
  - Sprint 128 권고

- [ ] **S127P1-05** (commit + memory update)

### Sprint 127 후속 후보 (다음 스프린트로)

- **P2**: 진짜 분석 오류 디버그 (NNG/NNP 242건, MAG/NNG 95건, VV/NNG 43건)
- **P3**: noisy 데이터 추가 (사용자 우선순위 높음)
- **P4**: CI 통합 (HF auto-DL + KLUE DP gate 3-mode)

### 검증 기준

- `cargo test --workspace` 전체 pass
- `cargo clippy --workspace --all-targets -- -D warnings` clean
- 새 테스트는 분석/sample 출력만 (회귀 위험 0)
- 보고서가 분할 차이의 실측 비율을 명시
