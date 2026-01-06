# mecab-ko-cli

한국어 형태소 분석기 CLI 도구

## 특징

- **명령줄 인터페이스**: 간편한 형태소 분석
- **다양한 출력 형식**: 기본, JSON, TSV 등
- **배치 처리**: 파일 단위 처리 지원
- **고성능**: Rust 기반 빠른 처리 속도

## 설치

```bash
cargo install mecab-ko-cli
```

## 사용법

```bash
# 기본 분석
echo "아버지가방에들어가신다" | mecab-ko

# JSON 출력
mecab-ko --format json input.txt

# 파일 처리
mecab-ko -o output.txt input.txt
```

## 옵션

```
-d, --dict <PATH>      사전 디렉토리 경로
-f, --format <FORMAT>  출력 형식 (default, json, tsv)
-o, --output <PATH>    출력 파일 경로
-h, --help             도움말 표시
```

## 라이선스

MIT 또는 Apache-2.0 중 선택
