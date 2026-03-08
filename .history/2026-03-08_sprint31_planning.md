# Session Log: Sprint 31 계획 수립

## 날짜: 2026-03-08

## 세션 개요
Sprint 30 완료 후 Sprint 31 MeCab 사전 개선 계획 수립

## 주요 작업

### 1. Sprint 30 완료 정리
- **최종 정확도**: Token 52.9%, Sentence 36.0%
- **npm v0.4.0**: 배포 완료
- **블로커**: MeCab 사전 한계로 코드 레벨 추가 개선 불가

### 2. MeCab 사전 개선 리서치
- Inflect.csv 구조 분석 (12컬럼, 비용 기반 우선순위)
- 불규칙 동사 6종 패턴 정리 (ㄷ/ㅂ/ㄹ/르/ㅅ/ㅎ)
- 주요 어미 목록 정리 (ETM, EC, EF)
- 모음조화 규칙 문서화

### 3. Sprint 31 계획 수립

#### P0 (Critical)
- S31-01: 활용형 자동 생성기 구현 (inflect_gen.rs)
- S31-02: Inflect.csv 확장 (20,000+ 엔트리)

#### P1 (High)
- S31-03: ETM 사전 확장 (25.8% → 45%+)
- S31-04: 불규칙 동사 패턴 완성

#### P2 (Medium)
- S31-05: JKS 사전 확장 (28.8% → 45%+)
- S31-06: EC 사전 확장 (37.6% → 55%+)
- S31-07: 세종 코퍼스 빈도 분석

#### P3 (Low)
- S31-08: PyPI 계정 복구 (응답 대기)
- S31-09: BERT 기반 재순위화 조사

## 기술적 세부사항

### 활용형 생성 전략
```
500 고빈도 동사 × 40 어미 = 20,000 활용형
- ETM: -는, -ㄴ/은, -ㄹ/을
- EC: -고, -아서/어서, -면/으면, -니까/으니까, -지만
- EF: -다/습니다, -아요/어요, -ㄹ게요/을게요
```

### 비용 계층
| 우선순위 | 비용 | 용도 |
|---------|------|------|
| Critical | -30000 | 피동/사동 |
| High | -20000 | 복합명사 |
| Common | -15000 | 고빈도 |
| Standard | -10000 | 일반 |
| Normal | -500 | 규칙 활용 |

## 파일 변경 내역
- `PLAN.md` - Sprint 31 계획 추가
- `PROGRESS.md` - Sprint 31 상태로 업데이트
- `docs/research/dictionary/inflection-generation-strategies.md` - 신규 리서치 문서

## Git 커밋
- `1466d3b` - docs: Add GitHub Discussion link for contributor removal request
- `a0070f4` - docs: Sprint 31 plan - MeCab dictionary improvement

## 다음 단계
1. inflect_gen.rs 모듈 구현 시작
2. 고빈도 동사 500개 목록 준비 (세종 코퍼스)
3. 불규칙 동사 패턴 감지 함수 구현

## 참고 자료
- [Korean Conjugation Guide](https://www.90daykorean.com/korean-conjugation/)
- [Korean Irregular Verbs](https://ltl-korea.com/grammar-bank/irregular-verbs/)
- [Transformer-based Reranking (ETRI)](https://onlinelibrary.wiley.com/doi/full/10.4218/etrij.2023-0364)
