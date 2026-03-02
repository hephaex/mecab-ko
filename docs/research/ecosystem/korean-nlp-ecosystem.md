# 한국어 NLP 생태계 조사 보고서

**날짜**: 2026-02-23
**카테고리**: ecosystem

## 요약 (3줄)
1. mecab-ko-dic(2018 중단)이 여전히 사실상 표준 - Nori, Lindera, KoNLPy 모두 의존
2. Kiwi가 정확도 최고(SBG 86.7%), Lindera v2.2.0이 Rust 구현 최신(N-Best, 미등록어 추정)
3. 세종 품사 태그셋이 표준, mecab-ko-dic의 12필드 CSV가 호환성 핵심

## 1. mecab-ko-dic

### 현황
- **최신 버전**: 2.1.1-20180720 (6년+ 미갱신)
- **저장소**: https://bitbucket.org/eunjeon/mecab-ko-dic
- **GitHub 미러**: LuminosoInsight, Pusnow, deepsearch-hq
- **엔진**: mecab-ko 0.996-ko-0.9.2

### 12필드 CSV 포맷
```
표층형,좌문맥ID,우문맥ID,비용,품사,의미부류,종성유무,읽기,타입,첫번째품사,마지막품사,분해식
```

| 필드 | 설명 | 예시 |
|------|------|------|
| surface | 표면형 | 도서관 |
| left_id / right_id | 문맥 ID | 1781, 3535 |
| cost | 단어 비용 | 2110 |
| pos | 품사 (세종) | NNG, NNP, VV |
| semantic | 의미 부류 | * |
| has_jongseong | 종성 유무 | T/F |
| reading | 읽기 | 도서관 |
| type | 타입 | *, Compound, Inflect, Preanalysis |
| expression | 분해식 | 도서/NNG/*+관/NNG/* |

### 세종 품사 태그 (주요)
**체언**: NNG(일반명사), NNP(고유명사), NNB(의존명사), NNBC(단위명사), NR(수사), NP(대명사)
**용언**: VV(동사), VA(형용사), VX(보조용언), VCP(긍정지정사), VCN(부정지정사)
**조사**: JKS(주격), JKO(목적격), JKG(관형격), JC(접속), JX(보조사)
**어미**: EP(선어말), EF(종결), EC(연결), ETN(명사형전성), ETM(관형형전성)
**기호**: SF(마침표), SS(따옴표), SH(한자), SL(외국어), SN(숫자)

### 일본어 ipadic과 차이점
- 종성유무(has_jongseong) 필드 - 조사 결합 규칙용
- 타입(Compound/Inflect/Preanalysis) - 복합어 분해
- 공백 페널티 비용 - 한국어 공백의 의미적 가중치
- 세종 코퍼스 기반 학습

## 2. Kiwi (최고 정확도)

### 현황
- **최신 버전**: v0.22.2 (2025-12-15) - 활발히 유지보수
- **GitHub**: https://github.com/bab2min/Kiwi (663 stars)
- **언어**: C++ (C++17, v0.21.0+)
- **Python**: kiwipiepy

### 아키텍처
- **2단계 파이프라인**: 형태소 분할 → 품사 태깅
- **KNLM**: 3.5-gram POS-형태소 언어 모델 (Kneser-Ney)
- **SBG**: Skip-Bigram 모델 - 원거리 의존성 해결 (77.4% vs KNLM 58.1%)
- **CoNg**: Contextual N-gram 임베딩 (v0.21.0+)
- **한글 표현**: 초성+중성 결합, 종성 분리 (427자 세트)

### 정확도
| 영역 | KNLM | SBG |
|------|------|-----|
| 문어체 | 93.32% | 더 높음 |
| 웹텍스트 | 86.49% | ~87% |
| 오탈자 | 75.68% | - |
| 원거리 의존성 | 58.1% | 77.4% |

### 최근 기능 (2024-2025)
- v0.20.0: 사이시옷 Z_SIOT 태그
- v0.21.0: CoNg 모델, C++17
- v0.22.0: 방언 분석, Android 바인딩
- v0.22.2: 미등록어 버그 수정

## 3. Nori (Elasticsearch/Lucene)

### 현황
- **소속**: Apache Lucene 내장
- **개발자**: Jim Ferenczi (Elastic)
- **문서**: https://www.elastic.co/docs/reference/elasticsearch/plugins/analysis-nori
- **JIRA**: LUCENE-8231

### 아키텍처
- Kuromoji(일본어) 아키텍처를 한국어에 적응
- **FST**: 811,757 용어를 5.4MB FST로 인코딩
- **연접 비용 행렬**: 10.26M 셀, 139MB → 12MB (가변길이 인코딩)
- **Rolling Viterbi**: 단일 패스 전방 소비, 즉시 가지치기
- N-best 제거 (Kuromoji 대비 단순화)

### Decompound 모드
- `none`: 복합어 분해 안 함
- `discard` (기본): 분해 후 원본 제거
- `mixed`: 분해 + 원본 유지 (검색에 유리)

### 성능
- 한국어 위키 413,985 문서: 3,000+ docs/sec
- MAP: Standard/CJK 대비 15-25% 향상
- 512MB 힙에서 동작

## 4. Lindera (Rust 구현)

### 현황
- **최신 버전**: v2.2.0 (2026-02-10) - 매우 활발
- **GitHub**: https://github.com/lindera/lindera (603 stars, 110 releases)
- **라이센스**: MIT

### 크레이트 구조
| 크레이트 | 역할 |
|----------|------|
| lindera-core | 사전 구조 + Viterbi |
| lindera-dictionary | 사전 빌드/로딩 |
| lindera-ko-dic | **한국어 사전** (mecab-ko-dic) |
| lindera-ko-dic-builder | 한국어 사전 빌더 |
| lindera-cli | CLI |
| lindera-python / lindera-wasm | 바인딩 |

### 핵심 기술
- FST (daachorse 지원, v2.1.0+)
- rkyv 0.8 제로카피 (bincode에서 마이그레이션, v1.5.0)
- Forward-DP Backward-A* N-Best (v2.2.0)
- 미등록어 POS 추정 (v2.2.0)
- Lattice 재사용 API (v1.5.0)
- Tantivy 검색엔진 통합

## 5. 기타 분석기

| 분석기 | 언어 | 상태 | 로딩 | 100K자 |
|--------|------|------|------|--------|
| MeCab-ko | C++ | 중단(2018) | 0.0007s | 0.28s |
| Kiwi | C++ | **활발** | - | ~1.9ms/줄 |
| Okt | Scala | 중단(2018) | 1.49s | 2.47s |
| KOMORAN | Java | 중단(2020) | 5.49s | 25.6s |
| Hannanum | Java | 레거시 | 0.66s | 8.83s |
| KKMA | Java | 레거시 | 5.70s | 35.7s |

## 학습 포인트
1. **Lindera v2.2.0이 가장 가까운 레퍼런스** - Rust + ko-dic + Viterbi + rkyv, 2026년 최신
2. **Nori의 압축 기법**이 직접 적용 가능 - FST 5.4MB, 연접 행렬 139→12MB
3. **Kiwi의 SBG/CoNg**는 Viterbi 이후의 정확도 향상 방향 - Phase 2 이후 검토

## 참고 자료
- [Kiwi 학술 논문](https://accesson.kr/kjdh/v.1/1/109/43508) - SBG 알고리즘 상세
- [Nori 공식 블로그](https://www.elastic.co/blog/nori-the-official-elasticsearch-plugin-for-korean-language-analysis) - 아키텍처/성능
- [KoNLPy 비교](https://konlpy.org/en/latest/morph/) - 한국어 분석기 벤치마크

## 프로젝트 적용 방안
- `mecab-ko-dict`: mecab-ko-dic 12필드 CSV 파서 + 세종 태그셋 구현
- `mecab-ko-core`: Lindera 아키텍처 참고하여 Viterbi 구현, Nori 압축 기법 적용
- `mecab-ko-elasticsearch`: Nori 호환 API (decompound 모드, POS 필터)
- 향후: Kiwi SBG 모델 참고하여 정확도 향상
