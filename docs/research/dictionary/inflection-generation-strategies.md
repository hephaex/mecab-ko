# MeCab-Ko 사전 개선 전략

## 조사 날짜: 2026-03-08

## 1. Inflect.csv 구조

### 파일 형식 (12컬럼)
```csv
표면형,left_id,right_id,cost,품사,의미,종성,읽기,타입,시작품사,끝품사,분석결과
가,489,3554,8084,JKS+NP,*,F,가,Inflect,JKS,NP,가/JKS/*+누구/NP/*
가,2420,3,2747,VV+EC,*,F,가,Inflect,VV,EC,가/VV/*+아/EC/*
```

### 주요 필드
- **표면형**: 활용된 형태
- **cost**: 낮을수록 높은 우선순위 (-30000 ~ +8000)
- **품사**: 복합 품사 태그 (VV+ETM, VA+EC 등)
- **종성**: T (받침 있음), F (받침 없음)
- **분석결과**: 형태소 분해 (stem/POS/*+ending/POS/*)

### 비용 계층
| 우선순위 | 비용 범위 | 용도 |
|---------|----------|------|
| Critical | -30000 | 피동/사동 동사 |
| High | -20000 | 복합명사 |
| Common | -15000 | 고빈도 패턴 |
| Standard | -10000 ~ -5000 | 일반 활용 |
| Normal | -500 | 규칙 활용 |

## 2. 불규칙 동사 6종

### ㄷ 불규칙 (ㄷ→ㄹ)
```
듣다 + 어요 → 들어요 (not 듣어요)
걷다 + 어요 → 걸어요
```

### ㅂ 불규칙 (ㅂ→우)
```
고맙다 + 어요 → 고마워요 (not 고맙어요)
가깝다 + 어요 → 가까워요
```

### ㄹ 불규칙 (ㄹ 탈락)
```
살다 + ㄴ → 사네요 (not 살네요)
만들다 + ㅂ니다 → 만듭니다
```

### 르 불규칙 (르→라/러)
```
모르다 + 아요 → 몰라요
부르다 + 어요 → 불러요
```

### ㅅ 불규칙 (ㅅ 탈락)
```
낫다 + 아요 → 나아요 (not 낫아요)
긋다 + 어요 → 그어요
```

### ㅎ 불규칙 (ㅎ 탈락)
```
그렇다 + 아요 → 그래요
빨갛다 + 아요 → 빨개요
```

## 3. 주요 어미 목록

### ETM (관형형어미)
```
-는 (현재): 가는, 먹는, 보는
-ㄴ/은 (과거/완료): 간, 먹은, 본
-ㄹ/을 (미래/추정): 갈, 먹을, 볼
```

### EC (연결어미)
```
-고: 가고, 먹고, 보고
-아서/어서: 가서, 먹어서, 봐서
-면/으면: 가면, 먹으면, 보면
-니까/으니까: 가니까, 먹으니까
-지만: 가지만, 먹지만
```

### EF (종결어미)
```
-다/습니다: 간다, 갑니다
-아요/어요: 가요, 먹어요
-ㄹ게요/을게요: 갈게요, 먹을게요
```

## 4. 모음조화 규칙

### 양성모음 (아/오) → 아
```
가다 + 아요 → 가요
오다 + 아요 → 와요
```

### 음성모음 (나머지) → 어
```
먹다 + 어요 → 먹어요
보다 + 어요 → 봐요
```

### 하다 특수 규칙
```
하다 + 여요 → 해요
공부하다 → 공부해요
```

## 5. 구현 계획

### Phase 1: 활용형 생성기
```rust
// rust/crates/mecab-ko-dict-builder/src/inflect_gen.rs
pub fn generate_inflections(base: &str, pos: PosTag) -> Vec<InflectEntry> {
    let verb_type = detect_irregular_type(base);
    let endings = get_endings_for_pos(pos);

    endings.iter()
        .map(|ending| apply_conjugation(base, ending, verb_type))
        .collect()
}
```

### Phase 2: 고빈도 동사 선정
```
상위 500 동사 (Sejong 코퍼스 빈도 기준):
하다, 있다, 되다, 가다, 오다, 보다, 알다, 같다, 주다, 받다...
```

### Phase 3: 품사별 확장
1. ETM: 500 동사 × 3 어미 = 1,500 엔트리
2. EC: 500 동사 × 5 어미 = 2,500 엔트리
3. EF: 500 동사 × 6 어미 = 3,000 엔트리

## 6. 참고 자료

- [Korean Conjugation Guide](https://www.90daykorean.com/korean-conjugation/)
- [Korean Irregular Verbs](https://ltl-korea.com/grammar-bank/irregular-verbs/)
- [MeCab User Dictionary Guide](https://medium.com/data-science/mecab-usage-and-add-user-dictionary-to-mecab-9ee58966fc6)
- [Transformer-based Reranking (ETRI)](https://onlinelibrary.wiley.com/doi/full/10.4218/etrij.2023-0364)
- [Verbix Korean Conjugator](https://www.verbix.com/languages/korean)
