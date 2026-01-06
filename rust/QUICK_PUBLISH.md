# Quick Publishing Guide

배포를 위한 빠른 참조 가이드입니다.

## 한 줄 요약

```bash
./scripts/toggle-deps.sh version 0.1.0 && ./scripts/publish.sh --dry-run --version 0.1.0
```

## 배포 순서

1. **mecab-ko-hangul** (의존성 없음)
2. **mecab-ko-dict** (hangul 의존)
3. **mecab-ko-core** (dict, hangul 의존)
4. **mecab-ko-dict-builder** (dict 의존)
5. **mecab-ko-cli** (core 의존)
6. **mecab-ko** (모든 크레이트 의존)

## 배포 전 체크리스트

```bash
# 테스트
cargo test --workspace

# Clippy
cargo clippy --workspace --all-features -- -D warnings

# 포맷
cargo fmt --all --check

# 문서
cargo doc --workspace --no-deps
```

## 배포 명령

### 1단계: 준비
```bash
# 의존성 전환 (path → version)
./scripts/toggle-deps.sh version 0.1.0
```

### 2단계: 검증
```bash
# Dry-run 테스트
./scripts/publish.sh --dry-run --version 0.1.0
```

### 3단계: 배포
```bash
# 실제 배포 (신중하게!)
./scripts/publish.sh --version 0.1.0
```

### 4단계: 복원
```bash
# 개발 모드로 복귀
./scripts/toggle-deps.sh path

# Git 커밋
git add .
git commit -m "chore: restore path dependencies"
```

## Git 태그

```bash
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin main
git push origin v0.1.0
```

## 확인

- https://crates.io/crates/mecab-ko
- https://docs.rs/mecab-ko

## 도움말

```bash
./scripts/publish.sh --help
./scripts/toggle-deps.sh
```

## 상세 문서

- [PUBLISHING.md](PUBLISHING.md) - 종합 가이드
- [PUBLISHING_CHECKLIST.md](PUBLISHING_CHECKLIST.md) - 상세 체크리스트
- [QA-005-SUMMARY.md](QA-005-SUMMARY.md) - 작업 요약
