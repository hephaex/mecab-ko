# Sprint 134 P3 — Normalize Endings Extension

> **한 줄 요약**: `normalize_endings`에 `이습니다→입니다`와 `하아→하여` 2개 규칙 추가. KLUE DP `canonical_lenient` surface-only **94.4% → 95.4%** (+1.0pp). SURFACE_MISMATCH 흡수율 54.3% → 62.3% (+8pp). Sample.tsv 100%/99.9% 무회귀.

## 배경

Sprint 133 P2에서 측정한 `canonical_lenient` 94.4%의 흡수율을 더 높이기 위해 Sprint 128 P2의 `normalize_endings` 함수에 추가 규칙. Sprint 133의 normalization_analysis 출력에서 식별한 top 패턴:

| Pattern | Occurrences | Type |
|---------|-------------|------|
| `이습니다 → 입니다` | ~80 | 다중-char 치환 |
| `하아 → 하여` | ~12 | char-pair (기존 패턴 확장) |

이 두 규칙으로 추가 ~92 cases 흡수 기대.

## 구현

### evaluate.rs `normalize_endings` 확장

```rust
fn normalize_endings(s: &str) -> String {
    // Step 1: char-pair 변환 (하았/하어/하아)
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    for (i, &c) in chars.iter().enumerate() {
        let prev = if i > 0 { chars[i - 1] } else { '\0' };
        if prev == '하' && (c == '았' || c == '아') {
            // 하았 → 하였, 하아 → 하여
            out.push(if c == '았' { '였' } else { '여' });
        } else if c == '어' && prev == '하' {
            out.push('여');
        } else {
            out.push(c);
        }
    }
    // Step 2: 다중-char 패턴 (이습니다 → 입니다)
    if out.contains("이습니다") {
        out = out.replace("이습니다", "입니다");
    }
    out
}
```

### 새 unit tests (3개)

- `test_surface_eq_canonical_lenient_haa_to_haye`: 편하아요 ↔ 편하어요
- `test_surface_eq_canonical_lenient_imnida`: 것입니다 ↔ 것이습니다, 이ㅂ니다 ↔ 이습니다
- `test_surface_eq_canonical_lenient_imnida_overcorrect`: overcorrect 방지 boundary

### accuracy_eval.rs `test_klue_dp_surface_normalization_analysis` 갱신

분석 테스트에 있던 로컬 `normalize_endings` 복사본을 새 규칙과 동기화. 그렇지 않으면 분석 출력이 stale 데이터 보여줌.

## 측정 결과 (KLUE DP val, 1,995 sentences, 22,404 어절)

### Surface-only metric (Sprint 133 신규 메트릭 재측정)

| Mode | Sprint 133 baseline | Sprint 134 | Δ |
|------|---------------------|-----------|---|
| strict | 87.7% | 87.7% | (rule만 lenient에 적용) |
| canonical | 91.6% | 91.6% | — |
| **canonical_lenient** | **94.4%** | **95.4%** | **+1.0pp** |

Canonical_lenient Δ vs strict: +6.7pp → **+7.6pp** (+0.9pp gain).

### Absorption analysis

| Tier | Sprint 128 | Sprint 134 |
|------|------------|-----------|
| NFC compose only | 873 (31.8%) | 873 (31.8%) |
| NFC + ending rules | 619 (22.5%) | **839 (30.5%)** |
| Still mismatch | 1257 (45.7%) | **1037 (37.7%)** |
| **Total absorbed** | **1492 (54.3%)** | **1712 (62.3%)** |

이중 cases 220개 추가 흡수. Eojeol_surface_only lift +1.0pp와 일치.

### 회귀

| 메트릭 | Before | After | 변화 |
|--------|--------|-------|------|
| Sample.tsv Token | 100.0% | 100.0% | 무회귀 |
| Sample.tsv Sentence | 99.9% | 99.9% | 무회귀 |
| KLUE strict morph | 66.8% | 66.8% | (normalize_endings는 lenient에만 적용) |
| All 40 ignored tests | pass | pass | — |
| clippy | 0 | 0 | — |

## 핵심 학습 포인트

### 1. 메트릭 천장 vs 룰 추가 비용

Sprint 132 이후 dict 트랙 천장(+0.039pp/entry → +0.007pp/entry). Sprint 134의 normalize rule 추가:
- 2 rules → +1.0pp surface_only canonical_lenient lift
- ≈ +0.5pp/rule (Sprint 130 dict 빈도 5+와 동급 효율)

**Normalize rule 추가는 dict 천장보다 lift 효율 높음** — 단, surface-only 메트릭 한정. 형태소 분석 메트릭(strict/practical morph)에는 무효.

### 2. Sprint 128 P2 +0pp 가설 일부 반박

Sprint 128 P2에서 surface_lenient가 형태소 메트릭에서 +0pp였던 원인:
- 형태소 분석은 split mismatch가 dominant
- Surface concat만 맞아도 split이 다르면 무효

Sprint 133에서 use case 분리 메트릭(surface_only)을 만들면서 이 규칙의 가치가 회복됨. Sprint 134에서 추가 +1.0pp 달성 — Sprint 128에서 만들었던 인프라가 Sprint 133의 use case 분리 후 비로소 lift 측정 가능.

**측정 메트릭이 잘못되면 좋은 작업도 +0pp로 보임**. Sprint 128 surface_lenient 인프라는 처음부터 정확했으나 측정 메트릭이 부적합했음.

### 3. 멀티-char 치환 vs char-pair

기존 char-pair (하았/하어): 인덱스 i-1, i 단순 비교.
신규 멀티-char (이습니다): `String::replace` 4-char 치환.

성능: replace는 char-pair 루프보다 약간 느리지만 의미 있는 차이 아님 (eval 1.17s, 21K 어절). 가독성·확장성이 char-pair 패턴보다 좋음.

### 4. Stale 분석 테스트의 위험

`test_klue_dp_surface_normalization_analysis`가 로컬 `normalize_endings` 복사본을 사용. evaluate.rs의 정식 함수를 갱신해도 분석 출력은 stale. 본 sprint에서 분석 테스트도 함께 갱신 — **분석 도구는 측정 도구와 동기 유지가 필수**. 향후 normalize_endings 추가 변경 시 양쪽 sync 잊지 말 것.

## Sprint 135 권고

### 추가 normalize rule 후보 (검토 필요)

Sprint 134 후 remaining mismatch top 패턴:
- 따르아 → 따라 (18×): 동사 활용 contraction
- 것이 → 게 (12×): 대명사 contraction
- 앞서 → 앞서어 (12×): 어미 보존 — gold가 더 짧음 (반대 방향)
- 갑니다 → 가이습니다 (5×): "이습니다" 룰이 가 + 이습니다 분해를 못 흡수

이들은 의미 단위 contraction으로 자동 norm 위험. 사례별 분석 후 안전 case만 추가 권고 — 또는 별도 sprint로 분리.

### 후속 트랙 (Sprint 132+ 결정과 일관)

- **P1**: Noisy data 추가 (deferred → 이번에도 보류 가능)
- **P4**: CRF retrain 인프라 조사 (대규모 작업)
- **P5**: borderline NNG↔NNP normalization layer

## 관련 문서

- [Sprint 128 P2 — Surface lenient infra](./2026-05-11_klue_dp_surface_lenient.md) — normalize_endings 원본
- [Sprint 133 P2 — Surface-only metric](./2026-05-18_sprint133_eojeol_surface_only.md) — 본 lift가 측정되는 메트릭

---

*작성: 2026-05-18*
