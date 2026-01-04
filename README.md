# 🇰🇷 MeCab-Ko

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![C++](https://img.shields.io/badge/c++-11-blue.svg)](https://isocpp.org/)
[![License](https://img.shields.io/badge/license-GPL%2FLGPL%2FBSD-green.svg)](LICENSE)

**한국어 형태소 분석기 - MeCab의 한국어 Fork**

> 은전한닢 프로젝트에서 시작된 MeCab-Ko를 현대화하고, Rust로 재구현하는 프로젝트입니다.

## 📖 프로젝트 개요

이 저장소는 두 가지 구현을 포함합니다:

| 구현 | 경로 | 상태 | 설명 |
|------|------|------|------|
| **Legacy (C/C++)** | `/legacy/` | ✅ 안정 | 기존 mecab-ko 구현 |
| **Rust (v2)** | `/rust/` | 🚧 개발중 | 현대적 Rust 재구현 |

### 왜 Rust로 재구현하는가?

| 기존 문제점 | Rust v2 해결책 |
|-------------|----------------|
| 오래된 사전 (2018년 이후 업데이트 없음) | 2024년 최신 말뭉치 기반 사전 v3.0 |
| C/C++ 메모리 안전성 이슈 | Rust의 메모리 안전성 보장 |
| 복잡한 빌드 (autotools) | Cargo 기반 간편한 빌드 |
| 플랫폼 제약 (WASM 미지원) | WASM, 다양한 플랫폼 지원 |
| 분리된 바인딩 프로젝트들 | 통합된 Python/Node.js 바인딩 |

## 🚀 빠른 시작

### Legacy (C/C++) 버전

```bash
cd legacy
./configure
make
make install

# 사전 설치
cd mecab-ko-dic
./configure
make
make install

# 실행
echo "안녕하세요" | mecab
```

### Rust v2 버전 (개발중)

```bash
cd rust

# 빌드
cargo build --release

# 테스트
cargo test

# 실행
cargo run --bin mecab-ko -- "안녕하세요"
```

#### Rust 라이브러리 사용

```toml
# Cargo.toml
[dependencies]
mecab-ko = "0.1"
```

```rust
use mecab_ko::Tokenizer;

fn main() {
    let tokenizer = Tokenizer::new().unwrap();
    let tokens = tokenizer.tokenize("안녕하세요, 형태소 분석기입니다.");
    
    for token in tokens {
        println!("{}\t{}", token.surface, token.pos);
    }
}
```

#### Python 바인딩

```bash
pip install mecab-ko-rs
```

```python
from mecab_ko import Mecab

mecab = Mecab()
print(mecab.morphs("안녕하세요"))  # ['안녕', '하', '세요']
print(mecab.nouns("형태소 분석기"))  # ['형태소', '분석기']
```

## 📂 저장소 구조

```
mecab-ko/
├── legacy/                     # 기존 C/C++ 구현
│   ├── src/                    # MeCab 소스 코드
│   ├── mecab-ko-dic/           # 한국어 사전
│   │   └── seed/               # 사전 원본 데이터
│   ├── configure               # autotools 빌드
│   └── Makefile
│
├── rust/                       # Rust v2 구현
│   ├── crates/
│   │   ├── mecab-ko-core/      # 핵심 분석 엔진
│   │   ├── mecab-ko-dict/      # 사전 관리
│   │   ├── mecab-ko-hangul/    # 한글 유틸리티
│   │   └── mecab-ko-cli/       # CLI 도구
│   ├── Cargo.toml              # Workspace 설정
│   └── README.md               # Rust 구현 상세
│
├── docs/                       # 프로젝트 문서
│   ├── PROJECT_PLAN.md         # 24주 로드맵
│   ├── ISSUE_BACKLOG.md        # 이슈 백로그
│   ├── AGENTS.md               # 멀티 에이전트 시스템
│   ├── DEVELOPMENT_WORKFLOW.md # 개발 워크플로우
│   └── AUTOMATION_GUIDE.md     # 자동화 가이드
│
├── .github/                    # GitHub 설정
│   ├── workflows/              # CI/CD
│   └── ISSUE_TEMPLATE/         # 이슈 템플릿
│
├── CONTRIBUTING.md             # 기여 가이드
├── SECURITY.md                 # 보안 정책
├── CODE_QUALITY.md             # 코드 품질 기준
└── README.md                   # 이 파일
```

## 📊 Rust v2 목표 성능

| 메트릭 | Legacy | Kiwi | **Rust v2** (목표) |
|--------|--------|------|-------------------|
| 속도 (어절/초) | ~100K | ~120K | **~150K** |
| 정확도 | ~93% | ~87% | **~95%** |
| 메모리 | ~200MB | ~100MB | **~150MB** |
| WASM 지원 | ❌ | ❌ | ✅ |

## 🗺️ 로드맵

### Phase 1: 기반 구축 (Q1 2025)
- [x] 프로젝트 설계 및 계획
- [ ] 한글 유틸리티 구현
- [ ] 사전 포맷 설계
- [ ] 기본 토크나이저

### Phase 2: 핵심 기능 (Q2 2025)
- [ ] Viterbi 알고리즘
- [ ] 사전 v3.0 빌드
- [ ] CLI 도구

### Phase 3: 생태계 통합 (Q3 2025)
- [ ] Python 바인딩 (PyO3)
- [ ] WASM 지원
- [ ] Elasticsearch 플러그인

### Phase 4: 안정화 (Q4 2025)
- [ ] 성능 최적화
- [ ] 문서화
- [ ] v1.0 릴리스

자세한 계획은 [PROJECT_PLAN.md](docs/PROJECT_PLAN.md)를 참조하세요.

## 🤝 기여하기

기여를 환영합니다! [CONTRIBUTING.md](CONTRIBUTING.md)를 참조해주세요.

### 개발 환경 설정

```bash
# 저장소 클론
git clone https://github.com/hephaex/mecab-ko.git
cd mecab-ko

# Rust 개발
cd rust
cargo build
cargo test

# Legacy 빌드 (선택)
cd ../legacy
./configure && make
```

## 📜 라이센스

- **Legacy (C/C++)**: GPL / LGPL / BSD (MeCab 원본 라이센스)
- **Rust v2**: MIT OR Apache-2.0

사전 데이터는 [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)을 따릅니다.

## 🙏 감사의 말

- [MeCab](https://taku910.github.io/mecab/) - Taku Kudo
- [은전한닢 프로젝트](https://bitbucket.org/eunjeon/mecab-ko) - mecab-ko 원본
- [Lindera](https://github.com/lindera/lindera) - Rust 형태소 분석기 참조
- [Kiwi](https://github.com/bab2min/Kiwi) - 한국어 형태소 분석기 참조

## 📞 연락처

- **Author**: hephaex (hephaex@gmail.com)
- **Issues**: [GitHub Issues](https://github.com/hephaex/mecab-ko/issues)
- **Discussions**: [GitHub Discussions](https://github.com/hephaex/mecab-ko/discussions)

---

<p align="center">
  Made with ❤️ for Korean NLP
</p>
