# 사전 빌드 가이드

MeCab-Ko는 CSV 형식의 사전 소스 파일에서 바이너리 사전을 빌드하는 도구를 제공합니다.

## 개요

사전 빌드 프로세스:
1. CSV 사전 파일 준비
2. 특성 정의 파일 작성
3. 사전 빌더 실행
4. 바이너리 사전 생성

## 사전 파일 구조

### CSV 사전 형식

```csv
표면형,좌문맥ID,우문맥ID,비용,품사,의미정보1,의미정보2,...
```

예시:
```csv
가,1788,3544,3775,JKS,*,F,가,*,*,*,*
가게,1781,3536,2876,NNG,*,F,가게,*,*,*,*
학교,1781,3536,-1723,NNG,장소,F,학교,Compound,*,*,학교
```

### 필드 설명

| 필드 | 설명 | 필수 |
|------|------|------|
| 표면형 | 사전 단어 | O |
| 좌문맥ID | 좌측 문맥 ID (0-65535) | O |
| 우문맥ID | 우측 문맥 ID (0-65535) | O |
| 비용 | 단어 비용 (낮을수록 선호) | O |
| 품사 | 품사 태그 (NNG, VV 등) | O |
| 의미정보1-7 | 추가 의미 정보 | X |

### 특성 정의 파일 (feature.def)

```
# 품사 태그 정의
NNG    일반 명사
NNP    고유 명사
VV     동사
VA     형용사
...
```

## CLI 도구로 빌드

### 기본 빌드

```bash
mecab-ko-dict-build \
  --input ./dict-src \
  --output ./dict-bin \
  --charset utf-8
```

### 옵션

```bash
mecab-ko-dict-build [OPTIONS]

Options:
  -i, --input <DIR>        입력 사전 디렉토리
  -o, --output <DIR>       출력 디렉토리
  -c, --charset <CHARSET>  문자 인코딩 [기본값: utf-8]
  -d, --dictionary <TYPE>  사전 타입 [mecab-ko-dic, ipa, uni]
  -f, --feature <FILE>     특성 정의 파일
  -m, --matrix <FILE>      연접 비용 행렬 파일
  -w, --wakati             Wakati 모드
  -h, --help               도움말 출력
  -V, --version            버전 정보
```

## Rust API로 빌드

### 기본 사용

```rust
use mecab_ko_dict::builder::{DictBuilder, BuildConfig};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = BuildConfig {
        input_dir: Path::new("./dict-src").to_path_buf(),
        output_dir: Path::new("./dict-bin").to_path_buf(),
        charset: "utf-8".to_string(),
        ..Default::default()
    };

    let builder = DictBuilder::new(config)?;
    builder.build()?;

    println!("사전 빌드 완료!");
    Ok(())
}
```

### 고급 설정

```rust
use mecab_ko_dict::builder::{DictBuilder, BuildConfig, DictType};

let config = BuildConfig {
    input_dir: Path::new("./dict-src").to_path_buf(),
    output_dir: Path::new("./dict-bin").to_path_buf(),
    charset: "utf-8".to_string(),
    dict_type: DictType::MecabKoDic,
    wakati: false,
    user_dict: Some(Path::new("./user.csv").to_path_buf()),
    matrix_file: Some(Path::new("./matrix.def").to_path_buf()),
    feature_file: Some(Path::new("./feature.def").to_path_buf()),
    compression: true,
    verbose: true,
};

let builder = DictBuilder::new(config)?;
builder.build()?;
```

## 사전 소스 디렉토리 구조

```
dict-src/
├── lex.csv              # 메인 사전 파일
├── matrix.def           # 연접 비용 행렬
├── feature.def          # 특성 정의
├── char.def             # 문자 정의
├── unk.def              # 미등록어 정의
└── rewrite.def          # 재작성 규칙 (선택)
```

### lex.csv

주요 사전 엔트리:

```csv
# 명사
학교,1781,3536,-1723,NNG,장소,F,학교,*,*,*,*
컴퓨터,1781,3536,-2154,NNG,*,F,컴퓨터,Compound,*,*,컴퓨터

# 동사
가다,1788,3544,4500,VV,*,F,가,*,*,*,*
먹다,1788,3544,5200,VV,*,F,먹,*,*,*,*

# 조사
이,1789,3545,2500,JKS,*,F,이,*,*,*,*
가,1788,3544,3775,JKS,*,F,가,*,*,*,*
```

### matrix.def

연접 비용 행렬:

```
# 크기 정의
1316 1316

# 좌문맥ID 우문맥ID 비용
0 0 0
0 1 100
0 2 200
...
```

### char.def

문자 타입 정의:

```
# 타입명 문자범위 invoke group length
DEFAULT 0 1 0
SPACE   0x0020 0 1 0
HANGUL  0xAC00..0xD7A3 0 2 0
ALPHA   a..z 1 1 0
ALPHA   A..Z 1 1 0
DIGIT   0..9 1 1 0
```

### unk.def

미등록어 처리:

```
DEFAULT,1,2,3000,UNK,*,*,*,*,*,*,*
HANGUL,1781,3536,5000,NNG,*,*,*,*,*,*,*
ALPHA,1,2,3500,SL,*,*,*,*,*,*,*
DIGIT,1,2,3500,SN,*,*,*,*,*,*,*
```

## 사용자 사전 빌드

### CSV 형식

```csv
# 표면형,품사,비용,기본형
딥러닝,NNG,-1000,딥러닝
머신러닝,NNG,-1000,머신러닝
트랜스포머,NNG,-1000,트랜스포머
```

### 간단한 형식으로 빌드

```bash
mecab-ko-dict-build \
  --input user.csv \
  --output user-dict.bin \
  --user-dict
```

### 프로그래밍 방식

```rust
use mecab_ko_dict::builder::UserDictBuilder;

let builder = UserDictBuilder::new();
builder
    .add_entry("딥러닝", "NNG", -1000, Some("딥러닝"))?
    .add_entry("머신러닝", "NNG", -1000, Some("머신러닝"))?
    .add_entry("GPT", "SL", -1000, None)?
    .build_to_file("user-dict.bin")?;
```

## 최적화

### 압축

바이너리 사전 압축:

```rust
let config = BuildConfig {
    compression: true,
    compression_level: 6, // 1-9
    ..Default::default()
};
```

### 메모리 매핑

대용량 사전은 메모리 매핑 사용:

```rust
let config = BuildConfig {
    use_mmap: true,
    ..Default::default()
};
```

### 증분 빌드

변경된 파일만 재빌드:

```rust
let builder = DictBuilder::new(config)?;
builder.build_incremental()?; // 증분 빌드
```

## 사전 검증

### 빌드 후 검증

```bash
mecab-ko-dict-validate \
  --dict-dir ./dict-bin \
  --check-all
```

### 프로그래밍 검증

```rust
use mecab_ko_dict::validator::DictValidator;

let validator = DictValidator::new("./dict-bin")?;

// 기본 검증
validator.validate_basic()?;

// 완전 검증
validator.validate_full()?;

// 통계 출력
let stats = validator.statistics();
println!("총 엔트리: {}", stats.total_entries);
println!("총 노드: {}", stats.total_nodes);
```

## 벤치마킹

빌드된 사전의 성능 측정:

```bash
mecab-ko-dict-bench \
  --dict-dir ./dict-bin \
  --corpus ./test-corpus.txt
```

결과:
```
사전 로딩 시간: 45ms
평균 분석 속도: 2.3MB/s
메모리 사용량: 125MB
```

## 사전 병합

여러 사전 소스 병합:

```rust
use mecab_ko_dict::builder::DictMerger;

let merger = DictMerger::new();
merger
    .add_source("./mecab-ko-dic")?
    .add_source("./user-dict")?
    .add_source("./domain-dict")?
    .merge_to("./merged-dict")?;
```

## 고급: 연접 비용 학습

### 말뭉치에서 학습

```bash
mecab-ko-cost-train \
  --corpus ./sejong-corpus.txt \
  --output ./matrix.def \
  --iterations 100
```

### CRF 기반 학습

```rust
use mecab_ko_dict::trainer::CostTrainer;

let trainer = CostTrainer::new();
trainer
    .load_corpus("./corpus.txt")?
    .train(100)?
    .save_matrix("./matrix.def")?;
```

## 문제 해결

### 빌드 오류

#### 1. "Invalid CSV format"

```bash
# CSV 유효성 검사
mecab-ko-dict-validate --csv ./lex.csv
```

#### 2. "Matrix dimension mismatch"

matrix.def의 차원이 일치하지 않음:

```bash
# 자동으로 matrix.def 생성
mecab-ko-dict-build --auto-matrix
```

#### 3. "Out of memory"

대용량 사전의 경우:

```bash
# 스트리밍 모드로 빌드
mecab-ko-dict-build --streaming --chunk-size 10000
```

### 성능 최적화

#### 빌드 시간 단축

```bash
# 병렬 빌드
mecab-ko-dict-build --parallel --jobs 8
```

#### 사전 크기 축소

```bash
# 미사용 엔트리 제거
mecab-ko-dict-build --prune --min-cost -10000
```

## 예제: mecab-ko-dic 빌드

```bash
# 1. 소스 다운로드
git clone https://github.com/hephaex/mecab-ko-dic.git
cd mecab-ko-dic

# 2. 빌드
mecab-ko-dict-build \
  --input . \
  --output ./build \
  --charset utf-8 \
  --dictionary mecab-ko-dic \
  --parallel

# 3. 검증
mecab-ko-dict-validate --dict-dir ./build

# 4. 설치
sudo cp -r ./build /usr/local/lib/mecab/dic/mecab-ko-dic
```

## 참고 자료

- [사전 포맷 명세](../reference/dictionary-format.md)
- [바이너리 포맷](../reference/binary-format.md)
- [품사 태그](../reference/pos-tags.md)
