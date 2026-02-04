# 예제 설치 가이드

이 디렉토리의 예제들을 실행하려면, `mecab-ko-core` 크레이트의 `examples/` 디렉토리로 복사해야 합니다.

## 방법 1: 수동 복사

```bash
# 프로젝트 루트에서
cp examples/*.rs rust/crates/mecab-ko-core/examples/
```

## 방법 2: 심볼릭 링크 (개발용)

```bash
# 프로젝트 루트에서
cd rust/crates/mecab-ko-core/examples

ln -s ../../../../examples/text_preprocessing.rs .
ln -s ../../../../examples/keyword_extraction.rs .
ln -s ../../../../examples/search_tokenizer.rs .
```

## 방법 3: 스크립트 사용

```bash
# 프로젝트 루트에서
./scripts/install_examples.sh
```

## 실행

복사 후 다음과 같이 실행할 수 있습니다:

```bash
cd rust/crates/mecab-ko-core

# 텍스트 전처리
cargo run --example text_preprocessing

# 키워드 추출
cargo run --example keyword_extraction

# 검색 토크나이저
cargo run --example search_tokenizer
```

## 주의사항

1. **사전 설치 필요**: 예제 실행 전 MeCab-Ko 사전이 설치되어 있어야 합니다.

   ```bash
   # Ubuntu/Debian
   sudo apt-get install mecab-ko mecab-ko-dic

   # macOS
   brew install mecab-ko mecab-ko-dic
   ```

2. **환경 변수 설정** (선택사항):

   ```bash
   export MECAB_DICDIR=/usr/local/lib/mecab/dic/mecab-ko-dic
   ```

3. **컴파일 오류 시**:
   - `mecab-ko-core`의 API가 변경되었을 수 있습니다.
   - README.md의 최신 API 사용법을 참고하세요.

## 예제 목록

- `text_preprocessing.rs` - NLP 전처리 파이프라인
- `keyword_extraction.rs` - 키워드 추출 알고리즘
- `search_tokenizer.rs` - 검색 엔진 토크나이저

자세한 내용은 `README.md`를 참고하세요.
