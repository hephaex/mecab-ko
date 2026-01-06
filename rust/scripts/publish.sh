#!/usr/bin/env bash
#
# MeCab-Ko Crates Publishing Script
#
# 이 스크립트는 mecab-ko 관련 크레이트들을 올바른 의존성 순서로 crates.io에 배포합니다.
#
# 사용법:
#   ./scripts/publish.sh [OPTIONS]
#
# 옵션:
#   --dry-run       실제 배포 없이 테스트만 수행
#   --version VER   배포할 버전 지정 (예: 0.1.0)
#   --skip-tests    테스트 건너뛰기 (권장하지 않음)
#   --help          도움말 표시

set -euo pipefail

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 기본 설정
DRY_RUN=false
SKIP_TESTS=false
VERSION=""
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# 배포 순서 (의존성 순서)
CRATES=(
    "mecab-ko-hangul"
    "mecab-ko-dict"
    "mecab-ko-core"
    "mecab-ko-dict-builder"
    "mecab-ko-cli"
    "mecab-ko"
)

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

# 도움말 표시
show_help() {
    cat << EOF
MeCab-Ko Crates Publishing Script

사용법:
    $0 [OPTIONS]

옵션:
    --dry-run           실제 배포 없이 테스트만 수행
    --version VERSION   배포할 버전 지정 (예: 0.1.0)
    --skip-tests        테스트 건너뛰기 (권장하지 않음)
    --help              이 도움말 표시

예제:
    # Dry-run으로 테스트
    $0 --dry-run

    # 버전 0.1.0으로 배포
    $0 --version 0.1.0

    # Dry-run + 버전 지정
    $0 --dry-run --version 0.1.0

배포 순서:
EOF
    for i in "${!CRATES[@]}"; do
        echo "    $((i+1)). ${CRATES[$i]}"
    done
}

# 명령행 인자 파싱
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --version)
                VERSION="$2"
                shift 2
                ;;
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                log_error "알 수 없는 옵션: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# 버전 확인
check_version() {
    local crate_dir="$1"
    local expected_version="$2"

    if [[ -z "$expected_version" ]]; then
        return 0
    fi

    cd "$crate_dir"
    local actual_version
    actual_version=$(cargo metadata --no-deps --format-version 1 | \
        grep -o '"version":"[^"]*"' | head -1 | cut -d'"' -f4)

    if [[ "$actual_version" != "$expected_version" ]]; then
        log_error "버전 불일치: $crate_dir"
        log_error "  예상: $expected_version"
        log_error "  실제: $actual_version"
        return 1
    fi

    log_info "버전 확인: $actual_version"
    return 0
}

# 의존성 버전 확인
check_dependencies() {
    local crate_name="$1"
    local crate_dir="$WORKSPACE_ROOT/crates/$crate_name"

    cd "$crate_dir"

    # Cargo.toml에서 path 의존성 확인
    if grep -q 'path = "\.\.' Cargo.toml; then
        log_warning "$crate_name: path 의존성 발견. 배포 전에 version으로 변경 필요."
        log_warning "  현재 Cargo.toml 확인:"
        grep -A1 'mecab-ko' Cargo.toml | grep -v '^--$' || true
        return 1
    fi

    return 0
}

# 크레이트 테스트
run_tests() {
    local crate_name="$1"
    local crate_dir="$WORKSPACE_ROOT/crates/$crate_name"

    if [[ "$SKIP_TESTS" == "true" ]]; then
        log_warning "테스트 건너뛰기: $crate_name"
        return 0
    fi

    cd "$crate_dir"
    log_info "테스트 실행: $crate_name"

    # 테스트
    if ! cargo test --all-features; then
        log_error "테스트 실패: $crate_name"
        return 1
    fi

    # Clippy
    if ! cargo clippy --all-features -- -D warnings; then
        log_error "Clippy 실패: $crate_name"
        return 1
    fi

    # 포맷 체크
    if ! cargo fmt --check; then
        log_error "포맷 체크 실패: $crate_name"
        log_info "다음 명령으로 수정 가능: cargo fmt"
        return 1
    fi

    log_success "테스트 통과: $crate_name"
    return 0
}

# 문서 빌드 확인
check_docs() {
    local crate_name="$1"
    local crate_dir="$WORKSPACE_ROOT/crates/$crate_name"

    cd "$crate_dir"
    log_info "문서 빌드 확인: $crate_name"

    if ! cargo doc --no-deps; then
        log_error "문서 빌드 실패: $crate_name"
        return 1
    fi

    log_success "문서 빌드 성공: $crate_name"
    return 0
}

# 패키지 검증
verify_package() {
    local crate_name="$1"
    local crate_dir="$WORKSPACE_ROOT/crates/$crate_name"

    cd "$crate_dir"
    log_info "패키지 검증: $crate_name"

    # 패키징 옵션 (dirty flag는 DRY_RUN일 때만)
    local package_flags=""
    if [[ "$DRY_RUN" == "true" ]]; then
        package_flags="--allow-dirty"
    fi

    # 패키징 테스트
    if ! cargo package --list $package_flags > /dev/null; then
        log_error "패키징 실패: $crate_name"
        return 1
    fi

    # 패키지 크기 확인
    local package_size
    package_size=$(cargo package --list $package_flags | wc -l)
    log_info "패키지 파일 수: $package_size"

    # Dry-run publish
    if ! cargo publish --dry-run $package_flags; then
        log_error "Dry-run publish 실패: $crate_name"
        return 1
    fi

    log_success "패키지 검증 통과: $crate_name"
    return 0
}

# 크레이트 배포
publish_crate() {
    local crate_name="$1"
    local crate_dir="$WORKSPACE_ROOT/crates/$crate_name"

    log_info "========================================="
    log_info "배포 시작: $crate_name"
    log_info "========================================="

    cd "$crate_dir"

    # 버전 확인
    if ! check_version "$crate_dir" "$VERSION"; then
        return 1
    fi

    # 의존성 확인 (path 대신 version 사용 확인)
    if ! check_dependencies "$crate_name"; then
        log_error "의존성 문제 발견. 수정 후 다시 시도하세요."
        return 1
    fi

    # 테스트 실행
    if ! run_tests "$crate_name"; then
        return 1
    fi

    # 문서 확인
    if ! check_docs "$crate_name"; then
        return 1
    fi

    # 패키지 검증
    if ! verify_package "$crate_name"; then
        return 1
    fi

    # 실제 배포
    if [[ "$DRY_RUN" == "false" ]]; then
        log_info "실제 배포 중: $crate_name"
        if ! cargo publish; then
            log_error "배포 실패: $crate_name"
            return 1
        fi

        log_success "배포 완료: $crate_name"

        # 배포 후 대기 (crates.io 인덱스 업데이트 대기)
        log_info "crates.io 인덱스 업데이트 대기 (30초)..."
        sleep 30
    else
        log_info "DRY-RUN 모드: 실제 배포 건너뛰기"
    fi

    log_success "========================================="
    return 0
}

# 메인 실행
main() {
    parse_args "$@"

    log_info "MeCab-Ko Crates Publishing Script"
    log_info "Workspace: $WORKSPACE_ROOT"

    if [[ "$DRY_RUN" == "true" ]]; then
        log_warning "DRY-RUN 모드: 실제 배포하지 않습니다"
    fi

    if [[ -n "$VERSION" ]]; then
        log_info "배포 버전: $VERSION"
    fi

    # workspace 루트로 이동
    cd "$WORKSPACE_ROOT"

    # Git 상태 확인
    if [[ -d .git ]]; then
        if [[ -n "$(git status --porcelain)" ]]; then
            log_warning "Git working directory가 깨끗하지 않습니다."
            log_warning "커밋되지 않은 변경사항이 있습니다."
            read -p "계속하시겠습니까? (y/N): " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_info "배포 취소"
                exit 0
            fi
        fi
    fi

    # 각 크레이트 순서대로 배포
    local failed_crates=()

    for crate_name in "${CRATES[@]}"; do
        if ! publish_crate "$crate_name"; then
            log_error "크레이트 배포 실패: $crate_name"
            failed_crates+=("$crate_name")

            # 실패 시 중단
            log_error "배포 프로세스 중단"
            break
        fi

        echo ""
    done

    # 결과 요약
    echo ""
    log_info "========================================="
    log_info "배포 결과 요약"
    log_info "========================================="

    if [[ ${#failed_crates[@]} -eq 0 ]]; then
        log_success "모든 크레이트 배포 성공!"

        if [[ "$DRY_RUN" == "false" ]]; then
            log_info ""
            log_info "다음 단계:"
            log_info "  1. crates.io에서 배포 확인"
            log_info "  2. docs.rs에서 문서 확인"
            log_info "  3. Git 태그 생성: git tag -a v${VERSION} -m 'Release v${VERSION}'"
            log_info "  4. 태그 푸시: git push origin v${VERSION}"
        fi
    else
        log_error "다음 크레이트 배포 실패:"
        for crate in "${failed_crates[@]}"; do
            log_error "  - $crate"
        done
        exit 1
    fi
}

# 스크립트 실행
main "$@"
