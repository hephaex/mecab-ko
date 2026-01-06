# FAQ

자주 묻는 질문과 답변을 모았습니다.

## 일반

### MeCab-Ko와 은전한닢(mecab-ko)의 차이점은?

| 항목 | 은전한닢 (mecab-ko) | MeCab-Ko (이 프로젝트) |
|------|---------------------|------------------------|
| 언어 | C++ | Rust |
| 메모리 안전성 | unsafe 코드 포함 | unsafe 없음 |
| 유지보수 | 2018년 이후 중단 | 활발히 개발 중 |
| 크로스 플랫폼 | 빌드 복잡 | Cargo로 간편 빌드 |
| WASM 지원 | 불가 | 지원 예정 |

### Kiwi나 Lindera와의 차이점은?

- **Kiwi**: C++ 기반, 독자적인 알고리즘, 높은 정확도
- **Lindera**: Rust 기반, 일본어 중심, 한국어 부분 지원
- **MeCab-Ko**: Rust 기반, 한국어 특화, mecab-ko 호환

### 라이선스는?

Apache 2.0 또는 MIT 중 선택하여 사용할 수 있습니다. 상업적 사용이 가능합니다.

## 설치

### Rust가 설치되어 있지 않아도 사용할 수 있나요?

CLI 도구의 경우, 미리 빌드된 바이너리를 사용하면 Rust 없이도 사용 가능합니다. (배포 예정)

라이브러리로 사용하려면 Rust가 필요합니다.

### Windows에서 빌드 오류가 발생합니다

Visual Studio Build Tools가 설치되어 있는지 확인하세요:

```powershell
# Install via rustup
rustup default stable-msvc
```

### macOS에서 `xcrun: error` 오류

Xcode Command Line Tools를 설치하세요:

```bash
xcode-select --install
```

## 사용법

### 분석 결과가 이상합니다

1. **사전 경로 확인**: 올바른 사전이 로드되었는지 확인
2. **사용자 사전**: 도메인 특화 단어는 사용자 사전에 추가
3. **인코딩**: 입력 텍스트가 UTF-8인지 확인

```bash
# Check dictionary info
mecab-ko dict

# Use with verbose output
mecab-ko -O dump "문제되는 텍스트"
```

### 신조어가 제대로 분석되지 않습니다

사용자 사전에 추가하세요:

```csv
# user.csv
새로운단어,NNG,-1000,
```

```bash
mecab-ko --user-dic user.csv "새로운단어를 분석합니다"
```

### 메모리 사용량이 높습니다

사전 로딩 시 약 150-200MB의 메모리가 필요합니다. 이는 정상적인 동작입니다.

메모리를 줄이려면:
- 필요한 기능만 포함한 경량 사전 사용 (개발 중)
- 서버 환경에서 싱글톤 패턴으로 토크나이저 공유

### 처리 속도가 느립니다

1. **릴리스 빌드 사용**:
   ```bash
   cargo build --release
   ```

2. **배치 처리**: 한 번에 여러 문장 처리
3. **토크나이저 재사용**: 매번 생성하지 않기

```rust
// Good: Reuse tokenizer
let tokenizer = Tokenizer::new()?;
for text in texts {
    let tokens = tokenizer.tokenize(text);
}

// Bad: Create new tokenizer each time
for text in texts {
    let tokenizer = Tokenizer::new()?;  // Slow!
    let tokens = tokenizer.tokenize(text);
}
```

## 사용자 사전

### CSV 파일 형식이 맞는지 확인하려면?

```bash
# Check if file is valid UTF-8
file user.csv

# Test loading
mecab-ko -q --user-dic user.csv "테스트" 2>&1
```

### 우선순위 조정은 어떻게 하나요?

비용(cost) 값을 조정합니다. 낮을수록 높은 우선순위:

```csv
최우선단어,NNG,-2000,
높은우선순위,NNG,-1000,
보통,NNG,-500,
```

### 동음이의어 처리는?

같은 표면형에 다른 품사를 등록할 수 있습니다:

```csv
# '배'의 여러 의미
배,NNG,-1000,    # 과일
배,NNG,-1000,    # 신체 부위
배,NNG,-1000,    # 선박
```

분석기는 문맥에 따라 적절한 분석을 선택합니다.

## 프로그래밍

### 병렬 처리가 가능한가요?

`Tokenizer`는 `Send + Sync`를 구현하므로 스레드 간 공유가 가능합니다:

```rust
use std::sync::Arc;
use rayon::prelude::*;

let tokenizer = Arc::new(Tokenizer::new()?);

let results: Vec<_> = texts
    .par_iter()
    .map(|text| tokenizer.tokenize(text))
    .collect();
```

### async/await와 함께 사용할 수 있나요?

현재 토크나이저는 동기 API만 제공합니다. 비동기 환경에서는 `spawn_blocking` 사용:

```rust
use tokio::task::spawn_blocking;

let tokens = spawn_blocking(move || {
    tokenizer.tokenize(text)
}).await?;
```

### Python에서 사용할 수 있나요?

Python 바인딩은 개발 예정입니다. 현재는 subprocess로 CLI 호출:

```python
import subprocess
import json

result = subprocess.run(
    ["mecab-ko", "-O", "json", "안녕하세요"],
    capture_output=True,
    text=True
)
tokens = json.loads(result.stdout)
```

### 웹 브라우저에서 사용할 수 있나요?

WASM 지원은 개발 예정입니다.

## 오류 해결

### `Dictionary error: IO error`

사전 파일 경로가 잘못되었거나 파일이 없습니다:

```bash
# Check if path exists
ls -la /path/to/dict

# Use explicit path
mecab-ko -d /correct/path/to/dict "테스트"
```

### `Invalid dictionary format`

사전 파일이 손상되었거나 버전이 맞지 않습니다:

1. 사전 재다운로드
2. 바이너리 사전 버전 확인
3. 소스에서 사전 재빌드

### `Version mismatch`

사전 버전과 프로그램 버전이 맞지 않습니다:

```bash
# Check versions
mecab-ko version
mecab-ko dict /path/to/dict
```

호환되는 버전의 사전을 사용하세요.

### `Failed to initialize tokenizer`

1. 기본 사전이 포함된 빌드인지 확인
2. 환경 변수 확인: `MECAB_KO_DIC_PATH`
3. 명시적 사전 경로 지정

## 기여

### 버그를 발견했습니다

GitHub Issues에 보고해 주세요:
- https://github.com/hephaex/mecab-ko/issues

포함할 정보:
- MeCab-Ko 버전
- 운영체제
- 재현 단계
- 입력 텍스트 (가능한 경우)
- 예상 결과 vs 실제 결과

### 기능 제안을 하고 싶습니다

GitHub Issues에 Feature Request로 등록해 주세요.

### 코드 기여는 어떻게 하나요?

1. 저장소 Fork
2. Feature 브랜치 생성
3. 변경 사항 커밋
4. Pull Request 제출

자세한 내용은 CONTRIBUTING.md를 참조하세요.

## 추가 질문

이 FAQ에서 답을 찾지 못했다면:

1. **GitHub Discussions**: https://github.com/hephaex/mecab-ko/discussions
2. **이슈 등록**: https://github.com/hephaex/mecab-ko/issues
3. **문서 검색**: https://docs.rs/mecab-ko
