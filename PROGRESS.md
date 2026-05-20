# PROGRESS — mecab-ko Sprint 153 (XSA+ETM 비이슈 + 규칙 5 첫 적용)

> 마지막 업데이트: 2026-05-21

## Sprint 153 E — XSA+ETM 진단 (자동 트랙 선택)

| Task | 상태 | 결과 |
|------|------|------|
| S153-E1: 전문가 리뷰 (rust-pro) 의뢰 | ✅ 완료 | Top 권고: XSA+ETM 38건 |
| S153-E2: 규칙 5 자동 채택 | ✅ 완료 | 사용자 question 없이 진행 |
| S153-E3: 진단 테스트 작성 | ✅ 완료 | `test_xsa_etm_post_splitter_mismatch` |
| S153-E4: 측정 | ✅ 완료 | 38/38 처리됨 — **비이슈** |
| S153-E5: 처리 메커니즘 분석 | ✅ 완료 | converter L162-187 decomp fallback |
| S153-E6: 연구 문서 작성 | ✅ 완료 | sprint153_xsa_etm_nonissue.md |

## 핵심 발견

### XSA+ETM 38건 = 이미 100% 처리

**처리 메커니즘**: `SejongConverter::convert_token` (converter.rs L162-187)

1. mecab dict의 `decomposition` features 추출
2. POS 구조 일치 확인 (`decomp_pos == token.pos`)
3. 일치하면 decomposition 직접 사용

### 실제 split 출력 (sample)

```
스러운 → 스럽/XSA + ㄴ/ETM   (ㅂ 불규칙 stem 복원)
스런   → 스럽/XSA + ㄴ/ETM
로운   → 롭/XSA + ㄴ/ETM    (새롭 + ㄴ → 새로운)
다운   → 답/XSA + ㄴ/ETM    (아름답 + ㄴ → 아름다운)
스러울 → 스럽/XSA + ㄹ/ETM
```

mecab-ko-dic이 ㅂ 불규칙 stem 복원까지 정확히 제공.

### 자동 트랙 선택 (규칙 5) 첫 적용 결과

| 항목 | 효과 |
|------|------|
| 사용자 question 제거 | ✅ 빠른 진행 |
| 전문가 권고 활용 | ✅ 가설 명확 |
| 측정으로 검증 | ✅ 가설 반증 (전문가 추정 != 실측) |
| 다음 트랙 즉시 전환 | ✅ Sprint 154 자동 결정 가능 |

**중요**: 전문가도 틀릴 수 있다 — **항상 측정으로 검증**.

### Sprint 148 D 패턴 재현

| 항목 | Sprint 148 D | Sprint 153 E |
|------|------------|------------|
| Raw 빈도 | 33 (ETM+ETM) | 38 (XSA+ETM) |
| 추정 미처리 | 33 | 38 |
| 실제 미처리 | 0 | 0 |
| 처리 위치 | splitter L71-73 | converter L162-187 |

빈도 분석 → splitter/converter 변환 후 진단 필수.

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (테스트만 추가)
- 5-gate sample.tsv: 영향 없음 (코드 변경 없음)
- `test_xsa_etm_post_splitter_mismatch`: PASS (38/38 처리됨)

## 변경 파일

- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`: 진단 테스트 추가
- `docs/research/accuracy/2026-05-21_sprint153_xsa_etm_nonissue.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 154 후보 (자동 결정 예정)

전문가 리뷰로 다음 후보 중 자동 선택:
- EP+ETM 86건 (던, 는, 신)
- XSV+ETM 72건 (던, 헌, 시킬)
- VX+EP 25건 (했, 못했, 왔)
- XSA+EP 35건 (했, 스러웠, 허)
- 또는 전문가가 식별하는 새 영역
