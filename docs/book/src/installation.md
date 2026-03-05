# 설치

MeCab-Ko는 여러 가지 방법으로 설치할 수 있습니다. 사용 목적에 따라 적절한 방법을 선택하세요.

## 요구 사항

### 시스템 요구 사항

- **운영체제**: Linux, macOS, Windows 10+
- **Rust**: 1.75.0 이상 (라이브러리 사용 시)
- **메모리**: 최소 256MB (사전 로딩 포함)
- **Python**: 3.8+ (Python 바인딩 사용 시)
- **Node.js**: 18+ (Node.js 바인딩 사용 시)

### Rust 설치

Rust가 설치되어 있지 않다면 [rustup](https://rustup.rs/)을 사용하여 설치하세요:

```bash
# Unix-like systems (Linux, macOS)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows (PowerShell)
# Visit https://rustup.rs/ and download rustup-init.exe
```

설치 확인:

```bash
rustc --version
# rustc 1.70.0 (or later)

cargo --version
# cargo 1.70.0 (or later)
```

## CLI 도구 설치

### Cargo를 통한 설치

가장 간단한 방법은 Cargo를 사용하는 것입니다:

```bash
cargo install mecab-ko-cli
```

설치 후 사용:

```bash
mecab-ko --version
mecab-ko "안녕하세요"
```

### 소스에서 빌드

최신 개발 버전을 사용하려면 소스에서 직접 빌드합니다:

```bash
# Clone the repository
git clone https://github.com/hephaex/mecab-ko.git
cd mecab-ko/rust

# Build in release mode
cargo build --release

# The binary is located at target/release/mecab-ko
./target/release/mecab-ko --version
```

시스템 경로에 설치:

```bash
# Option 1: Copy to a directory in PATH
sudo cp target/release/mecab-ko /usr/local/bin/

# Option 2: Add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

## 라이브러리로 사용

### crates.io에서 설치 (권장) 🎉

Rust 프로젝트에서 라이브러리로 사용하려면 `Cargo.toml`에 의존성을 추가하세요:

```toml
[dependencies]
mecab-ko = "0.4"
```

또는 cargo add 명령어를 사용합니다:

```bash
cargo add mecab-ko
```

특정 기능만 사용할 경우:

```toml
[dependencies]
# Core library only (Viterbi, Lattice, Sejong converter)
mecab-ko-core = "0.4"

# Hangul utilities only (자모 분리/결합)
mecab-ko-hangul = "0.4"

# Dictionary management only (사전 로딩)
mecab-ko-dict = "0.4"

# Dictionary builder (사전 빌드 도구)
mecab-ko-dict-builder = "0.4"

# Dictionary validator (사전 검증 도구)
mecab-ko-dict-validator = "0.4"
```

### Feature Flags

`mecab-ko` 크레이트는 다음 feature를 제공합니다:

```toml
[dependencies]
mecab-ko = { version = "0.4", features = ["builder", "serde"] }
```

| Feature | 설명 |
|---------|------|
| `builder` | 사전 빌더 기능 포함 |
| `serde` | JSON 직렬화 지원 |
| `rayon` | 병렬 처리 지원 |
| `zstd` | 사전 압축 지원 (기본 활성화) |

## 사전 설치

MeCab-Ko는 형태소 분석을 위해 사전이 필요합니다.

### 기본 사전

기본 사전은 라이브러리에 포함되어 있습니다. 별도 설치가 필요하지 않습니다.

### 커스텀 사전 경로

시스템에 설치된 mecab-ko-dic을 사용하려면:

```bash
# Using custom dictionary path
mecab-ko -d /path/to/mecab-ko-dic "분석할 텍스트"
```

일반적인 사전 경로:

| 운영체제 | 경로 |
|---------|------|
| Linux | `/usr/share/mecab-ko-dic` |
| macOS (Homebrew) | `/opt/homebrew/lib/mecab/dic/mecab-ko-dic` |
| Windows | `C:\Program Files\MeCab\dic\mecab-ko-dic` |

### 사용자 사전

사용자 정의 단어를 추가하려면 CSV 형식의 사용자 사전을 생성합니다:

```bash
# Create user dictionary file
cat > user.csv << EOF
딥러닝,NNG,-1000,딥러닝
머신러닝,NNG,-1000,머신러닝
앤트로픽,NNP,-1000,앤트로픽
EOF

# Use with --user-dic option
mecab-ko --user-dic user.csv "딥러닝과 머신러닝"
```

자세한 내용은 [사용자 사전](user-dictionary.md) 장을 참조하세요.

## 설치 확인

설치가 완료되면 다음 명령으로 확인합니다:

```bash
# Check version
mecab-ko --version

# Simple test
echo "안녕하세요" | mecab-ko
```

예상 출력:

```
안녕    NNG
하      XSV
세요    EP+EF
EOS
```

## 문제 해결

### 빌드 오류

Rust 버전이 오래된 경우:

```bash
rustup update stable
```

### 사전을 찾을 수 없음

사전 경로를 명시적으로 지정하세요:

```bash
mecab-ko -d /path/to/dict "텍스트"
```

### 권한 문제 (Linux/macOS)

```bash
# Make binary executable
chmod +x target/release/mecab-ko
```

## 다음 단계

설치를 완료했다면 [빠른 시작](quick-start.md)을 확인하세요.
