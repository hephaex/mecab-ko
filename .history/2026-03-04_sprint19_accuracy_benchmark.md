# Sprint 19 Accuracy Benchmark Session (2026-03-04)

## 세션 개요
Sprint 19 S19-03 정확도 벤치마크 완료

## 완료된 작업

### S19-03: 정확도 벤치마크 ✅

전체 mecab-ko-dic (816,283 엔트리)로 정확도 측정 수행.

#### 평가 결과

```
=== 정확도 평가 결과 ===
테스트 문장: 160
Token Accuracy: 15.2%
Sentence Accuracy: 8.1%
POS Accuracy: 15.2%
Precision: 0.181
Recall: 0.152
F1 Score: 0.165
```

#### 품사별 정확도 (주요)

| 품사 | 정확도 | 분석 |
|------|--------|------|
| NNG (일반명사) | 48.0% | 가장 높음 - 사전 커버리지 양호 |
| JKB (부사격조사) | 26.5% | 조사류 중간 수준 |
| EC (연결어미) | 7.3% | 어미 분석 어려움 |
| VV (동사) | 1.5% | 매우 낮음 - 활용형 분석 문제 |
| ETM (관형형어미) | 0.7% | 어미 경계 불일치 |

#### 개선 방향

1. **mini-dict 대비 30배 개선**: 0.5% → 15.2%
   - 전체 사전의 효과 확인

2. **명사류 상대적 강점**:
   - NNG 48%, NNP ~40% 추정
   - 사전 기반 매칭이 잘 동작

3. **용언 활용 분석 약점**:
   - VV, VA, EP, EC 등 어미 정확도 낮음
   - 원인: 형태소 경계 불일치
   - 예: "들어가신다" → 정답 "들어가/VV + 시/EP + ㄴ다/EC" vs 분석 "들어가/VV+EC 신다/EP+EC"

4. **평가 데이터 형식 점검 필요**:
   - `data/eval/sample.tsv` 형식 확인
   - 정답 데이터의 토큰화 기준 vs MeCab-Ko 토큰화 기준 차이 분석

## 기술 포인트

### 평가 명령어
```bash
cd rust
cargo run --release -p mecab-ko-cli -- evaluate \
  --dict-dir ../data/dict-output \
  --eval-file ../data/eval/sample.tsv
```

### 정확도 계산 방식
- **Token Accuracy**: 정답 토큰 중 맞힌 비율
- **Sentence Accuracy**: 전체 문장이 완전히 일치하는 비율
- **POS Accuracy**: 표면형+품사 모두 일치하는 비율
- **Precision/Recall/F1**: 예측 토큰 vs 정답 토큰 기준

### 평가 데이터 형식
```
원문\t표면형1/품사1 표면형2/품사2 ...
```

예시:
```
아버지가방에들어가신다	아버지/NNG 가/JKS 방/NNG 에/JKB 들어가/VV 시/EP ㄴ다/EC
```

## 다음 단계

1. **S19-04: 복합명사 분해 개선**
   - 평가 데이터 분석으로 분해 규칙 조정

2. **정확도 개선 방안 연구**
   - 어미 분석 알고리즘 개선
   - 평가 데이터 표준화

3. **v0.3.1 릴리스 준비**
   - 정확도 기준선 문서화
   - 전체 사전 빌드 가이드

## 커밋 이력

```
(pending) docs: complete S19-03 accuracy benchmark with full dictionary
30d7d4f feat(dict): build full mecab-ko-dic (816K entries)
```

## 학습 포인트

1. **전체 사전 효과**: mini-dict 0.5% → full-dict 15.2% (30배 개선)
2. **품사별 정확도 차이**: 명사 48% vs 동사 1.5% - 형태소 경계 분석 방식 차이
3. **평가 기준 표준화 필요**: 정답 데이터와 분석기의 토큰화 기준 일치 필요
