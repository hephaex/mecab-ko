#!/usr/bin/env bash
#
# Toggle between path and version dependencies
#
# 이 스크립트는 Cargo.toml의 의존성을 path ↔ version으로 전환합니다.
# - 개발 중: path 의존성 (로컬 변경사항 즉시 반영)
# - 배포 전: version 의존성 (crates.io 배포 필수)
#
# 사용법:
#   ./scripts/toggle-deps.sh <mode> [version]
#
# 모드:
#   path     - version을 path로 변경 (개발용)
#   version  - path를 version으로 변경 (배포용)
#
# 예제:
#   ./scripts/toggle-deps.sh version 0.1.0  # 배포 준비
#   ./scripts/toggle-deps.sh path           # 개발 모드로 복귀

set -euo pipefail

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-}"
VERSION="${2:-0.1.0}"

# 로깅 함수
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

# 도움말
show_help() {
    cat << EOF
Toggle between path and version dependencies

사용법:
    $0 <mode> [version]

모드:
    path        version을 path로 변경 (개발용)
    version     path를 version으로 변경 (배포용)

인자:
    version     버전 번호 (version 모드에서만 필요, 기본값: 0.1.0)

예제:
    # 배포 준비 (version 의존성으로 변경)
    $0 version 0.1.0

    # 개발 모드로 복귀 (path 의존성으로 변경)
    $0 path

EOF
}

# Cargo.toml에서 의존성을 path로 변경
to_path_deps() {
    local cargo_toml="$1"

    log_info "Path 의존성으로 변경: $cargo_toml"

    # mecab-ko-hangul
    sed -i 's/^mecab-ko-hangul = "[^"]*"/mecab-ko-hangul = { path = "..\/mecab-ko-hangul" }/g' "$cargo_toml"
    sed -i 's/^mecab-ko-hangul = { version = "[^"]*" }/mecab-ko-hangul = { path = "..\/mecab-ko-hangul" }/g' "$cargo_toml"

    # mecab-ko-dict
    sed -i 's/^mecab-ko-dict = "[^"]*"/mecab-ko-dict = { path = "..\/mecab-ko-dict" }/g' "$cargo_toml"
    sed -i 's/^mecab-ko-dict = { version = "[^"]*" }/mecab-ko-dict = { path = "..\/mecab-ko-dict" }/g' "$cargo_toml"

    # mecab-ko-core
    sed -i 's/^mecab-ko-core = "[^"]*"/mecab-ko-core = { path = "..\/mecab-ko-core" }/g' "$cargo_toml"
    sed -i 's/^mecab-ko-core = { version = "[^"]*" }/mecab-ko-core = { path = "..\/mecab-ko-core" }/g' "$cargo_toml"

    # mecab-ko-dict-builder (optional dependency)
    sed -i 's/^version = "[^"]*"$/path = "..\/mecab-ko-dict-builder"/g' "$cargo_toml"
}

# Cargo.toml에서 의존성을 version으로 변경
to_version_deps() {
    local cargo_toml="$1"
    local version="$2"

    log_info "Version 의존성으로 변경: $cargo_toml (v$version)"

    # mecab-ko-hangul
    sed -i "s/^mecab-ko-hangul = { path = \"[^\"]*\" }/mecab-ko-hangul = \"$version\"/g" "$cargo_toml"

    # mecab-ko-dict
    sed -i "s/^mecab-ko-dict = { path = \"[^\"]*\" }/mecab-ko-dict = \"$version\"/g" "$cargo_toml"

    # mecab-ko-core
    sed -i "s/^mecab-ko-core = { path = \"[^\"]*\" }/mecab-ko-core = \"$version\"/g" "$cargo_toml"

    # mecab-ko-dict-builder (optional dependency) - 더 정밀한 매칭
    # [dependencies.mecab-ko-dict-builder] 섹션 내의 path만 변경
    sed -i '/\[dependencies\.mecab-ko-dict-builder\]/,/^\[/ s/^path = "[^"]*"$/version = "'"$version"'"/' "$cargo_toml"
}

# 변경사항 확인
verify_changes() {
    local cargo_toml="$1"

    log_info "변경사항 확인: $cargo_toml"
    grep -E '^(mecab-ko-|path =|version =)' "$cargo_toml" | head -20 || true
}

# 메인 실행
main() {
    if [[ -z "$MODE" ]]; then
        log_error "모드를 지정해주세요."
        show_help
        exit 1
    fi

    if [[ "$MODE" != "path" && "$MODE" != "version" ]]; then
        log_error "잘못된 모드: $MODE"
        show_help
        exit 1
    fi

    if [[ "$MODE" == "version" && -z "$VERSION" ]]; then
        log_error "버전을 지정해주세요."
        show_help
        exit 1
    fi

    cd "$WORKSPACE_ROOT"

    log_info "========================================="
    log_info "의존성 전환 시작"
    log_info "모드: $MODE"
    if [[ "$MODE" == "version" ]]; then
        log_info "버전: $VERSION"
    fi
    log_info "========================================="

    # 크레이트 목록
    local crates=(
        "mecab-ko-dict"
        "mecab-ko-core"
        "mecab-ko-dict-builder"
        "mecab-ko-cli"
        "mecab-ko"
    )

    # 각 크레이트의 Cargo.toml 수정
    for crate in "${crates[@]}"; do
        local cargo_toml="$WORKSPACE_ROOT/crates/$crate/Cargo.toml"

        if [[ ! -f "$cargo_toml" ]]; then
            log_warning "파일이 없음: $cargo_toml"
            continue
        fi

        echo ""
        log_info "처리 중: $crate"

        # 백업 생성
        cp "$cargo_toml" "$cargo_toml.bak"

        if [[ "$MODE" == "path" ]]; then
            to_path_deps "$cargo_toml"
        else
            to_version_deps "$cargo_toml" "$VERSION"
        fi

        # 변경사항 확인
        verify_changes "$cargo_toml"

        log_success "완료: $crate"
    done

    echo ""
    log_info "========================================="
    log_success "모든 크레이트 처리 완료!"
    log_info "========================================="

    if [[ "$MODE" == "version" ]]; then
        log_info ""
        log_info "다음 단계:"
        log_info "  1. 변경사항 확인: git diff crates/*/Cargo.toml"
        log_info "  2. 빌드 테스트: cargo build --workspace"
        log_info "  3. 배포 실행: ./scripts/publish.sh --version $VERSION"
        log_info "  4. 배포 후: ./scripts/toggle-deps.sh path  # 개발 모드로 복귀"
    else
        log_info ""
        log_info "다음 단계:"
        log_info "  1. 변경사항 확인: git diff crates/*/Cargo.toml"
        log_info "  2. 빌드 테스트: cargo build --workspace"
    fi

    # 백업 파일 정리 여부 확인
    echo ""
    read -p "백업 파일(.bak)을 삭제하시겠습니까? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -f "$WORKSPACE_ROOT"/crates/*/Cargo.toml.bak
        log_success "백업 파일 삭제 완료"
    else
        log_info "백업 파일 유지: crates/*/Cargo.toml.bak"
    fi
}

main "$@"
