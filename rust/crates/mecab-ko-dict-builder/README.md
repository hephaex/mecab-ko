# mecab-ko-dict-builder

한국어 형태소 사전 빌더 - mecab-ko-dic CSV 형식을 바이너리 사전으로 변환

## 기능

- **CSV 파싱**: mecab-ko-dic 12컬럼 형식 지원
- **Double-Array Trie 구축**: yada 라이브러리 기반 고속 검색 구조
- **연접 비용 행렬**: matrix.def 파일 파싱 및 바이너리 변환
- **문자 타입 정의**: char.def 파싱 및 바이너리 변환 (NEW)
- **미등록어 정의**: unk.def 파싱 및 바이너리 변환 (NEW)
- **압축 지원**: zstd 압축 (레벨 0-22)
- **인코딩 지원**: UTF-8, EUC-KR 자동 감지
- **한글 종성 처리**: 자동 종성 유무 판별

## 사용법

### CLI

```bash
# 기본 사용
mecab-ko-dict-builder --input ./mecab-ko-dic --output ./dict

# 압축 레벨 지정
mecab-ko-dict-builder -i ./mecab-ko-dic -o ./dict -c 5

# 인코딩 지정
mecab-ko-dict-builder -i ./mecab-ko-dic -o ./dict -e utf8

# 자세한 로그 출력
mecab-ko-dict-builder -i ./mecab-ko-dic -o ./dict -v
```

### 라이브러리

```rust
use mecab_ko_dict_builder::{DictionaryBuilder, builder::BuildConfig, csv_parser::Encoding};

let config = BuildConfig {
    input_dir: "./mecab-ko-dic".to_string(),
    output_dir: "./dict".to_string(),
    compression_level: 3,
    encoding: Encoding::Auto,
    verbose: true,
};

let builder = DictionaryBuilder::new(config);
let result = builder.build()?;

println!("Built dictionary with {} entries", result.entry_count);
```

## 입력 형식

### CSV 파일 (12컬럼)

```csv
표면형,좌ID,우ID,비용,품사,품사세분류,종성유무,읽기,타입,첫품사,마지막품사,표현
가,1,1,100,NNG,*,T,가,*,NNG,NNG,*
가다,2,2,200,VV,*,F,가다,*,VV,VV,*
```

### matrix.def

```
<lsize> <rsize>
<right_id> <left_id> <cost>
...
```

### char.def (선택적)

문자 타입 정의:

```
# 타입 정의: TYPE_NAME invoke group length
DEFAULT 1 0 0
SPACE   0 1 0
HANGUL  1 1 0

# 문자 매핑: 0xCODE TYPE_NAME
0x0020 SPACE
0x0009 SPACE
```

### unk.def (선택적)

미등록어 처리 정의:

```
# 문자타입,좌ID,우ID,비용,Feature
DEFAULT,0,0,0,UNK,*,*,*,*,*,*,*
SPACE,0,0,0,SP,*,*,*,*,*,*,*
HANGUL,1,1,1000,UNK,*,*,*,*,*,*,*
```

## 출력 형식

### sys.dic (또는 sys.dic.zst)

Double-Array Trie 바이너리 파일

- 형태소 표면형 검색을 위한 자료구조
- 압축 시 zstd 형식

### matrix.bin (또는 matrix.bin.zst)

연접 비용 행렬 바이너리 파일

- Little-endian i16 배열
- 헤더: lsize (u16) + rsize (u16)
- 데이터: lsize × rsize 개의 i16 값

### char.bin (선택적)

문자 타입 정의 바이너리

- 타입 정의 및 문자 매핑 정보

### unk.bin (선택적)

미등록어 정의 바이너리

- 문자 타입별 미등록어 처리 규칙

## 성능

- **메모리 효율적**: 스트리밍 파싱으로 대용량 사전 처리
- **압축 효과**: zstd 레벨 3 기준 약 30-40% 크기 감소
- **빠른 빌드**: 멀티코어 활용 및 최적화된 자료구조

## 의존성

- `yada`: Double-Array Trie 구현
- `csv`: CSV 파싱
- `zstd`: 압축
- `encoding_rs`: 인코딩 변환
- `mecab-ko-hangul`: 한글 처리
- `mecab-ko-dict`: 사전 데이터 구조

## 라이선스

Apache-2.0 OR MIT

## 관련 프로젝트

- [mecab-ko-dic](https://bitbucket.org/eunjeon/mecab-ko-dic) - 원본 사전 데이터
- [mecab-ko](https://bitbucket.org/eunjeon/mecab-ko) - MeCab 한국어 버전
