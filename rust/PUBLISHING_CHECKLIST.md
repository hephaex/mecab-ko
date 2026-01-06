# Publishing Checklist for crates.io

이 문서는 crates.io 배포 전 필수 작업 목록입니다.

## 배포 전 필수 작업

### 1. Cargo.toml 의존성 수정

현재 모든 크레이트가 `path` 의존성을 사용하고 있습니다. 배포 전에 이를 `version`으로 변경해야 합니다.

**주의**: 실제 배포 시에만 변경하고, 개발 중에는 path를 사용하세요.

#### mecab-ko-dict/Cargo.toml
```toml
# 변경 전
mecab-ko-hangul = { path = "../mecab-ko-hangul" }

# 변경 후
mecab-ko-hangul = "0.1.0"
```

#### mecab-ko-core/Cargo.toml
```toml
# 변경 전
mecab-ko-hangul = { path = "../mecab-ko-hangul" }
mecab-ko-dict = { path = "../mecab-ko-dict" }

# 변경 후
mecab-ko-hangul = "0.1.0"
mecab-ko-dict = "0.1.0"
```

#### mecab-ko-dict-builder/Cargo.toml
```toml
# 변경 전
mecab-ko-hangul = { path = "../mecab-ko-hangul" }
mecab-ko-dict = { path = "../mecab-ko-dict" }

# 변경 후
mecab-ko-hangul = "0.1.0"
mecab-ko-dict = "0.1.0"
```

#### mecab-ko-cli/Cargo.toml
```toml
# 변경 전
mecab-ko-core = { path = "../mecab-ko-core" }
mecab-ko-dict = { path = "../mecab-ko-dict" }

# 변경 후
mecab-ko-core = "0.1.0"
mecab-ko-dict = "0.1.0"
```

#### mecab-ko/Cargo.toml
```toml
# 변경 전
mecab-ko-core = { path = "../mecab-ko-core" }
mecab-ko-dict = { path = "../mecab-ko-dict" }
mecab-ko-hangul = { path = "../mecab-ko-hangul" }

[dependencies.mecab-ko-dict-builder]
path = "../mecab-ko-dict-builder"
optional = true

# 변경 후
mecab-ko-core = "0.1.0"
mecab-ko-dict = "0.1.0"
mecab-ko-hangul = "0.1.0"

[dependencies.mecab-ko-dict-builder]
version = "0.1.0"
optional = true
```

### 2. Git 커밋 및 태그

```bash
# 모든 변경사항 커밋
git add .
git commit -m "chore: prepare for v0.1.0 release"

# 태그 생성
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin main
git push origin v0.1.0
```

### 3. 품질 검증

각 크레이트에서 다음을 확인:

```bash
cd crates/mecab-ko-hangul
cargo test --all-features
cargo clippy --all-features -- -D warnings
cargo fmt --check
cargo doc --no-deps

# 다른 크레이트도 동일하게
```

### 4. crates.io 인증 설정

```bash
# https://crates.io/settings/tokens 에서 토큰 생성
cargo login
# 토큰 입력
```

## 배포 실행

### 방법 1: 자동 스크립트 사용 (권장)

```bash
# Dry-run으로 먼저 테스트
./scripts/publish.sh --dry-run --version 0.1.0

# 문제없으면 실제 배포
./scripts/publish.sh --version 0.1.0
```

### 방법 2: 수동 배포

```bash
# 1. mecab-ko-hangul
cd crates/mecab-ko-hangul
cargo publish --dry-run  # 테스트
cargo publish            # 실제 배포
sleep 30                 # crates.io 인덱스 업데이트 대기

# 2. mecab-ko-dict
cd ../mecab-ko-dict
cargo publish --dry-run
cargo publish
sleep 30

# 3. mecab-ko-core
cd ../mecab-ko-core
cargo publish --dry-run
cargo publish
sleep 30

# 4. mecab-ko-dict-builder
cd ../mecab-ko-dict-builder
cargo publish --dry-run
cargo publish
sleep 30

# 5. mecab-ko-cli
cd ../mecab-ko-cli
cargo publish --dry-run
cargo publish
sleep 30

# 6. mecab-ko
cd ../mecab-ko
cargo publish --dry-run
cargo publish
```

## 배포 후 확인

### 1. crates.io 확인
- https://crates.io/crates/mecab-ko-hangul
- https://crates.io/crates/mecab-ko-dict
- https://crates.io/crates/mecab-ko-core
- https://crates.io/crates/mecab-ko-dict-builder
- https://crates.io/crates/mecab-ko-cli
- https://crates.io/crates/mecab-ko

### 2. docs.rs 확인
- https://docs.rs/mecab-ko-hangul
- https://docs.rs/mecab-ko-dict
- https://docs.rs/mecab-ko-core
- https://docs.rs/mecab-ko-dict-builder
- https://docs.rs/mecab-ko-cli
- https://docs.rs/mecab-ko

### 3. 설치 테스트

```bash
# 새 프로젝트에서 테스트
cargo new test-mecab-ko
cd test-mecab-ko

# Cargo.toml에 추가
# [dependencies]
# mecab-ko = "0.1.0"

cargo build
cargo test
```

### 4. 문서 업데이트

- README.md에 설치 방법 추가
- GitHub Release 생성
- 프로젝트 홈페이지 업데이트 (있는 경우)

## 배포 후 path 의존성 복원

배포가 완료되면 개발 편의를 위해 Cargo.toml의 의존성을 다시 path로 변경:

```bash
# 또는 git으로 원래대로 복원
git checkout crates/*/Cargo.toml

# 커밋
git commit -m "chore: restore path dependencies for development"
git push
```

## 문제 해결

### 배포 실패 시

1. **에러 확인**: 에러 메시지를 자세히 읽고 원인 파악
2. **수정**: 문제를 수정하고 다시 dry-run
3. **재배포**: 이미 배포된 크레이트는 건너뛰고 실패한 크레이트부터 재시작

### 버전 충돌 시

crates.io에서는 동일 버전을 다시 배포할 수 없습니다. 버전을 올려야 합니다:

```toml
# Cargo.toml
[workspace.package]
version = "0.1.1"  # 0.1.0 -> 0.1.1로 증가
```

## 참고

자세한 내용은 [PUBLISHING.md](PUBLISHING.md)를 참조하세요.
