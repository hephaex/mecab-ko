# Sprint 33 최종 세션 로그

**날짜**: 2026-03-10
**세션 유형**: PM Auto Mode (Sprint 33)
**시작 정확도**: 58.2% (이전 세션에서)
**최종 정확도**: 59.2%
**총 개선 폭**: +2.6% (Sprint 32 대비)

---

## 1. 세션 목표

### Primary Goals
1. EC (연결어미): 31.6% → 50%+ ✅ **달성 (51.3%)**
2. ETM (관형사형어미): 25.8% → 40%+ ✅ **달성 (48.4%)**
3. 전체 정확도: 60%+ ⏳ **진행 중 (59.2%, 0.8% 남음)**

---

## 2. 구현된 솔루션 상세

### 2.1 VV+EF 복합 패턴 분리 (split_morpheme 함수)

**파일**: `rust/crates/mecab-ko-core/src/sejong.rs`

```rust
// VV+EF 특별 패턴 처리
if pos == "VV+EF" {
    // "ㅂ니다" 패턴: "합니다" → "하/VV + ㅂ니다/EF"
    if surface.ends_with("니다") && surface.chars().count() >= 3 {
        let chars: Vec<char> = surface.chars().collect();
        let first_char = chars[0];
        if let Some(stem) = Self::remove_jongseong_bieup(first_char) {
            if chars.len() == 3 {
                return vec![
                    (stem.to_string(), "VV".to_string()),
                    ("ㅂ니다".to_string(), "EF".to_string()),
                ];
            }
        }
    }

    // "ㄹ게요" 패턴: "할게요" → "하/VV + ㄹ게요/EF"
    if surface.ends_with("게요") && surface.chars().count() >= 3 {
        let chars: Vec<char> = surface.chars().collect();
        let stem_char = chars[chars.len() - 3];
        if let Some(stem) = Self::remove_jongseong_rieul(stem_char) {
            let prefix: String = chars[..chars.len() - 3].iter().collect();
            let full_stem = format!("{}{}", prefix, stem);
            return vec![
                (full_stem, "VV".to_string()),
                ("ㄹ게요".to_string(), "EF".to_string()),
            ];
        }
    }

    // "ㄹ까요" 패턴: "할까요" → "하/VV + ㄹ까요/EF"
    if surface.ends_with("까요") && surface.chars().count() >= 3 {
        // ... 동일한 패턴
    }

    // "ㄹ래요" 패턴: "할래요" → "하/VV + ㄹ래요/EF"
    if surface.ends_with("래요") && surface.chars().count() >= 3 {
        // ... 동일한 패턴
    }

    // "해요" → "하/VV + 아요/EF"
    if surface == "해요" {
        return vec![
            ("하".to_string(), "VV".to_string()),
            ("아요".to_string(), "EF".to_string()),
        ];
    }

    // "봐요" → "보/VV + 아요/EF"
    if surface == "봐요" {
        return vec![
            ("보".to_string(), "VV".to_string()),
            ("아요".to_string(), "EF".to_string()),
        ];
    }
}
```

### 2.2 53차 보정: XPN 패턴 확장

```rust
// 50차 보정: "MM + NNG" 패턴의 MM → XPN 변환
// 53차 추가: 새, 첫, 맨, 헛, 옛, 순 (sample.tsv 기반)
let prefix_patterns: std::collections::HashSet<&str> = [
    "전", "현", "신", "구", "친", "총", "부", "대",
    "새", "첫", "맨", "헛", "옛", "순",  // 53차 추가
]
.into_iter()
.collect();

// 53차 보정: "VA + NNG" 패턴 중 접두사 후보는 XPN으로 변환
// "큰/VA 집/NNG" → "큰/XPN 집/NNG"
if curr_pos == "VA"
    && (curr_surface == "큰" || curr_surface == "작")
    && (next_pos == "NNG" || next_pos == "NNP")
{
    tokens[i].pos = "XPN".to_string();
}
```

### 2.3 54차 보정: 있/VX → 있/VV 변환

```rust
// 54차 보정: 있/VX → 있/VV 변환 (보조동사가 아닌 경우)
// "있/VX"가 앞에 "고/EC"가 없으면 본동사 VV로 변환
// 예: "시간 있/VX 어요" → "시간 있/VV 어요"
// 단, "가고 있/VX 다"는 보조동사이므로 유지
for i in 0..tokens.len() {
    if tokens[i].surface == "있" && tokens[i].pos == "VX" {
        let is_auxiliary = i > 0
            && tokens[i - 1].surface == "고"
            && tokens[i - 1].pos == "EC";
        if !is_auxiliary {
            tokens[i].pos = "VV".to_string();
        }
    }
}
```

### 2.4 55차 보정: EC 뒤 "합니다" 병합

```rust
// 55차 보정: EC 뒤의 "하/VV + ㅂ니다/EF" → "합니다/EF" 병합
// "해야 합니다" 패턴에서 합니다는 보조용언으로 분리하지 않음
let mut i = 0;
while i < tokens.len().saturating_sub(1) {
    if tokens[i].pos == "VV"
        && (tokens[i].surface == "하" || tokens[i].surface == "가" || tokens[i].surface == "오")
        && tokens[i + 1].pos == "EF"
        && tokens[i + 1].surface == "ㅂ니다"
    {
        let after_ec = i >= 1 && tokens[i - 1].pos == "EC";
        if after_ec {
            let merged = match tokens[i].surface.as_str() {
                "하" => "합니다".to_string(),
                "가" => "갑니다".to_string(),
                "오" => "옵니다".to_string(),
                _ => format!("{}ㅂ니다", tokens[i].surface),
            };
            tokens[i].surface = merged;
            tokens[i].pos = "EF".to_string();
            tokens.remove(i + 1);
            continue;
        }
    }
    i += 1;
}
```

### 2.5 56차 보정: ETM 유니코드 정규화

```rust
// 56차 보정: ETM 표면형 유니코드 정규화
// 한글 자모 (U+1100~U+11FF) → 호환 자모 (U+3130~U+318F)
// 예: "ᆫ/ETM" → "ㄴ/ETM", "ᆯ/ETM" → "ㄹ/ETM", "ᆷ/ETM" → "ㅁ/ETM"
for token in tokens.iter_mut() {
    if token.pos == "ETM" {
        let normalized = token.surface.replace('ᆫ', "ㄴ")
            .replace('ᆯ', "ㄹ")
            .replace('ᆷ', "ㅁ");
        if normalized != token.surface {
            token.surface = normalized;
        }
    }
}
```

### 2.6 종성 제거 헬퍼 함수

```rust
/// 한글 음절에서 ㄹ 받침을 제거 (할 → 하, 갈 → 가)
fn remove_jongseong_rieul(ch: char) -> Option<char> {
    let code = ch as u32;
    if (0xAC00..=0xD7A3).contains(&code) {
        let jongseong = (code - 0xAC00) % 28;
        if jongseong == 8 { // ㄹ = 8
            let new_code = code - 8;
            char::from_u32(new_code)
        } else { None }
    } else { None }
}

/// 한글 음절에서 ㅂ 받침을 제거 (합 → 하, 갑 → 가)
fn remove_jongseong_bieup(ch: char) -> Option<char> {
    let code = ch as u32;
    if (0xAC00..=0xD7A3).contains(&code) {
        let jongseong = (code - 0xAC00) % 28;
        if jongseong == 17 { // ㅂ = 17
            let new_code = code - 17;
            char::from_u32(new_code)
        } else { None }
    } else { None }
}
```

---

## 3. 파일 변경 요약

| 파일 | 변경 유형 | 라인 변경 |
|------|----------|----------|
| `rust/crates/mecab-ko-core/src/sejong.rs` | 수정 | +140 라인 |
| `rust/crates/mecab-ko-core/tests/accuracy_eval.rs` | 수정 | +8 라인 |
| `PLAN.md` | 수정 | 업데이트 |
| `PROGRESS.md` | 수정 | 업데이트 |
| `.history/2026-03-10_sprint33_59.2_session.md` | 생성 | 새 파일 |

---

## 4. Git 커밋 이력

```
c8c9fca docs: Add Sprint 33 session log (59.2% accuracy)
e6db827 docs: Update progress - 59.2% accuracy
079d643 feat(accuracy): Sprint 33 progress - 59.2% accuracy
5c39c78 feat(accuracy): Sprint 33 progress - 59.1% accuracy
1fcb4f6 feat(accuracy): Sprint 33 progress - 58.7% accuracy
```

---

## 5. 테스트 결과

### 5.1 전체 정확도
```
=== 정확도 평가 결과 ===
테스트 문장: 300
Token Accuracy: 59.2%
Sentence Accuracy: 39.7%
POS Accuracy: 59.2%
Precision: 0.546
Recall: 0.592
F1 Score: 0.568

토큰 통계:
  정답 토큰: 1619
  예측 토큰: 1755
  완전 일치 문장: 119 / 300 (39.7%)
```

### 5.2 품사별 정확도 (최종)
| 품사 | 토큰 수 | 정확도 | Sprint 32 대비 |
|------|---------|--------|----------------|
| NP | 26개 | 88.5% | ±0.0% |
| JX | 15개 | 86.7% | ±0.0% |
| VA | 87개 | 75.9% | +3.5% |
| JKO | 52개 | 75.0% | -1.9% |
| JKB | 42개 | 71.4% | ±0.0% |
| NNG | 357개 | 64.4% | -2.0% |
| **VV** | 226개 | **58.0%** | **+5.8%** |
| VX | 35개 | 57.1% | ±0.0% |
| EP | 89개 | 56.2% | -1.1% |
| JKS | 52개 | 55.8% | -5.7% |
| **EF** | 258개 | **53.9%** | **+2.7%** |
| XSV | 64개 | 51.6% | -1.5% |
| MAG | 31개 | 51.6% | -3.2% |
| **EC** | 117개 | **51.3%** | **+19.7%** |
| **ETM** | 31개 | **48.4%** | **+22.6%** |
| NNB | 23개 | 43.5% | -4.3% |
| ETN | 19개 | 26.3% | +10.5% |
| **XPN** | 12개 | **25.0%** | **+25.0%** |

### 5.3 개별 테스트 결과
| 테스트 | 통과 | 전체 | 통과율 |
|--------|------|------|--------|
| EF 에러 분석 | 21 | 25 | 84.0% |
| ETM 에러 분석 | 9 | 15 | 60.0% |
| EC 에러 분석 | 15 | 16 | 93.8% |
| XPN 에러 분석 | 7 | 9 | 77.8% |
| ETN 에러 분석 | 4 | 10 | 40.0% |

---

## 6. 주요 성과

### 6.1 목표 달성
| 목표 | 시작 | 목표 | 달성 | 상태 |
|------|------|------|------|------|
| EC 정확도 | 31.6% | 50%+ | **51.3%** | ✅ 달성 |
| ETM 정확도 | 25.8% | 40%+ | **48.4%** | ✅ 달성 |
| 전체 정확도 | 56.6% | 60%+ | **59.2%** | ⏳ 0.8% 남음 |

### 6.2 주요 개선 품사
| 품사 | Sprint 32 | Sprint 33 | 개선 |
|------|-----------|-----------|------|
| ETM | 25.8% | **48.4%** | **+22.6%** |
| XPN | 0.0% | **25.0%** | **+25.0%** |
| EC | 31.6% | **51.3%** | **+19.7%** |
| ETN | 15.8% | **26.3%** | **+10.5%** |

---

## 7. 60% 달성을 위한 남은 과제

### 7.1 현재 병목
1. **NNG (357개, 127 에러)**: 동음이의어 문제로 코드 보정 어려움
2. **VV (226개, 95 에러)**: 단음절 활용형 분리 (MeCab 사전 한계)
3. **EF (258개, 119 에러)**: 추가 패턴 필요
4. **토큰 정렬 불일치**: 예측과 정답의 토큰 수/위치 차이

### 7.2 추가 개선 방안
1. **사전 레벨 수정**: MeCab-Ko-Dic 직접 수정
2. **BERT 기반 재순위화**: 문맥 기반 POS 태깅
3. **사용자 사전 확장**: 더 낮은 비용으로 패턴 추가

---

## 8. 기술적 학습 포인트

### 8.1 한글 유니코드 처리
- **자모 정규화**: ᆫ(U+11AB) → ㄴ(U+3134)
- **종성 계산**: `(code - 0xAC00) % 28`
- **종성 제거**: `code - jongseong_index`

### 8.2 Viterbi 알고리즘 한계
- 사용자 사전 비용을 -30000까지 낮춰도 MeCab이 기본 경로 선호
- 단일 토큰 경로가 분리된 경로보다 비용이 낮을 수 있음

### 8.3 세종 코퍼스 vs MeCab-Ko-Dic
- 세종: 어미 분리 (`가/VV 았/EP 다/EF`)
- MeCab: 어미 결합 (`갔다/VV+EP+EF`)
- 변환 과정에서 복잡한 후처리 필요

---

## 9. 세션 통계

| 항목 | 값 |
|------|-----|
| 세션 시작 시간 | 2026-03-10 |
| 시작 정확도 | 58.2% |
| 종료 정확도 | 59.2% |
| 이번 세션 개선 | +1.0% |
| Sprint 32 대비 개선 | +2.6% |
| 새 보정 규칙 수 | 6개 (51~56차) |
| 새 헬퍼 함수 수 | 2개 |
| 수정 파일 수 | 4개 |
| 총 커밋 수 | 5개 |

---

*작성: Claude (PM Auto Mode)*
*프로젝트: MeCab-Ko (한국어 형태소 분석기)*
*Sprint: 33 - EC/ETM 정확도 개선*
