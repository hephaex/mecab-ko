# Sprint 146 A — 명시 surface 안전 패턴 분리 (NP+JX skip, VCP+EP "였" 추가)

> **결과**: Sprint 145 분석 기반 안전 패턴 시도. NP+JX는 데이터 검증 후 skip 결정 (KLUE에 결합 surface 부재). VCP+EP "였" 분리 추가 (101건 패턴). 실측 lift 없으나 형태론적 정확성 향상 + 단위 테스트로 정확성 보장.

---

## 1. NP+JX 검증 → Skip 결정

### 1.1 가설

Sprint 145 분석: NP+JX 211건 결합 토큰 (samples: "그는", "게다가", "난").

### 1.2 mecab CLI 직접 확인

```
$ echo "그는\n이는\n난\n게다가\n저는" | mecab-cli
그	NP    ← 이미 분리됨
는	JX
이	NP    ← 이미 분리됨
는	JX
난	NP+JX ← 결합 (contraction "나는")
게다가	NP+JX ← 결합
저	NP    ← 이미 분리됨
는	JX
```

→ "그는"/"이는"/"저는"은 mecab이 이미 분해. 결합 토큰은 contraction ("난", "게다가").

### 1.3 KLUE gold 확인

```bash
$ grep -oE "[가-힣]+/NP \S*/JX" data/eval/klue_dp_val.tsv | sort -u
저/NP 는/JX
아무/NP 도/JX
여기/NP 까지/JX
# "난"이나 "게다가" 결합 surface는 KLUE에 없음
```

KLUE gold에 "난"/"게다가" NP+JX 결합 surface 등장 안 함. 모두 분리된 morpheme.

### 1.4 결정: Skip

- mecab 분리 출력은 이미 KLUE/UD와 일치 (그는/저는)
- mecab 결합 출력 (난/게다가)은 KLUE에 등장 안 함 → 분리 시도 시 false morpheme 추가
- 안전한 분리 대상 surface 없음 → **skip**

---

## 2. VCP+EP "였" 분리 추가

### 2.1 가설

VCP+EP 101건 결합 토큰 (sample: "였"). KLUE gold:
```
이/VCP 었/EP   ← gold는 분리
```

mecab은 결합으로 출력할 수 있음 (단 mecab CLI에서는 "였/EP" 단독).

### 2.2 구현 (splitter.rs)

```rust
if pos == "VCP+EP" && surface == "였" {
    return vec![
        ("이".to_string(), "VCP".to_string()),
        ("었".to_string(), "EP".to_string()),
    ];
}
```

명시 surface "였"만 처리. 다른 VCP+EP는 일반 split 흐름.

### 2.3 단위 테스트 (2개 신규)

- `test_split_morpheme_vcp_ep_yeoss`: "였" → "이/VCP + 었/EP" 검증
- `test_split_morpheme_vcp_ep_other_surface_no_split`: 명시 외 surface skip

### 2.4 5-gate 측정 결과

| 메트릭 | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv Token / Sentence | 100.0% / 99.9% | 100.0% / 99.9% | — |
| KLUE morph / eo strict | 66.8% / 20.7% | 66.8% / 20.7% | — |
| KLUE practical | 71.6% / 23.5% | 71.6% / 23.5% | — |
| Surface-only canonical_lenient | 95.5% | 95.5% | — |
| UD Kaist morph / eo strict | 66.3% / 20.7% | 66.3% / 20.7% | — |
| UD GSD morph / eo strict | 67.4% / 23.1% | 67.4% / 23.1% | — |

**모든 메트릭 동일** — 실측 lift 없음.

### 2.5 원인 분석

Sprint 145 분석에서 VCP+EP 101건 ("였")은 token POS에 `+`가 포함된 raw mecab feature. 실제 `tokenize()` 결과는 SejongConverter 거치기 전 raw. CLI 출력에서는 "였/EP" 단독으로 보임 → mecab의 실제 token 분해는 split_morpheme 호출 전후 다를 수 있음.

가능한 해석:
- VCP+EP 결합은 raw feature에만 존재, SejongConverter 처리 단계에서 이미 다른 path 흐름
- 또는 측정 도구의 변동 폭 내 (eojeol count 단위 변화 없음)

### 2.6 결정: 유지

실측 lift 0이지만:
- 형태론적으로 정확한 분리 (KLUE gold와 일치)
- 단위 테스트로 정확성 보장
- 향후 mecab dict 업데이트로 VCP+EP 결합 출력 증가 시 즉시 대응
- 회귀 없음 (sample.tsv + 5-gate 모두 동일)

**downstream 일관성 향상** 가치로 유지.

---

## 3. 핵심 학습 포인트

### 3.1 분석 빈도 ≠ 실측 영향

Sprint 145 분석에서 VCP+EP 101건은 `Token.pos` 문자열에 `+` 포함된 모든 케이스. 실측 평가에서는 `evaluate_dataset_dual` 내부의 SejongConverter 처리 후 비교 → 분석 단계의 raw count가 실측 lift와 직접 비례하지 않음.

**적용 원칙**:
빈도 분석 후 실측 검증 필수. 큰 빈도라도 SejongConverter 다른 path를 거치면 영향 없을 수 있음.

### 3.2 mecab CLI 출력은 절대적 기준

splitter.rs 추가 패턴 시 mecab CLI 직접 확인 + KLUE gold 비교가 가장 정확한 기준. 분석 테스트의 `Token.pos` 통계는 후처리 전 raw feature.

### 3.3 NP+JX 같이 mecab이 이미 분리하는 패턴은 skip

mecab의 분리 출력과 KLUE gold가 일치하는 패턴은 추가 작업 불필요. 결합 출력만 분리 후보.

### 3.4 형태론적 정확성도 commit 가치

실측 lift 0이라도 형태론적으로 정확한 코드 변경은 downstream 일관성 향상. 단 회귀 없음 검증 필수.

---

## 4. Sprint 147 후보

### 후보 A: 추가 안전 패턴 (예: XSV+EP, XSV+EC 명시 surface)

XSV+EP 413건, XSV+EC 751건. 명시 surface 식별 + 분리 시도.

### 후보 B [메인]: Full CRF Retrain (Track E)

3-5 sprint, 메인 목표.

### 후보 C: NIKL Modu 수동 다운로드

Academic license, 도메인 확장 (구어/SNS).

### 후보 D: VV+EP 명시 동사 분리

VV+EP 542건. 명시 동사 surface 식별 ("흘렸"/"버렸" 등) → "VV + 었/EP" 분리.

---

## 5. 변경 파일

- `rust/crates/mecab-ko-core/src/sejong/splitter.rs`:
  - VCP+EP "였" 분리 추가 (~7줄)
  - 단위 테스트 2개 (yeoss + other_surface_no_split)
- `docs/research/accuracy/2026-05-20_sprint146_explicit_surface_splits.md` (본 문서)
- `PLAN.md`, `PROGRESS.md` 갱신

---

*작성: 2026-05-20 (Sprint 146 A)*
