# Sprint 26 정확도 35% 목표 달성 세션 로그

## 날짜: 2026-03-05

## 세션 개요
Sprint 26의 정확도 35% 목표 달성. 34.2% → 35.2%로 최종 개선.

## 완료한 작업

### 1. 축약형 동사 분리 로직 추가
- **문제**: "했어요", "갔어요" 등 축약형이 세종 변환에서 분리되지 않음
- **원인**: EndingRule에서 "았어요"가 "했어요"와 매칭되지 않음 (축약형)
- **해결**: `try_split_contracted` 함수 추가
  - 축약형 어간 인식: 했/갔/왔/봤/샀/잤/됐
  - 원래 어간과 선어말어미로 복원: 했 → 하+았
  - 종결어미 패턴 매칭: 어요, 어, 다, 지, 니, 나, 습니다, 습니까
- **결과**: EP 15.0% → 21.2% (+6.2%p)

### 2. 축약형 과거 시제 사용자 사전 추가
- 했습니다, 갔습니다, 왔습니다, 봤습니다, 됐습니다 (VV+EP+EF)
- 비용 -10000으로 높은 우선순위 설정
- **결과**: XSV 10.9% → 17.2% (+6.3%p)

## 정확도 변화

| 지표 | Sprint 25 종료 | Sprint 26 시작 | Sprint 26 최종 | 개선 |
|------|----------------|----------------|----------------|------|
| Token Accuracy | 30.4% | 34.2% | **35.2%** | +4.8%p |
| Sentence Accuracy | 12.3% | 17.7% | **18.7%** | +6.4%p |
| 완전 일치 문장 | 37 | 53 | **56** | +19 |

### 품사별 최종 정확도
| 품사 | Sprint 25 | Sprint 26 | 개선 |
|------|-----------|-----------|------|
| NP | 53.8% | 65.4% | +11.6%p |
| VX | 8.6% | 20.0% | +11.4%p |
| XSV | 9.4% | 17.2% | +7.8%p |
| EP | 13.8% | 21.2% | +7.4%p |
| EF | 25.1% | 32.1% | +7.0%p |
| JKO | 40.4% | 46.2% | +5.8%p |
| JKB | 34.9% | 39.5% | +4.6%p |
| VA | 58.6% | 62.1% | +3.5%p |
| EC | 23.9% | 27.4% | +3.5%p |
| NNG | 46.5% | 49.9% | +3.4%p |
| VV | 27.0% | 30.1% | +3.1%p |
| JKS | 19.2% | 21.2% | +2.0%p |

## Git 커밋
- `6b12cef` - feat(accuracy): Achieve 35.2% token accuracy - Sprint 26 goal complete

## 파일 변경 요약

| 파일 | 변경 |
|------|------|
| rust/crates/mecab-ko-core/src/sejong.rs | try_split_contracted 함수 추가 |
| data/user-dict/verb-inflections.csv | 축약형 과거 시제 패턴 8개 추가 (239 → 247) |
| PROGRESS.md | Sprint 26 완료 상태 업데이트 |

## 기술 학습 포인트

1. **축약형 처리의 복잡성**: 한국어 동사 축약(하+았→했)은 단순 suffix 매칭으로 해결 불가
2. **다중 분리 전략**: EndingRule 실패 시 축약형 특수 처리 fallback 적용
3. **사용자 사전 활용**: 비용 조정으로 Viterbi 경로 우선순위 제어
4. **점진적 개선**: 매 변경 후 정확도 측정으로 회귀 방지

## 다음 스프린트에서 할 일

1. **Sprint 27 계획**:
   - 정확도 40% 목표
   - 공백 연접 문제 해결 (SpacePenalty 재검토)
   - JKS 정확도 개선 (21.2% → 목표 30%)
   - MAG, NNB 정확도 개선

2. **npm 배포**:
   - npm 토큰 확보 시 mecab-ko-wasm v0.4.0 배포

## 핵심 코드 추가

```rust
/// 축약형 동사 분리 시도
/// 예: 했어요 → 하 + 았 + 어요, 갔어요 → 가 + 았 + 어요
fn try_split_contracted(&self, surface: &str, pos: &str) -> Option<Vec<(String, String)>> {
    let contracted_stems = [
        ("했", "하", "았"),
        ("갔", "가", "았"),
        ("왔", "오", "았"),
        ("봤", "보", "았"),
        ("샀", "사", "았"),
        ("잤", "자", "았"),
        ("됐", "되", "었"),
    ];

    let chars: Vec<char> = surface.chars().collect();
    let first_char = chars[0].to_string();

    for (contracted, stem, prefinal) in &contracted_stems {
        if first_char == *contracted {
            let ending: String = chars[1..].iter().collect();
            let ef_patterns = ["어요", "어", "다", "지", "니", "나", "습니다", "습니까"];
            for ef in &ef_patterns {
                if ending == *ef || ending.ends_with(ef) {
                    return Some(vec![
                        (stem.to_string(), tags[0].clone()),
                        (prefinal.to_string(), tags[1].clone()),
                        (ending.clone(), tags[2].clone()),
                    ]);
                }
            }
        }
    }
    None
}
```
