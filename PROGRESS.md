# PROGRESS — mecab-ko Sprint 148 (ETM+ETM "라는" 분석)

> 마지막 업데이트: 2026-05-20

## Sprint 148 D — ETM+ETM "라는" 분석 스프린트

| Task | 상태 | 비고 |
|------|------|------|
| S148-D1: ETM+ETM "라는" 빈도 확인 | ✅ 완료 | 33건 (3 datasets), KLUE 8건 |
| S148-D2: mecab 출력 → SejongConverter 변환 추적 | ✅ 완료 | 중복 태그 규칙으로 이미 처리됨 |
| S148-D3: gold 비교 mismatch 측정 | ✅ 완료 | 0 mismatch — 비이슈 확인 |
| S148-D4: 진단 테스트 추가 | ✅ 완료 | `test_etm_etm_raneun_diagnosis` |

## 핵심 발견

### ETM+ETM "라는" = 이미 올바르게 처리됨

**분석**:
- 빈도: 33건 (3 silver 통합)
- mecab raw output: `라는/ETM+ETM`
- SejongConverter (splitter.rs L71-73): 중복 태그 규칙 → `ETM+ETM → ETM`
- 변환 후: `라는/ETM`
- gold 기대값: `라는/ETM`
- **mismatch: 0건**

**중복 태그 규칙 (splitter.rs)**:
```rust
// 중복 태그 처리: "JKB+JKB" 같은 경우 첫 번째 태그만 사용
if tags.len() >= 2 && tags[0] == tags[1] && pos != "EP+EP" {
    return vec![(surface.to_string(), tags[0].clone())];
}
```

→ ETM+ETM이므로 `tags[0] == tags[1] == "ETM"` → `[(라는, ETM)]` 반환.

### 언어학적 배경

"라는"은 한국어에서 "이라고 하는"의 축약형:
- mecab: `라/ETM + 는/ETM` → ETM+ETM (내부 형태소 분리)
- gold (KLUE/UD): `라는/ETM` (단일 형태소로 처리)

mecab의 ETM+ETM 분석은 언어학적으로 합리적이며, SejongConverter가 이미 gold 형식으로 정규화.

### 코드 변경 없음

Sprint 148 D = 분석 스프린트. 기존 코드가 이미 올바르게 동작함을 확인.

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: **399 passed / 0 failed** (변경 없음)
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings`: clean
- `test_etm_etm_raneun_diagnosis`: PASS (33건 탐지, 0 mismatch)

## 변경 파일

- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`: `test_etm_etm_raneun_diagnosis` 추가
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 149 후보

- A: VA+ETM 542건 분석 (어려울, 바른, 큰 — 형용사 활용)
- B: Full CRF Retrain (Track E)
- C: NIKL Modu 수동 다운로드
- E: 추가 practical 동치 후보 조사 (ETM+ETM 外 패턴)
