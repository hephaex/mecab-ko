# Sprint 148 D — ETM+ETM "라는" 분석: 비이슈 확인

> **결과**: mecab의 ETM+ETM "라는" 33건은 SejongConverter 중복 태그 규칙으로 이미 정규화됨. gold 비교 mismatch 0건. 코드 변경 불필요.

---

## 1. 분석 동기

Sprint 145 compound POS 빈도 분석에서 `ETM+ETM 33건, samples: 라는` 발견.
"mecab 비정상 출력"으로 분류 → Sprint 148 D에서 실제 영향 조사.

---

## 2. mecab 출력 vs gold 비교

### 2.1 mecab raw output

mecab은 "라는"을 `ETM+ETM`으로 분석:
- 내부 형태소: `라/ETM + 는/ETM` (관형형어미 복합)
- "이라는" 등 맥락에서 발생

### 2.2 KLUE/UD gold

gold는 "라는"을 단일 ETM으로 처리:
```
이/VCP 라는/ETM 진술  — "이라는" eojeol 분리 시
라는/ETM 것         — 독립 eojeol
아니/VCN 라는/ETM   — 부정사 뒤
```

### 2.3 SejongConverter 처리 (splitter.rs L71-73)

```rust
// 중복 태그 처리: "JKB+JKB" 같은 경우 첫 번째 태그만 사용
if tags.len() >= 2 && tags[0] == tags[1] && pos != "EP+EP" {
    return vec![(surface.to_string(), tags[0].clone())];
}
```

ETM+ETM → `tags[0] == tags[1] == "ETM"` → `[("라는", "ETM")]` 반환.
이미 gold 형식으로 정규화.

---

## 3. 진단 결과

`test_etm_etm_raneun_diagnosis` (Sprint 148 D):

| Dataset | ETM+ETM "라는" 건수 | Gold mismatch |
|---------|-------------------|---------------|
| KLUE DP val | 8건 | 0 |
| UD Kaist test | (포함) | 0 |
| UD GSD test | (포함) | 0 |
| **전체** | **33건** | **0** |

---

## 4. 언어학적 배경

"라는"은 한국어에서 인용 축약형:
- "이라고 하는" → "이라는" → eojeol 분리 후 "라는"
- mecab의 내부 분석: `라/ETM + 는/ETM` (관형형어미 연속 결합)
- gold 표준: `라는` 자체를 단일 ETM으로 처리

두 분석 모두 언어학적으로 합리적. SejongConverter가 중간에서 정규화.

---

## 5. 핵심 학습 포인트

### 5.1 빈도 분석 → 실제 영향 확인이 필수

raw compound POS 빈도 (33건)가 실제 evaluation 영향을 의미하지 않음.
SejongConverter 변환 후 비교해야 실제 영향 측정 가능.

**적용 원칙**: 빈도 분석 발견 → SejongConverter 변환 경로 추적 → 실측 mismatch 확인.

### 5.2 중복 태그 규칙의 광범위한 적용

splitter.rs L71-73의 중복 태그 규칙은 ETM+ETM 외에도 JKB+JKB, NNG+NNG 등 다양한 중복 패턴에 일반 적용됨. 이 규칙이 실제로 어느 패턴을 커버하는지 추가 조사 가능.

### 5.3 "비정상 출력" 판단 기준

mecab의 compound POS가 항상 "비정상"은 아님. 언어학적으로 합리적인 내부 분석이 gold 표준과 다를 수 있으나 변환 레이어가 이를 처리하는 경우 "비이슈".

판단 흐름:
1. raw 빈도 확인
2. SejongConverter 변환 추적
3. gold 비교
4. mismatch > 0이어야 조치 대상

---

## 6. 변경 파일

- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`: `test_etm_etm_raneun_diagnosis` 추가
- `docs/research/accuracy/2026-05-20_sprint148_etm_etm_raneun_nonissue.md` (본 문서)
- `PLAN.md`, `PROGRESS.md` 갱신

---

*작성: 2026-05-20 (Sprint 148 D)*
