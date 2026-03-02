# MeCab 알고리즘 내부 구조 조사 보고서

**날짜**: 2026-02-23
**카테고리**: algorithms

## 요약 (3줄)
1. MeCab은 DA Trie(사전 조회) + Viterbi(최적 경로) + CRF(비용 학습) 3단 구조
2. 사전 포맷은 CSV(원본) → 바이너리(sys.dic) 변환, 핵심 파일: sys.dic, matrix.def, char.def, unk.def
3. 한국어 특화: mecab-ko-dic은 12필드 CSV(종성유무, 복합어 분해식 포함), 공백 페널티 비용 추가

## 핵심 구조체 (mecab.h)

### mecab_node_t
```c
struct mecab_node_t {
    struct mecab_node_t *prev, *next;     // 이전/다음 노드
    struct mecab_node_t *enext, *bnext;   // 같은 위치 시작/종료 노드
    struct mecab_path_t *rpath, *lpath;   // 좌/우 경로
    const char *surface;                   // 표층형 문자열
    const char *feature;                   // 자질 문자열
    unsigned short length, rlength;        // 표층형 길이, 공백포함 길이
    unsigned short rcAttr, lcAttr;         // 우/좌 문맥 ID
    unsigned short posid;                  // 품사 ID
    unsigned char char_type, stat;         // 문자 유형, 상태
    short wcost;                           // 단어 비용
    long cost;                             // BOS부터 누적 최적 비용
    float alpha, beta, prob;               // 전방/후방/주변 확률
};
```

### mecab_path_t
```c
struct mecab_path_t {
    struct mecab_node_t *rnode, *lnode;   // 우/좌 노드
    struct mecab_path_t *rnext, *lnext;  // 다음 우/좌 경로
    int cost;                             // 연접 비용
    float prob;                           // 주변 확률
};
```

## 사전 파일 포맷

### CSV 사전 엔트리 형식
```
표층형,좌문맥ID,우문맥ID,비용,자질1,자질2,...
```
- 처음 4필드 필수, 자질은 가변
- 활용어는 미리 전개 필요 (MeCab은 활용 처리 안 함)

### mecab-ko-dic 12필드 CSV
```
표층형,좌문맥ID,우문맥ID,비용,품사,의미부류,종성유무,읽기,타입,첫번째품사,마지막품사,분해식
```
예: `트위치,,,,NNP,*,F,트위치,*,*,*,*`
- 종성유무: T(있음)/F(없음) - 조사 결합 규칙에 필요
- 타입: Compound(복합어), Inflect(활용형), Preanalysis(선분석)

### char.def (문자 카테고리)
```
카테고리명  INVOKE(0|1)  GROUP(0|1)  LENGTH(n)
```
- INVOKE: 0=사전에 없을 때만, 1=항상 미등록어 처리
- GROUP: 1=같은 카테고리 문자를 그룹핑
- LENGTH: 미등록어 후보 최대 길이
- DEFAULT, SPACE 카테고리 필수

### unk.def (미등록어)
```
카테고리,0,0,0,자질1,자질2,...
```
- 각 문자 카테고리별 복수 자질 정의 가능

### matrix.def (연접 비용 행렬)
- 2차원 배열: `matrix[right_id][left_id] = cost`
- mecab-ko-dic: ~2,000 x 2,000 크기

### rewrite.def (자질 재작성)
- `[unigram rewrite]`, `[left rewrite]`, `[right rewrite]` 3섹션
- 패턴 매칭: `*`(전체), `(A|B)`(대안), 리터럴
- 매크로: `$1 $2 $3...` (CSV 요소 참조)

### feature.def (CRF 자질 템플릿)
- `%F[n]`: 유니그램 자질, `%t`: 문자 유형
- `%L[n]`/`%R[n]`: 좌/우 문맥 자질
- UNIGRAM, BIGRAM 라벨

## Viterbi 알고리즘 상세

### 전방 패스 (Forward)
```
for each position i in input:
    for each node n ending at position i:
        for each predecessor p:
            new_cost = p.cost + connection_cost[p.rcAttr][n.lcAttr] + n.wcost
            if new_cost < n.cost:
                n.cost = new_cost
                n.prev = p
```

### 역추적 (Backward)
```
node = EOS
path = []
while node != BOS:
    path.prepend(node)
    node = node.prev
```

### 비용 체계
- **단어 비용 (wcost)**: 형태소 출현 확률의 음의 로그. 낮을수록 자주 나타남
- **연접 비용**: 두 형태소 간 전이 확률. matrix.def에서 조회
- **총 비용**: 모든 단어 비용 + 연접 비용의 합이 최소인 경로가 최적

## Nori (Lucene) 구현

- Kuromoji(일본어) 아키텍처를 한국어에 적응
- FST로 사전 조회 (DA Trie 대신)
- 바이너리 사전: ~28MB (sys.dic 기준)
- 연접 비용 행렬: ConnectionCosts.dat (11.2MB)
- 공백 처리: 공백 후 품사별 페널티 비용 부과
- N-best 제거, 긴 토큰 분해 제거 (단순화)
- MAP 기준 Standard/CJK 분석기 대비 15-25% 향상

## 학습 포인트
1. **래티스 구축의 핵심은 common-prefix-search** - 입력의 각 위치에서 DA Trie로 가능한 모든 사전 엔트리를 탐색
2. **한국어 공백은 의미적 가중치** - 일본어와 달리 공백 존재 여부가 분석에 영향, 품사별 페널티 필요
3. **char.def의 INVOKE/GROUP/LENGTH 3값**이 미등록어 처리 전략을 결정

## 참고 자료
- [MeCab 공식 문서 (일본어)](https://taku910.github.io/mecab/) - 알고리즘 상세
- [MeCab 사전 학습 문서](https://taku910.github.io/mecab/learn.html) - CRF 비용 추정
- [How Japanese Tokenizers Work](https://medium.com/data-science/how-japanese-tokenizers-work-87ab6b256984) - Viterbi 시각적 설명

## 프로젝트 적용 방안
- `mecab-ko-dict`: char.def/unk.def 파서 구현 필요, matrix.def → 2D i16 배열
- `mecab-ko-core`: Lattice 노드는 mecab_node_t 간소화 버전, Viterbi 전방패스+역추적
- `mecab-ko-dict-builder`: CSV → 바이너리 변환기, CRF 비용은 기존값 재활용
