# Sprint 153 E — XSA+ETM 38건 비이슈 확인 + 자동 트랙 선택 첫 적용

> **결과**: XSA+ETM 38건이 mecab dict decomposition features로 100% 처리됨. ending_rules에 XSA 항목 부재에도 불구하고 converter의 decomp fallback이 ㅂ 불규칙 stem 복원까지 정확히 수행. Sprint 148 D와 동일 패턴 (non-issue). agents.md 규칙 5 (자동 트랙 선택) 첫 실 적용.

---

## 1. 자동 트랙 선택 프로세스 (규칙 5 첫 적용)

### 1.1 전문가 리뷰 의뢰

rust-pro agent에게 토크나이저 전문가 리뷰 요청:
- 입력: 현재 정확도 지표 + 미처리 패턴 빈도 (Sprint 145 분석)
- 출력: Top 권고 + 이유 + 실행 계획

### 1.2 전문가 권고

**Top 권고**: (a) XSA+ETM 38건 분석 → 분리 규칙 추가

**근거** (전문가 분석):
1. `ending_rules.rs`, `splitter.rs` 양쪽에 XSA+ETM 항목 부재 확인 (grep 결과)
2. 어휘 범위 제한 (스러운/스런/로운 — 닫힌 집합)
3. Sprint 150 A VA+ETM 패턴 재사용 가능

**예상 효과**: KLUE morph strict +0.15~0.25pp, sample.tsv 무회귀

### 1.3 자동 채택

규칙 5에 따라 권고를 자동 채택 → 진단 테스트 작성 → 측정.

---

## 2. 진단 결과 — 권고와 다른 발견

### 2.1 측정

```
Raw XSA+ETM: 38
Split by splitter: 38  (100%)
NOT split: 0
```

전문가의 예측 (38건 모두 미처리)과 **반대 결과**: 38건 모두 이미 정확히 분리.

### 2.2 실제 split 결과

```
스러운 → 스럽/XSA + ㄴ/ETM   (ㅂ 불규칙)
스런   → 스럽/XSA + ㄴ/ETM
로운   → 롭/XSA + ㄴ/ETM    (새롭 + ㄴ → 새로운)
다운   → 답/XSA + ㄴ/ETM    (아름답 + ㄴ → 아름다운)
스러울 → 스럽/XSA + ㄹ/ETM
```

ㅂ 불규칙 stem 복원 (스럽/롭/답)까지 정확히 수행.

### 2.3 처리 메커니즘 — converter.rs L162-187

`SejongConverter::convert_token`:
1. **First try**: mecab dict의 `decomposition` features 추출
2. POS 구조 일치 확인 (`decomp_pos == token.pos`)
3. 일치하면 decomposition 결과 직접 사용
4. 불일치 시에만 ending_rules/splitter 폴백

mecab-ko-dic 2.1.1의 XSA 엔트리들이 features 컬럼에 `스럽/XSA+ㄴ/ETM` 같은 분해 정보를 포함하고 있어 자동 처리됨.

---

## 3. 핵심 학습 포인트

### 3.1 ending_rules 부재 ≠ 미처리

전문가 리뷰조차 `ending_rules`/`splitter`만 보고 미처리로 잘못 추정. 실제로는:
- `SejongConverter::convert_token` L162-187의 **decomposition fallback**이 mecab dict features를 활용
- 이 경로가 ending_rules보다 먼저 시도됨
- 사전이 분해 정보를 제공하면 ending_rules는 무관

**적용 원칙**: 정확도 진단 시 다음 모든 경로 확인 필요:
1. mecab dict decomposition features
2. SejongConverter 특수 케이스 (skip_decomp 등)
3. splitter.rs ending_rules
4. splitter.rs fallback compound split

### 3.2 자동 트랙 선택의 가치 — 빠른 가설 검증

규칙 5 (전문가 리뷰 자동 채택)로 사용자 question 없이 진행. 결과:
- 진단 빠르게 완료 (10분)
- 가설 (XSA+ETM 미처리) 즉시 검증 → 반증
- 다음 트랙으로 즉시 전환 가능

만약 사용자 question 단계가 있었다면 같은 결과에 도달하는 데 더 오래 걸렸을 것.

### 3.3 Sprint 148 D 패턴 재현

| 항목 | Sprint 148 D (ETM+ETM) | Sprint 153 E (XSA+ETM) |
|------|----------------------|----------------------|
| Raw 빈도 | 33건 | 38건 |
| 추정 미처리 | 33건 | 38건 |
| 실제 미처리 | 0건 | 0건 |
| 처리 메커니즘 | splitter L71-73 중복 태그 규칙 | converter L162-187 decomp fallback |
| 결과 | 비이슈 | 비이슈 |
| 가치 | 진단 테스트 + 학습 | 진단 테스트 + 학습 + 자동 트랙 첫 적용 |

**공통 패턴**: 빈도 분석은 raw 수치만 보여줌 → splitter/converter 변환 후 진단 필수.

### 3.4 규칙 5 운영 효과

- ✅ 빠른 진행 (사용자 confirm 단계 제거)
- ✅ 전문가 의견 활용
- ⚠️ 전문가도 틀릴 수 있음 → **항상 측정으로 검증**
- 결론: 자동 채택 + 진단으로 reasonable

---

## 4. 변경 파일

- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`: `test_xsa_etm_post_splitter_mismatch` 진단 추가
- `docs/research/accuracy/2026-05-21_sprint153_xsa_etm_nonissue.md` (본 문서)
- `PLAN.md`, `PROGRESS.md` 갱신

---

## 5. Sprint 154 후보 (자동 결정 예정)

남은 안전한 패턴:
- EP+ETM 86건 (던, 는, 신) — `ending_rules` 확인 결과 `VV+EP+ETM`만 존재, 단독 EP+ETM 미처리 가능성
- XSV+ETM 72건 (던, 헌, 시킬) — 미처리 가능성
- VX+EP 25건 (했, 못했, 왔) — 보조용언 + 과거
- XSA+EP 35건 (했, 스러웠, 허)

**다음 결정**: 자동 트랙 선택 (전문가 리뷰 → 진단 → 결정).

---

*작성: 2026-05-21 (Sprint 153 E)*
