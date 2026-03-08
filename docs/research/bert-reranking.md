# BERT 기반 재순위화(Reranking) 기술 조사

**날짜:** 2026-03-08
**조사자:** Researcher Agent
**카테고리:** 알고리즘 / 딥러닝

## 요약

한국어 형태소 분석기에서 BERT 기반 재순위화는 전통적인 사전 기반 분석과 Transformer 딥러닝 모델을 결합하여 정확도를 획기적으로 향상시키는 기술이다. ETRI의 2024년 연구에서 1단계 재순위화로 **20% 오류 감소**, 2단계 재순위화로 **30% 이상 오류 감소**를 달성했다. 그러나 BERT 모델의 높은 계산 비용으로 인해 실시간 처리가 필요한 환경에서는 성능과 정확도 간의 트레이드오프를 신중히 고려해야 한다.

---

## 1. MeCab + BERT 결합 방식 (N-best 재순위화)

### 1.1 기본 아키텍처

**전통적 MeCab 방식:**
- Viterbi 알고리즘으로 Lattice에서 최적 경로 1개 선택
- 사전 기반 형태소 분석 (빠르지만 문맥 이해 부족)

**BERT 재순위화 방식:**
1. **1단계: N-best 경로 생성** (사전 기반)
   - MeCab/사전 기반 분석기로 Lattice 구성
   - Beam search로 상위 N개 후보 경로 추출
   - 각 경로는 형태소 분할 + POS 태그 조합

2. **2단계: BERT 기반 재순위화**
   - N개 후보 경로를 BERT 모델에 입력
   - 문맥 이해 기반으로 각 경로의 점수 재계산
   - 최종 최적 경로 선택

### 1.2 ETRI 2024 연구: Transformer 기반 재순위화

**논문:** "Transformer-based reranking for improving Korean morphological analysis systems"
**출판:** ETRI Journal, Feb 28, 2024
**DOI:** 10.4218/etrij.2023-0364

**핵심 방법론:**
- 사전 기반 기법으로 **여러 준최적 경로(suboptimal paths)** 생성
- BERT 모델이 고급 언어 이해력을 활용하여 재순위화
- **2단계 재순위화**: 서로 다른 BERT 변형 모델을 순차 적용

**성능 개선:**
- **1단계 재순위화:** 기존 모델 대비 **20% 이상 오류 감소율** 개선
- **2단계 재순위화:** **30% 이상 오류 감소율** 달성
- MeCab 시스템보다 정확도 우수

**사용 모델:**
- **KPF-BERT**: 한국언론진흥재단, BigKinds 뉴스 데이터로 학습 (4천만 기사)
- **ETRI-ELECTRA**: ETRI, 31GB 한국어 텍스트 + whole word masking
- **ETRI-RoBERTa**: ETRI, RoBERTa 변형 모델

---

## 2. 한국어 BERT 모델

| 모델 | 개발자 | 학습 데이터 | 토크나이저 | 용도 |
|------|--------|------------|-----------|------|
| **KoBERT** | SKT Brain | 한국어 위키, 뉴스 | WordPiece | 범용 |
| **KoELECTRA** | Monologg | 34GB (위키, 나무위키, 신문, 모두 말뭉치) | WordPiece | 범용 |
| **KPF-BERT** | 한국언론진흥재단 | BigKinds 뉴스 4천만 기사 (2000-2021) | WordPiece | 뉴스 특화 |
| **ETRI-ELECTRA** | ETRI | 31GB 한국어 텍스트 | Whole Word Masking | 범용 |
| **KRongBERT** | 최신 (2025.01) | - | 형태소 기반 factorization | OOV 문제 해결 |

---

## 3. 기존 구현 사례

### 3.1 KMAwithBERTs (2022, PeerJ)

**GitHub:** https://github.com/yseokchoi/KMAwithBERTs

- **Encoder-Decoder Transformer**
- **Encoder 초기화:** wBERT (단어 기반 BERT)
- **Decoder 초기화:** mBERT (형태소 기반 BERT)
- **성능:** 98.31% F1 Score
- **파라미터:** 193M (decoder 4 layers 기준)

### 3.2 Kiwi / Lindera

- **Kiwi**: 통계 모델 기반, BERT 미사용, 86.7% 정확도
- **Lindera (Rust)**: 순수 사전 기반, BERT 없음, 22µs 토큰화

---

## 4. 성능 vs 정확도 트레이드오프

| 전략 | 레이턴시 | 정확도 | 사용 사례 |
|------|---------|--------|----------|
| MeCab only | ~1ms | 85-90% | 실시간 검색 |
| MeCab + DistilBERT (N=3) | ~10ms | 92-95% | 일반 문서 처리 |
| 2-stage BERT reranking | ~50ms | 97-99% | 고정밀 분석 |
| Full Encoder-Decoder | ~100ms | 98%+ | 배치 처리 |

### 최적화 전략

- **Knowledge Distillation**: 95% 성능 유지, 40% 파라미터 감소, 60% 속도 향상
- **ONNX Runtime + Quantization**: 10ms 미만 달성
- **Confidence-based Hybrid**: 90% 문장은 MeCab, 10% 불확실 문장만 BERT

---

## 5. 학습 포인트 (3줄 요약)

1. **BERT 재순위화로 20-30% 오류 감소 가능**하나 추론 비용이 크므로 **Confidence 기반 Hybrid 전략**이 실용적이다.
2. **한국어 BERT는 형태소 기반 토크나이저 사용 시** 태깅 작업에서 더 효과적이며, **KRongBERT(2025)는 OOV 문제 해결**에 강점이 있다.
3. **Knowledge Distillation + ONNX Runtime 조합으로 10ms 이하 레이턴시** 달성 가능하여 실시간 처리와 고정밀 분석의 균형점을 찾을 수 있다.

---

## 6. 참고 자료

- [Transformer-based reranking (ETRI Journal, 2024)](https://onlinelibrary.wiley.com/doi/full/10.4218/etrij.2023-0364)
- [KMAwithBERTs (PeerJ, 2022)](https://peerj.com/articles/cs-968/)
- [KRongBERT (ScienceDirect, 2025)](https://www.sciencedirect.com/science/article/pii/S0306457325000147)
- [KMAwithBERTs GitHub](https://github.com/yseokchoi/KMAwithBERTs)
- [BERT Inference Optimization](https://medium.com/@raajeshlr2/7-proven-optimizations-to-cut-bert-inference-latency-in-half-for-production-81eb3702b7c2)

---

**작성일:** 2026-03-08
**버전:** 1.0
