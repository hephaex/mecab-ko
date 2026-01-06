# MeCab-Ko-Dic 사전 포맷 분석 v2.0

> **문서 버전**: 2.1
> **작성일**: 2026-01-05
> **대상**: mecab-ko-dic (https://bitbucket.org/eunjeon/mecab-ko-dic)
> **구현체**: mecab-ko-dict Rust crate

이 문서는 DIC-001 이슈의 산출물로, mecab-ko-dic의 구조와 데이터 포맷을 완전히 분석하고 문서화합니다.
Rust 구현체(`mecab-ko-dict` crate)의 데이터 구조와 API도 함께 설명합니다.

---

## 목차

1. [저장소 구조](#1-저장소-구조)
2. [CSV 사전 파일 구조](#2-csv-사전-파일-구조)
3. [품사 태그 체계](#3-품사-태그-체계)
4. [정의 파일 분석](#4-정의-파일-분석)
5. [바이너리 사전 포맷](#5-바이너리-사전-포맷)
6. [빌드 프로세스](#6-빌드-프로세스)
7. [Rust 구현체 데이터 구조](#7-rust-구현체-데이터-구조)

---

## 1. 저장소 구조

```
mecab-ko-dic/
├── seed/                    # 원본 CSV 사전 및 정의 파일
│   ├── *.csv               # 품사별 단어 사전 (33개 파일)
│   ├── char.def            # 문자 카테고리 정의
│   ├── unk.def             # 미등록어 처리 정의
│   ├── feature.def         # CRF 피처 템플릿
│   ├── rewrite.def         # 피처 재작성 규칙
│   ├── pos-id.def          # 품사 ID 매핑
│   ├── dicrc               # 사전 설정 파일
│   ├── build.sh            # 빌드 스크립트
│   └── corpus/             # 학습용 말뭉치
│       └── eunjeon_corpus.txt
├── final/                  # 최종 배포용 사전
│   ├── Makefile.am         # Automake 설정
│   ├── configure.ac        # Autoconf 설정
│   ├── autogen.sh          # 자동 빌드 스크립트
│   ├── tools/              # 유틸리티 스크립트
│   │   └── add-userdic.sh  # 사용자 사전 추가
│   └── user-dic/           # 사용자 사전 CSV
└── utils/                  # Python 유틸리티
```

---

## 2. CSV 사전 파일 구조

### 2.1 파일 목록 (총 33개)

| 카테고리 | 파일 | 품사 | 크기 |
|----------|------|------|------|
| **명사** | NNG.csv | 일반 명사 | 12.4 MB |
| | NNP.csv | 고유 명사 | 173 KB |
| | NNB.csv | 의존 명사 | 4.4 KB |
| | NNBC.csv | 단위 의존 명사 | 26 KB |
| | NP.csv | 대명사 | 12 KB |
| | NR.csv | 수사 | 18 KB |
| **용언** | VV.csv | 동사 | 290 KB |
| | VA.csv | 형용사 | 94 KB |
| | VX.csv | 보조 용언 | 4 KB |
| | VCP.csv | 긍정 지정사 (이다) | 288 B |
| | VCN.csv | 부정 지정사 (아니다) | 258 B |
| **수식언** | MAG.csv | 일반 부사 | 711 KB |
| | MAJ.csv | 접속 부사 | 10 KB |
| | MM.csv | 관형사 | 20 KB |
| **조사** | J.csv | 조사 (JKS, JKO 등) | 14 KB |
| **어미** | EC.csv | 연결 어미 | 96 KB |
| | EF.csv | 종결 어미 | 70 KB |
| | EP.csv | 선어말 어미 | 1.6 KB |
| | ETM.csv | 관형형 전성 어미 | 5 KB |
| | ETN.csv | 명사형 전성 어미 | 456 B |
| **접사/어근** | XPN.csv | 접두사 | 2.4 KB |
| | XSN.csv | 명사파생 접미사 | 3.8 KB |
| | XSA.csv | 형용사파생 접미사 | 642 B |
| | XSV.csv | 동사파생 접미사 | 738 B |
| | XR.csv | 어근 | 129 KB |
| **기타** | IC.csv | 감탄사 | 50 KB |
| | Inflect.csv | 활용형 | 3.6 MB |
| | Group.csv | 복합어/그룹 | 228 KB |
| | Hanja.csv | 한자어 | 4.6 MB |
| | Foreign.csv | 외래어 | 530 KB |
| | Wikipedia.csv | 위키피디아 | 1.8 MB |
| | CoinedWord.csv | 신조어 | 5.7 KB |
| | NorthKorea.csv | 북한어 | 108 B |

### 2.2 CSV 컬럼 구조 (12 컬럼, TSV 형식)

```
표층형	좌문맥ID	우문맥ID	비용	품사	의미분류	종성	읽기	타입	첫품사	끝품사	분석결과
```

| # | 필드명 | 설명 | 예시 |
|---|--------|------|------|
| 1 | surface | 표층형 (실제 단어) | `가건물` |
| 2 | left_id | 좌측 문맥 ID | `0` |
| 3 | right_id | 우측 문맥 ID | `0` |
| 4 | cost | 단어 비용 | `0` |
| 5 | pos | 품사 태그 | `NNG` |
| 6 | semantic_class | 의미 분류 | `*` |
| 7 | jongseong | 종성 유무 (T/F) | `T` |
| 8 | reading | 읽기/기본형 | `가건물` |
| 9 | type | 타입 | `Compound`, `Inflect`, `*` |
| 10 | first_pos | 첫 형태소 품사 | `*` |
| 11 | last_pos | 끝 형태소 품사 | `*` |
| 12 | expression | 형태소 분해 | `가/NNG/*+건물/NNG/*` |

### 2.3 종성(받침) 플래그

7번 컬럼의 T/F 값은 한국어 조사 연결에 중요:
- `T`: 종성 있음 (예: 책, 강) → "은", "을" 연결
- `F`: 종성 없음 (예: 나, 사과) → "는", "를" 연결

### 2.4 복합어/활용형 표현

12번 컬럼의 형태소 분해 형식:
```
형태소/품사/속성+형태소/품사/속성+...
```

**예시**:
```csv
# 복합어 (Compound)
세종시,0,0,0,NNP,지명,F,세종시,Compound,*,*,세종/NNP/지명+시/NNG/*

# 활용형 (Inflect)
위한,0,0,0,VV+ETM,*,T,위한,Inflect,VV,ETM,위하/VV/*+ᆫ/ETM/*
입니다,0,0,0,VCP+EF,*,F,입니다,Inflect,VCP,EF,이/VCP/*+ᄇ니다/EF/*
```

---

## 3. 품사 태그 체계

### 3.1 체언 (명사류)

| 태그 | 명칭 | 설명 | 예시 |
|------|------|------|------|
| NNG | 일반 명사 | General Noun | 사과, 컴퓨터 |
| NNP | 고유 명사 | Proper Noun | 서울, 삼성 |
| NNB | 의존 명사 | Dependent Noun | 것, 수, 바 |
| NNBC | 단위 의존 명사 | Counter Noun | 개, 명, 원 |
| NP | 대명사 | Pronoun | 나, 너, 그것 |
| NR | 수사 | Numeral | 하나, 둘, 첫째 |

### 3.2 용언 (동사/형용사류)

| 태그 | 명칭 | 설명 | 예시 |
|------|------|------|------|
| VV | 동사 | Verb | 가다, 먹다 |
| VA | 형용사 | Adjective | 예쁘다, 크다 |
| VX | 보조 용언 | Auxiliary Verb | 있다, 하다 |
| VCP | 긍정 지정사 | Positive Copula | 이다 |
| VCN | 부정 지정사 | Negative Copula | 아니다 |

### 3.3 수식언

| 태그 | 명칭 | 설명 | 예시 |
|------|------|------|------|
| MM | 관형사 | Determiner | 이, 그, 새 |
| MAG | 일반 부사 | Adverb | 매우, 아주 |
| MAJ | 접속 부사 | Conjunctive Adverb | 그러나, 그리고 |

### 3.4 관계언 (조사)

| 태그 | 명칭 | 설명 | 예시 |
|------|------|------|------|
| JKS | 주격 조사 | Nominative | 이/가 |
| JKC | 보격 조사 | Complementizer | 이/가 |
| JKG | 관형격 조사 | Genitive | 의 |
| JKO | 목적격 조사 | Accusative | 을/를 |
| JKB | 부사격 조사 | Adverbial | 에, 에서, 로 |
| JKV | 호격 조사 | Vocative | 아/야 |
| JKQ | 인용격 조사 | Quotative | 라고, 고 |
| JX | 보조사 | Auxiliary | 은/는, 도, 만 |
| JC | 접속 조사 | Conjunctive | 와/과, 하고 |

### 3.5 어미

| 태그 | 명칭 | 설명 | 예시 |
|------|------|------|------|
| EP | 선어말 어미 | Pre-Final Ending | 시, 았/었 |
| EF | 종결 어미 | Final Ending | 다, 요, 니까 |
| EC | 연결 어미 | Connective Ending | 고, 면, 어서 |
| ETN | 명사형 전성 어미 | Nominal Ending | 기, 음 |
| ETM | 관형형 전성 어미 | Adnominal Ending | ㄴ, 는, ㄹ |

### 3.6 접사/어근

| 태그 | 명칭 | 설명 | 예시 |
|------|------|------|------|
| XPN | 체언 접두사 | Noun Prefix | 풋-, 헛- |
| XSN | 명사파생 접미사 | Noun Suffix | -님, -질 |
| XSV | 동사파생 접미사 | Verb Suffix | -하다, -되다 |
| XSA | 형용사파생 접미사 | Adj Suffix | -스럽다, -롭다 |
| XR | 어근 | Root | 깨끗, 착하 |

### 3.7 기호/특수

| 태그 | 명칭 | 설명 |
|------|------|------|
| SF | 마침표/물음표/느낌표 | . ? ! |
| SE | 줄임표 | … |
| SSO | 여는 괄호 | ( [ { |
| SSC | 닫는 괄호 | ) ] } |
| SC | 쉼표/콜론/빗금 | , : / |
| SY | 기타 기호 | @ # $ |
| SL | 외국어 | English, 日本語 |
| SH | 한자 | 韓國 |
| SN | 숫자 | 123, 45.6 |
| SP | 공백 | (space) |

---

## 4. 정의 파일 분석

### 4.1 char.def (문자 카테고리 정의)

#### 형식
```
CATEGORY_NAME  INVOKE  GROUP  LENGTH
0xHHHH..0xJJJJ CATEGORY [CATEGORY2...]
```

#### mecab-ko-dic 카테고리

| 카테고리 | INVOKE | GROUP | LENGTH | 설명 |
|----------|--------|-------|--------|------|
| DEFAULT | 0 | 1 | 0 | 기본 |
| SPACE | 0 | 1 | 0 | 공백 |
| **HANGUL** | 0 | 1 | 2 | 한글 |
| HANJA | 0 | 0 | 1 | 한자 |
| ALPHA | 1 | 1 | 0 | 알파벳 |
| NUMERIC | 1 | 1 | 0 | 숫자 |
| SYMBOL | 1 | 1 | 0 | 기호 |
| HANJANUMERIC | 1 | 1 | 0 | 한자 숫자 |

#### 속성 설명

| 속성 | 값 | 의미 |
|------|-----|------|
| INVOKE | 0 | 사전에 있으면 미등록어 처리 생략 |
| INVOKE | 1 | 항상 미등록어 후보도 생성 |
| GROUP | 0 | 그룹핑 비활성화 |
| GROUP | 1 | 동일 카테고리 문자 그룹핑 |
| LENGTH | n | 1~n 길이의 미등록어 후보 생성 |

#### 한글 유니코드 범위

```
0xAC00..0xD7A3  HANGUL     # 한글 음절 (가~힣, 11,172자)
0x1100..0x11FF  HANGUL     # 한글 자모
0x3130..0x318F  HANGUL     # 한글 호환 자모 (ㄱ~ㅎ, ㅏ~ㅣ)
```

### 4.2 unk.def (미등록어 정의)

#### 형식
```
CATEGORY,left_id,right_id,cost,POS,semantic,jongseong,reading,type,first,last,expr
```

#### mecab-ko-dic 미등록어 매핑

| 카테고리 | 품사 | 설명 |
|----------|------|------|
| DEFAULT | SY | 기본 → 기호 |
| SPACE | SP | 공백 |
| HANGUL | UNKNOWN | 한글 미등록어 |
| HANJA | SH | 한자 |
| ALPHA | SL | 외국어 |
| NUMERIC | SN | 숫자 |
| SYMBOL | SY | 기호 |
| HIRAGANA | SL | 외국어 |
| KATAKANA | SL | 외국어 |

### 4.3 matrix.def (연접 비용 행렬)

#### 형식
```
<left_size> <right_size>
<right_id> <left_id> <cost>
...
```

#### 비용 의미

| 비용 범위 | 의미 |
|-----------|------|
| 음수 (예: -16,124) | 자연스러운 연결 (선호) |
| 0 | 중립 |
| 양수 (예: 5,824) | 부자연스러운 연결 (비선호) |

#### Viterbi 비용 계산

```
총 비용 = matrix[lNode.rcAttr + lsize * rNode.lcAttr] + rNode.wcost + space_penalty
```

### 4.4 dicrc (사전 설정)

```ini
cost-factor = 800                          # 비용 스케일 팩터
bos-feature = BOS/EOS,*,*,*,*,*,*,*        # 문장 시작/끝 피처
eval-size = 4                              # 평가할 피처 수
config-charset = UTF-8                     # 문자셋

# 한국어 특화: 공백 페널티
left-space-penalty-factor = 100,3000,120,6000,172,3000,183,3000,...
```

---

## 5. 바이너리 사전 포맷

### 5.1 생성 파일 목록

| 파일 | 설명 | 생성 도구 |
|------|------|-----------|
| sys.dic | 시스템 사전 (DA Trie + 토큰) | mecab-dict-index |
| unk.dic | 미등록어 사전 | mecab-dict-index |
| matrix.bin | 연접 비용 행렬 | mecab-dict-index |
| char.bin | 문자 카테고리 맵 | mecab-dict-index |
| model.bin | CRF 모델 (선택) | mecab-dict-index |

### 5.2 sys.dic 바이너리 구조

```
[헤더 - 40바이트]
├── magic (4B)           # 매직 넘버
├── version (4B)         # 사전 버전
├── type (4B)            # SYS=0, UNK=1, USR=2
├── lexsize (4B)         # 어휘 항목 수
├── lsize (4B)           # 좌문맥 크기
├── rsize (4B)           # 우문맥 크기
├── dsize (4B)           # Double-Array 크기
├── tsize (4B)           # 토큰 배열 크기
├── fsize (4B)           # 피처 문자열 크기
├── dummy (4B)           # 예약
└── charset[32]          # 문자셋

[Double-Array Trie]
└── Darts 라이브러리 형식

[토큰 배열]
└── Token[] {
    lcAttr: u16      # 좌문맥 ID
    rcAttr: u16      # 우문맥 ID
    posid: u16       # 품사 ID
    wcost: i16       # 단어 비용
    feature: u32     # 피처 오프셋
    compound: u32    # 복합어 정보
}

[피처 문자열]
└── NULL 종료 문자열들
```

### 5.3 matrix.bin 바이너리 구조

```
[헤더]
├── lsize (2B)           # 좌측 크기
└── rsize (2B)           # 우측 크기

[연접 비용 행렬]
└── short[lsize * rsize] # 비용 값 배열
```

### 5.4 char.bin 바이너리 구조

```
[헤더]
└── category_count (4B)  # 카테고리 개수

[카테고리 이름]
└── char[32] * N         # 각 카테고리 이름

[CharInfo 테이블]
└── CharInfo[0xFFFF] {   # 모든 UCS-2 코드포인트
    type: 18 bits        # 카테고리 비트마스크
    default_type: 8 bits # 기본 카테고리 ID
    length: 4 bits       # LENGTH 값
    group: 1 bit         # GROUP 플래그
    invoke: 1 bit        # INVOKE 플래그
}
```

---

## 6. 빌드 프로세스

### 6.1 빌드 파이프라인

```
[seed/*.csv] ──────────────────────────────────────────────┐
      │                                                     │
      ├── mecab-dict-index -p  ───→ [left-id.def, right-id.def]
      │                                                     │
[corpus.txt] ── mecab-cost-train ──→ [model.bin]           │
      │                                                     │
      └── mecab-dict-gen ─────────→ [*.csv with costs]     │
                                          │                 │
                                          ▼                 │
                               [비용 조정 스크립트]          │
                                          │                 │
                                          ▼                 ▼
                               mecab-dict-index ───→ [sys.dic, unk.dic,
                                                      matrix.bin, char.bin]
```

### 6.2 빌드 스크립트 (seed/build.sh)

```bash
# 1단계: 품사 ID 할당
$DICT_INDEX -p -d . -c UTF-8 -t UTF-8 -f UTF-8

# 2단계: CRF 모델 학습
$COST_TRAIN -p ${cpu_count} -c 1.0 ${corpus_file} ${model_file}

# 3단계: 사전 생성 (비용 자동 할당)
$DICT_GEN -o ../final -m $model_file

# 4단계: 수동 비용 조정
./change_word_cost.sh
./change_connection_cost.sh

# 5단계: 바이너리 컴파일
cd ../final && ./configure && make
```

### 6.3 도구 목록

| 도구 | 용도 |
|------|------|
| `mecab-dict-index` | 사전 컴파일 |
| `mecab-dict-gen` | 비용 자동 생성 |
| `mecab-cost-train` | CRF 모델 학습 |
| `autoconf/automake` | 빌드 시스템 |

### 6.4 사용자 사전 추가

```bash
# final/tools/add-userdic.sh
$DICT_INDEX \
    -m ${DIC_PATH}/model.def \
    -d ${DIC_PATH} \
    -u ${DIC_PATH}/user-custom.dic \
    -f utf-8 -t utf-8 \
    -a user-dic/custom.csv
```

---

## 부록: 한국어 특화 기능

### A. mecab-ko의 left-space-penalty

띄어쓰기 앞의 형태소에 페널티를 부여하여 한국어 띄어쓰기 특성 반영:

```
품사ID 120 (조사) → 좌측 공백 시 6000 비용 추가
품사ID 172 (어미) → 좌측 공백 시 3000 비용 추가
```

### B. 종성 기반 조사 연결

| 선행 종성 | 조사 형태 |
|-----------|-----------|
| T (받침 있음) | 은, 을, 이, 과 |
| F (받침 없음) | 는, 를, 가, 와 |

### C. 수동 연접 비용 조정 예시

```
# change_connection_cost.txt
JX,*,T,는|JKO,*,을 10000     # "는을" 방지
JX,*,T,은|JX,*,은 10000      # "은은" 방지
JKG,*,F,의|BOS/EOS,*,* 10000 # "의"로 시작 방지
```

---

## 7. Rust 구현체 데이터 구조

### 7.1 Entry 구조체

`mecab-ko-dict` crate에서 정의한 사전 엔트리 구조:

```rust
/// 사전 엔트리
/// 파일 위치: rust/crates/mecab-ko-dict/src/lib.rs
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// 표면형 - 사전에 등록된 단어
    pub surface: String,

    /// 좌문맥 ID (left context ID)
    /// 연접 비용 계산 시 현재 노드의 좌측 ID로 사용
    pub left_id: u16,

    /// 우문맥 ID (right context ID)
    /// 연접 비용 계산 시 이전 노드의 우측 ID로 사용
    pub right_id: u16,

    /// 비용 (word cost)
    /// 단어 생성 비용, 낮을수록 우선 선택
    pub cost: i16,

    /// 품사 정보 (feature string)
    /// CSV의 5~12번째 필드를 콤마로 연결한 문자열
    /// 형식: "품사태그,의미부류,종성유무,읽기,타입,첫품사,끝품사,분석결과"
    pub feature: String,
}
```

#### feature 필드 상세

`feature` 필드는 8개의 서브필드로 구성됩니다:

| 인덱스 | 필드명 | 설명 | 예시 |
|--------|--------|------|------|
| 0 | 품사태그 | 품사 정보 | `NNG`, `VV+EC` |
| 1 | 의미부류 | 의미 분류 | `*`, `지명` |
| 2 | 종성유무 | 받침 여부 | `T`, `F` |
| 3 | 읽기 | 발음/원형 | `가방` |
| 4 | 타입 | 엔트리 타입 | `*`, `Compound`, `Inflect`, `Preanalysis` |
| 5 | 첫품사 | 복합 형태소 시작 품사 | `*`, `VV` |
| 6 | 끝품사 | 복합 형태소 종료 품사 | `*`, `EC` |
| 7 | 분석결과 | 형태소 분해 | `가깝/VA/*+아/EC/*` |

#### feature 예시

```
# 일반명사
NNG,*,T,가방,*,*,*,*

# 복합어 (Compound)
NNG,*,F,가가대소,Compound,*,*,가가/NNG/*+대소/NNG/*

# 활용형 (Inflect)
VA+EC,*,F,가까와,Inflect,VA,EC,가깝/VA/*+아/EC/*

# 고유명사 (지명)
NNP,지명,F,세종시,Compound,*,*,세종/NNP/지명+시/NNG/*
```

### 7.2 UserEntry 구조체

사용자 정의 사전용 엔트리:

```rust
/// 사용자 사전 엔트리
/// 파일 위치: rust/crates/mecab-ko-dict/src/user_dict.rs
#[derive(Debug, Clone, PartialEq)]
pub struct UserEntry {
    /// 표면형
    pub surface: String,
    /// 좌문맥 ID
    pub left_id: u16,
    /// 우문맥 ID
    pub right_id: u16,
    /// 비용 (낮을수록 우선)
    pub cost: i16,
    /// 품사 태그
    pub pos: String,
    /// 읽기 (발음)
    pub reading: Option<String>,
    /// 원형 (기본형)
    pub lemma: Option<String>,
}
```

### 7.3 Matrix 구조체

연접 비용 행렬 구현:

```rust
/// 밀집 연접 비용 행렬 (Dense Matrix)
/// 파일 위치: rust/crates/mecab-ko-dict/src/matrix.rs
#[derive(Debug, Clone)]
pub struct DenseMatrix {
    /// 좌문맥 크기
    lsize: usize,
    /// 우문맥 크기
    rsize: usize,
    /// 비용 배열 (row-major: costs[right_id + lsize * left_id])
    costs: Vec<i16>,
}
```

#### Matrix 인터페이스

```rust
/// 연접 비용 행렬 인터페이스
pub trait Matrix {
    /// 연접 비용 조회
    /// right_id: 이전 노드의 우문맥 ID
    /// left_id: 현재 노드의 좌문맥 ID
    fn get(&self, right_id: u16, left_id: u16) -> i32;

    /// 좌문맥 크기
    fn left_size(&self) -> usize;

    /// 우문맥 크기
    fn right_size(&self) -> usize;
}
```

#### 사용 예시

```rust
use mecab_ko_dict::matrix::{DenseMatrix, Matrix, MatrixLoader};

// 텍스트 파일에서 로드
let matrix = DenseMatrix::from_def_file("matrix.def")?;

// 바이너리 파일에서 로드
let matrix = DenseMatrix::from_bin_file("matrix.bin")?;

// 자동 포맷 감지 로드
let matrix = MatrixLoader::load("matrix.def")?;

// 연접 비용 조회
let cost = matrix.get(right_id, left_id);
println!("Connection cost: {}", cost);
```

### 7.4 CharCategoryDef 구조체

문자 카테고리 정의:

```rust
/// 문자 카테고리 정의
/// 파일 위치: rust/crates/mecab-ko-core/src/unknown.rs
#[derive(Debug, Clone)]
pub struct CharCategoryDef {
    /// 카테고리 이름 (예: "HANGUL", "ALPHA")
    pub name: String,
    /// 카테고리 ID (0-255)
    pub id: CategoryId,
    /// INVOKE 플래그: 항상 미등록어 후보 생성 여부
    pub invoke: bool,
    /// GROUP 플래그: 동일 카테고리 문자 그룹핑 여부
    pub group: bool,
    /// LENGTH: 미등록어 후보 최대 길이 (0이면 제한 없음)
    pub length: usize,
}
```

### 7.5 UnknownDef 구조체

미등록어 정의:

```rust
/// 미등록어 정의
/// 파일 위치: rust/crates/mecab-ko-core/src/unknown.rs
#[derive(Debug, Clone)]
pub struct UnknownDef {
    /// 적용 카테고리 ID
    pub category_id: CategoryId,
    /// 좌문맥 ID
    pub left_id: u16,
    /// 우문맥 ID
    pub right_id: u16,
    /// 단어 비용
    pub cost: i16,
    /// 품사 태그
    pub pos: String,
    /// 피처 문자열 (품사 정보 전체)
    pub feature: String,
}
```

### 7.6 Trie 구조체

사전 검색용 Double-Array Trie:

```rust
/// Double-Array Trie
/// 파일 위치: rust/crates/mecab-ko-dict/src/trie.rs
pub struct Trie<'a> {
    /// 내부 Double-Array (yada 라이브러리)
    da: DoubleArray<Cow<'a, [u8]>>,
}

impl<'a> Trie<'a> {
    /// 정확히 일치하는 키 검색
    pub fn exact_match(&self, key: &str) -> Option<u32>;

    /// 공통 접두사 검색 (형태소 후보 탐색)
    pub fn common_prefix_search<'b>(
        &'b self,
        text: &'b str,
    ) -> impl Iterator<Item = (u32, usize)> + 'b;
}
```

### 7.7 Matrix 타입 비교

| 타입 | 설명 | 메모리 | 사용 시나리오 |
|------|------|--------|---------------|
| `DenseMatrix` | 전체 행렬을 메모리에 저장 | O(lsize * rsize * 2) | 일반적 사용 |
| `SparseMatrix` | HashMap으로 희소 엔트리만 저장 | O(n * 10) (n=엔트리 수) | 대부분이 기본값인 경우 |
| `MmapMatrix` | 메모리 맵 방식 | 디스크에서 직접 접근 | 대용량 사전, 멀티프로세스 |

### 7.8 바이너리 포맷 상세

#### matrix.bin 바이너리 포맷 (Rust 구현체)

```
[헤더 - 4바이트]
├── lsize (u16, little-endian)   # 좌문맥 크기
└── rsize (u16, little-endian)   # 우문맥 크기

[데이터]
└── costs[lsize * rsize] (i16, little-endian)  # 비용 배열
```

**접근 공식**:
```
index = right_id + lsize * left_id
cost = costs[index]
```

#### Trie 바이너리 포맷

yada 라이브러리의 Double-Array Trie 직렬화 형식을 사용합니다.
zstd 압축 지원 (`*.zst` 확장자).

---

## 8. API 사용 예제

### 8.1 사전 엔트리 생성

```rust
use mecab_ko_dict::Entry;

let entry = Entry {
    surface: "안녕".to_string(),
    left_id: 1,
    right_id: 1,
    cost: 100,
    feature: "NNG,*,T,안녕,*,*,*,*".to_string(),
};
```

### 8.2 사용자 사전 사용

```rust
use mecab_ko_dict::user_dict::{UserDictionary, UserDictionaryBuilder};

// 빌더 패턴
let dict = UserDictionaryBuilder::new()
    .default_cost(-1000)
    .add("딥러닝", "NNG")
    .add_with_cost("머신러닝", "NNG", -500)
    .add_full("챗GPT", "NNP", -1000, Some("챗지피티"))
    .build();

// 직접 추가
let mut dict = UserDictionary::new();
dict.add_entry("클로드", "NNP", Some(-1000), None);

// CSV 파일에서 로드
dict.load_from_csv("user-dict.csv")?;

// 검색
let entries = dict.lookup("딥러닝");
```

### 8.3 연접 비용 행렬 사용

```rust
use mecab_ko_dict::matrix::{DenseMatrix, Matrix};

// 로드
let matrix = DenseMatrix::from_def_file("matrix.def")?;

// "나는" = "나/NP" + "는/JX" 연접 비용 계산
// NP의 right_id가 154, JX의 left_id가 120이라고 가정
let connection_cost = matrix.get(154, 120);

// 총 비용 계산
let total_cost = prev_node_cost + connection_cost + current_word_cost;
```

### 8.4 미등록어 처리

```rust
use mecab_ko_core::unknown::UnknownHandler;

let handler = UnknownHandler::korean_default();

// 미등록어 후보 생성
let candidates = handler.generate_candidates("테스트ABC", 0, false);

for candidate in candidates {
    println!("{}: {} (cost: {})",
        candidate.surface,
        candidate.pos,
        candidate.cost
    );
}
```

---

## 참고 자료

- [mecab-ko-dic Bitbucket](https://bitbucket.org/eunjeon/mecab-ko-dic)
- [mecab-ko GitHub](https://bitbucket.org/eunjeon/mecab-ko)
- [세종 품사 태그 스프레드시트](https://docs.google.com/spreadsheets/d/1-9blXKjtjeKZqsf4NzHeYJCrr49-nXeRF6D80udfcwY)
- [MeCab 공식 문서](https://taku910.github.io/mecab/)

---

## 변경 이력

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| v2.1 | 2026-01-05 | Rust 구현체 Entry/Matrix/Trie 구조체 문서 추가 |
| v2.0 | 2026-01-04 | 사전 포맷 완전 분석 및 문서화 |
| v1.0 | - | 초기 mecab-ko-dic 포맷 |
