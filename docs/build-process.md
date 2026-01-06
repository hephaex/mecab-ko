# MeCab-Ko-Dic 빌드 프로세스 가이드

> **문서 버전**: 1.0
> **작성일**: 2026-01-04
> **대상**: mecab-ko-dic 사전 빌드 시스템

이 문서는 DIC-001 이슈의 산출물로, mecab-ko-dic의 빌드 프로세스를 상세히 문서화합니다.

---

## 목차

1. [빌드 환경 준비](#1-빌드-환경-준비)
2. [빌드 파이프라인 개요](#2-빌드-파이프라인-개요)
3. [단계별 빌드 프로세스](#3-단계별-빌드-프로세스)
4. [핵심 도구 분석](#4-핵심-도구-분석)
5. [비용 조정 시스템](#5-비용-조정-시스템)
6. [사용자 사전 빌드](#6-사용자-사전-빌드)
7. [Rust 재구현 고려사항](#7-rust-재구현-고려사항)

---

## 1. 빌드 환경 준비

### 1.1 필수 도구

```bash
# MeCab 코어 도구
mecab-dict-index    # 사전 컴파일러
mecab-dict-gen      # 사전 생성기
mecab-cost-train    # CRF 학습기

# 빌드 시스템
autoconf            # configure 스크립트 생성
automake            # Makefile 생성
make                # 빌드 실행
```

### 1.2 디렉토리 구조

```
mecab-ko-dic/
├── seed/           # 입력: CSV 원본 + 정의 파일
│   ├── *.csv
│   ├── *.def
│   ├── dicrc
│   ├── build.sh
│   └── corpus/
└── final/          # 출력: 바이너리 사전
    ├── *.dic
    ├── *.bin
    ├── *.def
    └── dicrc
```

---

## 2. 빌드 파이프라인 개요

### 2.1 전체 흐름도

```
┌─────────────────────────────────────────────────────────────────┐
│  Phase 1: 초기화                                                │
│  ───────────────                                                │
│  seed/*.csv + *.def ──→ mecab-dict-index -p                    │
│                              │                                  │
│                              ▼                                  │
│                    left-id.def, right-id.def                   │
├─────────────────────────────────────────────────────────────────┤
│  Phase 2: CRF 학습                                              │
│  ───────────────                                                │
│  corpus/eunjeon_corpus.txt ──→ mecab-cost-train                │
│                                      │                          │
│                                      ▼                          │
│                                 model.bin                       │
├─────────────────────────────────────────────────────────────────┤
│  Phase 3: 사전 생성                                             │
│  ───────────────                                                │
│  *.csv + model.bin ──→ mecab-dict-gen                          │
│                              │                                  │
│                              ▼                                  │
│                    비용이 할당된 CSV 파일들                     │
├─────────────────────────────────────────────────────────────────┤
│  Phase 4: 비용 조정                                             │
│  ───────────────                                                │
│  change_word_cost.sh ──→ 단어 비용 수동 조정                   │
│  change_connection_cost.sh ──→ 연접 비용 수동 조정             │
├─────────────────────────────────────────────────────────────────┤
│  Phase 5: 바이너리 컴파일                                       │
│  ─────────────────                                              │
│  final/*.csv ──→ mecab-dict-index                              │
│                       │                                         │
│                       ▼                                         │
│    sys.dic, unk.dic, matrix.bin, char.bin                      │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 입출력 파일 매핑

| 입력 | 처리 | 출력 |
|------|------|------|
| `*.csv` | 사전 컴파일 | `sys.dic` |
| `unk.def` | 미등록어 컴파일 | `unk.dic` |
| `char.def` | 문자 속성 컴파일 | `char.bin` |
| `matrix.def` | 연접 행렬 컴파일 | `matrix.bin` |
| `feature.def` | 모델 컴파일 | `model.bin` |

---

## 3. 단계별 빌드 프로세스

### 3.1 Phase 1: 품사 ID 할당

```bash
# seed/build.sh 일부
DICT_INDEX=/usr/local/libexec/mecab/mecab-dict-index

# 품사 ID 자동 생성 (-p 옵션)
$DICT_INDEX -p -d . -c UTF-8 -t UTF-8 -f UTF-8
```

**생성 파일**:
- `left-id.def`: 좌측 문맥 ID 매핑
- `right-id.def`: 우측 문맥 ID 매핑

**left-id.def 형식**:
```
BOS/EOS,*,*,*,*,*,*,* 0
NNG,*,*,*,*,*,*,* 150
VV,*,*,*,*,*,*,* 173
JKO,*,*,*,*,*,*,* 120
...
```

### 3.2 Phase 2: CRF 모델 학습

```bash
COST_TRAIN=/usr/local/libexec/mecab/mecab-cost-train
corpus_file="corpus/eunjeon_corpus.txt"
model_file="model.def"

# CRF 학습 실행
$COST_TRAIN -p ${cpu_count} -c 1.0 ${corpus_file} ${model_file}
```

**코퍼스 형식** (`eunjeon_corpus.txt`):
```
안녕	NNG,인사,T,안녕,*,*,*,*
하	XSV,*,F,하,*,*,*,*
세요	EP+EF,*,F,세요,Inflect,EP,EF,시/EP/*+어요/EF/*
EOS
나	NP,*,F,나,*,*,*,*
는	JX,*,T,는,*,*,*,*
...
```

**학습 파라미터**:
- `-p N`: 병렬 처리 스레드 수
- `-c 1.0`: CRF 정규화 계수

### 3.3 Phase 3: 사전 생성

```bash
DICT_GEN=/usr/local/libexec/mecab/mecab-dict-gen

# 모델 기반 비용 자동 할당
$DICT_GEN -o ../final -m $model_file
```

**처리 내용**:
1. CSV 파일의 `left_id`, `right_id`, `cost` 컬럼 채우기
2. CRF 모델 기반 최적 비용 계산
3. `matrix.def` 생성 (연접 비용 행렬)

### 3.4 Phase 4: 비용 수동 조정

#### 단어 비용 조정

```bash
# change_word_cost.sh
while read line; do
    surface=$(echo $line | cut -d',' -f1)
    pos=$(echo $line | cut -d',' -f2)
    new_cost=$(echo $line | cut -d',' -f3)

    # CSV 파일에서 해당 단어의 비용 수정
    sed -i "s/^${surface},.*,${pos},/${surface},0,0,${new_cost},${pos},/" *.csv
done < change_word_cost.txt
```

**change_word_cost.txt 예시**:
```
은,JX,-1000       # "은" 보조사 비용 낮춤
를,JKO,-500       # "를" 목적격 조사
```

#### 연접 비용 조정

```bash
# change_connection_cost.sh
# matrix.def 에서 특정 품사 조합의 비용 수정
```

**change_connection_cost.txt 예시**:
```
JX,*,T,는|JKO,*,을 10000     # "는을" 연접 억제
JX,*,T,은|JX,*,은 10000      # "은은" 연접 억제
JKG,*,F,의|BOS/EOS,*,* 10000 # 문장 시작 "의" 억제
```

### 3.5 Phase 5: 바이너리 컴파일

```bash
cd ../final

# Autotools 빌드
./autogen.sh    # configure 스크립트 생성
./configure     # 빌드 설정
make            # 바이너리 생성
make install    # 설치 (선택)
```

**생성되는 바이너리**:

| 파일 | 크기 (대략) | 설명 |
|------|-------------|------|
| `sys.dic` | 40-50 MB | 시스템 사전 (DA Trie) |
| `unk.dic` | 수 KB | 미등록어 사전 |
| `matrix.bin` | 수 MB | 연접 비용 행렬 |
| `char.bin` | 수백 KB | 문자 속성 테이블 |

---

## 4. 핵심 도구 분석

### 4.1 mecab-dict-index

**소스**: `mecab-ko/src/dictionary_compiler.cpp`

**주요 옵션**:
```
-d DIR      입력 사전 디렉토리
-o DIR      출력 디렉토리
-f CHARSET  입력 CSV 문자셋 (UTF-8)
-t CHARSET  출력 바이너리 문자셋 (UTF-8)
-p          품사 ID 할당 모드
-s          시스템 사전 빌드
-u FILE     사용자 사전 생성
-m FILE     모델 파일 (비용 자동 추정)
-U          미등록어 사전 빌드
-C          문자 카테고리 빌드
```

**내부 처리 흐름**:
```cpp
// dictionary_compiler.cpp
int DictionaryCompiler::run(int argc, char **argv) {
    // 1. 문자 카테고리 컴파일
    CharProperty::compile(char_def, unk_def, char_bin);

    // 2. 미등록어 사전 컴파일
    Dictionary::compile(param, tmp, unk_dic);

    // 3. 시스템 사전 컴파일
    Dictionary::compile(param, dic_files, sys_dic);

    // 4. 연접 행렬 컴파일
    Connector::compile(matrix_def, matrix_bin);
}
```

### 4.2 Double-Array Trie (Darts)

**소스**: `mecab-ko/src/darts.h`

사전 검색에 사용되는 고속 Trie 구조:

```cpp
// 공통 접두사 검색
size_t commonPrefixSearch(const char *key,
                          result_type *result,
                          size_t result_len,
                          size_t len) const;

// 정확 매칭 검색
int exactMatchSearch(const char *key, size_t len) const;
```

**특징**:
- O(n) 검색 복잡도 (n = 키 길이)
- 메모리 효율적 압축
- 한글 UTF-8 지원

### 4.3 연접 비용 행렬 (Connector)

**소스**: `mecab-ko/src/connector.cpp`

```cpp
int Connector::cost(const Node *lNode, const Node *rNode) const {
    // 행렬 인덱스 계산: rcAttr + lsize * lcAttr
    int base_cost = matrix_[lNode->rcAttr + lsize_ * rNode->lcAttr];

    // 단어 비용 추가
    int word_cost = rNode->wcost;

    // mecab-ko 특화: 공백 페널티
    int space_penalty = get_space_penalty_cost(rNode);

    return base_cost + word_cost + space_penalty;
}
```

---

## 5. 비용 조정 시스템

### 5.1 비용 구성 요소

```
총 비용 = 연접 비용 + 단어 비용 + 공백 페널티
          (matrix)   (wcost)    (left-space-penalty)
```

### 5.2 비용 스케일링

**dicrc 설정**:
```ini
cost-factor = 800
```

CRF 학습된 raw 비용에 800을 곱하여 정수화:
```
final_cost = raw_cost * 800
```

### 5.3 공백 페널티 (mecab-ko 특화)

**dicrc 설정**:
```ini
left-space-penalty-factor = 100,3000,120,6000,172,3000,183,3000
```

형식: `품사ID1,페널티1,품사ID2,페널티2,...`

| 품사 ID | 품사 | 페널티 | 의미 |
|---------|------|--------|------|
| 120 | JKO (목적격 조사) | 6000 | 조사 앞 공백 억제 |
| 172 | EC (연결 어미) | 3000 | 어미 앞 공백 억제 |

---

## 6. 사용자 사전 빌드

### 6.1 CSV 형식

```csv
# user-dic/custom.csv
카카오뱅크,,,0,NNP,기업,F,카카오뱅크,Compound,*,*,카카오/NNP/*+뱅크/NNG/*
테슬라,,,0,NNP,기업,F,테슬라,*,*,*,*
GPT-4,,,0,SL,*,*,GPT-4,*,*,*,*
```

### 6.2 빌드 명령

```bash
# final/tools/add-userdic.sh
DICT_INDEX=/usr/local/libexec/mecab/mecab-dict-index
DIC_PATH=/usr/local/lib/mecab/dic/mecab-ko-dic

$DICT_INDEX \
    -m ${DIC_PATH}/model.def \    # 비용 자동 추정
    -d ${DIC_PATH} \              # 시스템 사전 참조
    -u ${DIC_PATH}/user-custom.dic \  # 출력 파일
    -f utf-8 -t utf-8 \
    -a user-dic/custom.csv        # 입력 CSV
```

### 6.3 사전 우선순위

```
1. 사용자 사전 (user-*.dic)
2. 시스템 사전 (sys.dic)
3. 미등록어 사전 (unk.dic)
```

---

## 7. Rust 재구현 고려사항

### 7.1 필요한 Crate

| 기능 | C++ | Rust 대안 |
|------|-----|-----------|
| Double-Array Trie | darts | `yada`, `daachorse` |
| CRF 학습 | crfpp | `crfsuite-rs`, 자체 구현 |
| 바이너리 직렬화 | 자체 포맷 | `bincode`, `rkyv` |
| 문자셋 변환 | iconv | `encoding_rs` |

### 7.2 바이너리 호환성

**옵션 1: 기존 포맷 호환**
- 장점: 기존 mecab-ko-dic 그대로 사용 가능
- 단점: 레거시 포맷 제약

**옵션 2: 새 포맷 설계 (v3.0)**
- 장점: Rust 최적화, 현대적 압축
- 단점: 사전 재빌드 필요

### 7.3 핵심 구현 모듈

```rust
// 제안 구조
crates/
├── mecab-ko-dict/
│   ├── src/
│   │   ├── loader.rs      // 사전 로더
│   │   ├── builder.rs     // 사전 빌더
│   │   ├── trie.rs        // Double-Array Trie
│   │   ├── matrix.rs      // 연접 행렬
│   │   ├── char_prop.rs   // 문자 속성
│   │   └── format.rs      // 바이너리 포맷
│   └── Cargo.toml
```

### 7.4 빌드 도구 재구현

```rust
// mecab-ko-dict-tools (CLI)
pub enum Command {
    /// 품사 ID 할당
    AssignPosId { input_dir: PathBuf },

    /// 사전 컴파일
    Compile {
        input_dir: PathBuf,
        output_dir: PathBuf,
        charset: String,
    },

    /// 사용자 사전 추가
    AddUserDict {
        sys_dic: PathBuf,
        user_csv: PathBuf,
        output: PathBuf,
    },
}
```

---

## 부록: 빌드 트러블슈팅

### A. 일반적인 오류

| 오류 | 원인 | 해결 |
|------|------|------|
| `unknown POS tag` | CSV 품사 태그 오류 | pos-id.def 확인 |
| `charset mismatch` | 인코딩 불일치 | `-f UTF-8 -t UTF-8` 확인 |
| `matrix size mismatch` | ID 범위 초과 | left/right-id.def 재생성 |

### B. 성능 최적화

```bash
# 병렬 빌드
make -j$(nproc)

# CRF 학습 병렬화
mecab-cost-train -p $(nproc) ...
```

---

## 참고 자료

- [MeCab 공식 문서](https://taku910.github.io/mecab/)
- [mecab-ko-dic 소스](https://bitbucket.org/eunjeon/mecab-ko-dic)
- [CRF++ 라이브러리](https://taku910.github.io/crfpp/)
- [Darts 라이브러리](https://chasen.org/~taku/software/darts/)
