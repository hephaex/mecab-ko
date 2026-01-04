# Legacy MeCab-Ko (C/C++)

> ⚠️ **레거시 코드**: 이 디렉토리는 기존 C/C++ 구현을 보존합니다.
> 새로운 개발은 `/rust/` 디렉토리에서 진행됩니다.

## 개요

mecab-ko는 은전한닢 프로젝트에서 사용하기 위한 MeCab의 fork 프로젝트입니다.
최소한의 변경으로 한국어의 특성에 맞는 기능을 추가하는 것이 목표입니다.

## 주요 기능

띄어쓰기를 하지 않는 일본어와 달리 띄어쓰기를 하는 한국어 특성에 맞게,
특정 품사가 띄어쓰기 되어있는 경우 해당 품사의 비용을 늘리는 기능을 제공합니다.

### 설정 예시

```text
# dicrc 설정
# 좌측에 공백을 포함하는 품사의 연접 비용 조정
left-space-penalty-factor = 120,6000,184,6000,100,500
```

## 설치 방법

```bash
tar zxfv mecab-ko-XX.tar.gz
cd mecab-ko-XX
./configure
make
make check
make install
```

## 디렉토리 구조

```
legacy/
├── src/              # MeCab 소스 코드
├── mecab-ko-dic/     # 한국어 사전
│   └── seed/         # 사전 원본 데이터 (CSV)
├── configure         # autotools 설정
├── Makefile          # 빌드 파일
└── README.md         # 이 파일
```

## 사전 데이터

`mecab-ko-dic/seed/` 디렉토리에는 다음과 같은 사전 파일들이 있습니다:

- `NNG.csv` - 일반명사
- `NNP.csv` - 고유명사
- `VV.csv` - 동사
- `VA.csv` - 형용사
- `EF.csv` - 어미
- `Wikipedia*.csv` - 위키피디아 추출 명사
- 기타 품사별 CSV 파일들

## 라이센스

MeCab의 라이센스를 그대로 따릅니다:
- GPL (GNU General Public License)
- LGPL (Lesser GNU General Public License)
- BSD License

## 마이그레이션

새로운 Rust 구현으로의 마이그레이션이 진행 중입니다.
자세한 내용은 `/rust/README.md`를 참조하세요.

### Rust v2의 장점

- 메모리 안전성 보장
- 현대적인 빌드 시스템 (Cargo)
- WASM 지원
- 통합된 Python/Node.js 바인딩
- 최신 사전 v3.0

## 관련 링크

- [프로젝트 메인](https://github.com/hephaex/mecab-ko)
- [Rust 구현](/rust/)
- [프로젝트 계획](/docs/PROJECT_PLAN.md)
