# 마이그레이션 가이드

기존 MeCab-Ko C++ 버전에서 Rust 버전으로 마이그레이션하는 방법을 안내합니다.

## C++ API에서 Rust API로

### Tagger 생성

**C++ (기존)**
```cpp
MeCab::Tagger *tagger = MeCab::createTagger("");
```

**Rust (신규)**
```rust
let tagger = Tagger::new(TaggerConfig::default())?;
```

### 분석 실행

**C++ (기존)**
```cpp
const char* result = tagger->parse("안녕하세요");
```

**Rust (신규)**
```rust
let result = tagger.parse("안녕하세요")?;
```

### Node 순회

**C++ (기존)**
```cpp
for (const MeCab::Node *node = tagger->parseToNode(text);
     node;
     node = node->next) {
    std::cout << node->surface << std::endl;
}
```

**Rust (신규)**
```rust
let result = tagger.parse("텍스트")?;
for node in result.iter() {
    println!("{}", node.surface);
}
```

### 리소스 정리

**C++ (기존)**
```cpp
delete tagger;
```

**Rust (신규)**
```rust
// 자동으로 Drop 트레잇이 처리
// 명시적으로 drop(tagger); 가능하지만 불필요
```

## Python 바인딩 마이그레이션

### mecab-python에서 mecab-ko로

**mecab-python (기존)**
```python
import MeCab

tagger = MeCab.Tagger()
result = tagger.parse("안녕하세요")
```

**mecab-ko (신규)**
```python
from mecab_ko import Tagger

tagger = Tagger()
result = tagger.parse("안녕하세요")
```

대부분의 API가 호환되므로 import만 변경하면 됩니다.

## 설정 파일 마이그레이션

### mecabrc 파일

**C++ (기존)**
```
dicdir = /usr/local/lib/mecab/dic/mecab-ko-dic
userdic = /path/to/user.dic
```

**Rust (신규)**
```rust
let config = TaggerConfig {
    dict_dir: Some(PathBuf::from("/usr/local/lib/mecab/dic/mecab-ko-dic")),
    user_dict: Some(PathBuf::from("/path/to/user.csv")),
    ..Default::default()
};
```

## CLI 도구 마이그레이션

### 명령어 옵션

대부분의 옵션이 호환됩니다:

| C++ | Rust | 설명 |
|-----|------|------|
| `-d DIR` | `-d DIR` | 사전 디렉토리 |
| `-u FILE` | `--user-dic FILE` | 사용자 사전 |
| `-O FORMAT` | `-o FORMAT` | 출력 포맷 |
| `-N NUM` | `--nbest NUM` | N-best 개수 |

## 사전 포맷 호환성

Rust 버전은 기존 C++ 사전 파일을 그대로 사용할 수 있습니다.

```bash
# 기존 mecab-ko-dic 사용
mecab-ko -d /usr/local/lib/mecab/dic/mecab-ko-dic "텍스트"
```

## 성능 차이

일반적으로 Rust 버전이 C++ 버전과 유사하거나 약간 더 빠릅니다:

- 단일 스레드: C++ 대비 95-105%
- 멀티 스레드: C++ 대비 110-130% (Rayon 사용 시)
- 메모리 사용: C++ 대비 80-90%

## 호환성 문제

### 1. 출력 포맷 차이

일부 edge case에서 출력 포맷이 약간 다를 수 있습니다. 대부분의 경우 동일합니다.

### 2. 사용자 사전 포맷

CSV 포맷이 조금 다릅니다:

**C++ (기존)**
```
표면형,좌문맥ID,우문맥ID,비용,품사,...
```

**Rust (신규, 간소화 지원)**
```
표면형,품사,비용,기본형
```

둘 다 지원하므로 기존 파일도 사용 가능합니다.

## 빌드 시스템 마이그레이션

### Makefile에서 Cargo로

**Makefile (기존)**
```makefile
mecab: main.o
    g++ -o mecab main.o -lmecab
```

**Cargo.toml (신규)**
```toml
[dependencies]
mecab-ko = "0.1"
```

## Docker 이미지 마이그레이션

**Dockerfile (C++ 기존)**
```dockerfile
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y mecab mecab-ko-dic
```

**Dockerfile (Rust 신규)**
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
COPY --from=builder /app/target/release/mecab-ko /usr/local/bin/
```

## 단계별 마이그레이션

### 1단계: 병렬 실행

기존 C++ 버전과 Rust 버전을 동시에 실행하여 결과 비교:

```bash
# C++ 버전
echo "테스트" | mecab-ko-old > output_cpp.txt

# Rust 버전
echo "테스트" | mecab-ko > output_rust.txt

# 비교
diff output_cpp.txt output_rust.txt
```

### 2단계: 테스트 환경에서 검증

```python
# 기존 시스템 테스트
import MeCab_old
tagger_old = MeCab_old.Tagger()

# 새 시스템 테스트
from mecab_ko import Tagger
tagger_new = Tagger()

# 결과 비교
test_texts = ["문장1", "문장2", "문장3"]
for text in test_texts:
    result_old = tagger_old.parse(text)
    result_new = tagger_new.parse(text)
    assert result_old == result_new
```

### 3단계: 점진적 전환

1. 새로운 기능부터 Rust 버전 사용
2. 읽기 전용 작업을 Rust로 전환
3. 중요하지 않은 서비스 전환
4. 메인 서비스 전환

## 체크리스트

- [ ] 의존성 확인 (Rust 1.70.0 이상)
- [ ] 사전 파일 호환성 확인
- [ ] API 변경사항 파악
- [ ] 테스트 케이스 작성
- [ ] 성능 벤치마크
- [ ] 롤백 계획 수립
- [ ] 모니터링 설정
- [ ] 문서 업데이트

## 문제 해결

### "사전을 찾을 수 없음" 오류

```bash
# 사전 경로 확인
mecab-ko -d /path/to/dict --version

# 환경 변수 설정
export MECAB_DICT_DIR=/usr/local/lib/mecab/dic/mecab-ko-dic
```

### "성능 저하" 문제

```rust
// Release 빌드 사용
cargo build --release

// 병렬 처리 활성화
use rayon::prelude::*;
results = texts.par_iter().map(|t| tagger.parse(t)).collect();
```

## 지원

마이그레이션 관련 질문:
- GitHub Issues
- GitHub Discussions
