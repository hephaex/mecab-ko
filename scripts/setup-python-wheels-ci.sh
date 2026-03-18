#!/bin/bash

##############################################################################
# Python Wheels CI/CD Setup Script
#
# 이 스크립트는 Python Wheels 자동 빌드 및 배포를 위한
# 모든 전제 조건을 검증하고 설정합니다.
#
# 사용법:
#   bash scripts/setup-python-wheels-ci.sh
#
# 요구사항:
#   - Git
#   - Python 3.8+
#   - GitHub CLI (gh)
#
##############################################################################

set -euo pipefail

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 로깅 함수
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[!]${NC} $*"
}

log_error() {
    echo -e "${RED}[✗]${NC} $*"
}

log_section() {
    echo ""
    echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}$*${NC}"
    echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
}

# 확인 함수
confirm() {
    local prompt="$1"
    local response
    read -p "$(echo -e ${YELLOW}$prompt${NC}) (y/N) " response
    [[ "$response" =~ ^[Yy]$ ]]
}

##############################################################################
# 1. 사전 조건 검사
##############################################################################

check_prerequisites() {
    log_section "1. 사전 조건 검사"

    local all_good=true

    # Git 확인
    if command -v git &> /dev/null; then
        log_success "Git 설치됨"
    else
        log_error "Git이 설치되지 않았습니다"
        all_good=false
    fi

    # Python 확인
    if command -v python3 &> /dev/null; then
        local py_version=$(python3 --version | awk '{print $2}')
        log_success "Python $py_version 설치됨"
    else
        log_error "Python3가 설치되지 않았습니다"
        all_good=false
    fi

    # GitHub CLI 확인
    if command -v gh &> /dev/null; then
        log_success "GitHub CLI 설치됨"
    else
        log_error "GitHub CLI (gh)가 설치되지 않았습니다"
        log_info "설치: https://cli.github.com/"
        all_good=false
    fi

    # 저장소 확인
    if git rev-parse --git-dir > /dev/null 2>&1; then
        log_success "Git 저장소 감지됨"
    else
        log_error "Git 저장소가 아닙니다"
        all_good=false
    fi

    if [ "$all_good" = false ]; then
        log_error "사전 조건을 만족하지 않습니다"
        exit 1
    fi
}

##############################################################################
# 2. 워크플로우 파일 검사
##############################################################################

check_workflow_file() {
    log_section "2. 워크플로우 파일 검사"

    local workflow_file=".github/workflows/python-wheels.yml"

    if [ -f "$workflow_file" ]; then
        log_success "워크플로우 파일 존재: $workflow_file"

        # 주요 섹션 검증
        if grep -q "name: Python Wheels Build & Deploy" "$workflow_file"; then
            log_success "워크플로우 이름 확인됨"
        else
            log_error "워크플로우 이름이 올바르지 않음"
            return 1
        fi

        if grep -q "build_wheels:" "$workflow_file"; then
            log_success "build_wheels job 확인됨"
        else
            log_error "build_wheels job이 없음"
            return 1
        fi

        if grep -q "publish_to_pypi:" "$workflow_file"; then
            log_success "publish_to_pypi job 확인됨"
        else
            log_error "publish_to_pypi job이 없음"
            return 1
        fi
    else
        log_error "워크플로우 파일이 없음: $workflow_file"
        return 1
    fi
}

##############################################################################
# 3. 프로젝트 설정 검사
##############################################################################

check_project_config() {
    log_section "3. 프로젝트 설정 검사"

    local pyproject="rust/crates/mecab-ko-python/pyproject.toml"
    local cargo_toml="rust/crates/mecab-ko-python/Cargo.toml"

    # pyproject.toml 검사
    if [ -f "$pyproject" ]; then
        log_success "pyproject.toml 존재"

        if grep -q 'name = "mecab-ko-python"' "$pyproject"; then
            log_success "프로젝트 이름: mecab-ko-python"
        else
            log_error "프로젝트 이름이 올바르지 않음"
        fi

        if grep -q 'build-backend = "maturin"' "$pyproject"; then
            log_success "빌드 백엔드: maturin"
        else
            log_error "빌드 백엔드가 maturin이 아님"
        fi

        # Python 버전 확인
        local py_versions=$(grep 'Programming Language :: Python :: 3' "$pyproject" | wc -l)
        log_info "지원하는 Python 버전: $py_versions개"
    else
        log_error "pyproject.toml이 없음: $pyproject"
        return 1
    fi

    # Cargo.toml 검사
    if [ -f "$cargo_toml" ]; then
        log_success "Cargo.toml 존재"

        if grep -q 'crate-type = \["cdylib"\]' "$cargo_toml"; then
            log_success "라이브러리 타입: cdylib (Python extension)"
        fi
    else
        log_error "Cargo.toml이 없음: $cargo_toml"
        return 1
    fi
}

##############################################################################
# 4. 로컬 빌드 테스트
##############################################################################

test_local_build() {
    log_section "4. 로컬 빌드 테스트"

    if ! command -v maturin &> /dev/null; then
        log_warning "maturin이 설치되지 않았습니다"
        log_info "maturin 설치 중..."
        python3 -m pip install 'maturin[pyo3]' --quiet
    fi

    log_info "로컬 빌드 테스트 시작..."
    cd rust/crates/mecab-ko-python

    if maturin develop --quiet 2>/dev/null; then
        log_success "로컬 빌드 성공"

        # Import 테스트
        if python3 -c "import mecab_ko; print(f'Version: {mecab_ko.__version__}')" 2>/dev/null; then
            log_success "Import 테스트 성공"
        else
            log_error "Import 테스트 실패"
            cd - > /dev/null
            return 1
        fi
    else
        log_error "로컬 빌드 실패"
        log_info "상세 빌드 로그: maturin develop"
        cd - > /dev/null
        return 1
    fi

    cd - > /dev/null
}

##############################################################################
# 5. PyPI 설정 확인
##############################################################################

check_pypi_config() {
    log_section "5. PyPI 설정 확인"

    log_info "PyPI 프로젝트 확인:"
    echo ""
    echo "  프로젝트 URL: https://pypi.org/project/mecab-ko-python/"
    echo ""
    echo "  필요한 작업:"
    echo "    1. PyPI에 로그인"
    echo "    2. 프로젝트 settings > Publishing으로 이동"
    echo "    3. 'Add Trusted Publisher' 클릭"
    echo ""
    echo "  정보 입력:"
    echo "    - PyPI or TestPyPI: PyPI"
    echo "    - GitHub owner: $(git config --get remote.origin.url | grep -oP '(?<=/)[^/]*(?=/[^/]*$)')"
    echo "    - Repository: $(git config --get remote.origin.url | grep -oP '[^/]*(?=\.git$)')"
    echo "    - Workflow filename: python-wheels.yml"
    echo "    - Environment name: pypi"
    echo ""

    if confirm "PyPI에서 Trusted Publisher를 설정했나요?"; then
        log_success "PyPI Trusted Publisher 설정 확인됨"
    else
        log_warning "PyPI Trusted Publisher 설정이 필요합니다"
        log_info "자세한 설정 방법은 PYTHON_WHEELS_CI_CD.md를 참조하세요"
    fi
}

##############################################################################
# 6. GitHub Environment 설정 확인
##############################################################################

check_github_environment() {
    log_section "6. GitHub Environment 설정 확인"

    if ! command -v gh &> /dev/null; then
        log_warning "GitHub CLI가 설치되지 않아 자동 확인 불가능"
        log_info "수동 확인:"
        echo "  Repository Settings > Environments > pypi"
        echo "  - Environment name: pypi"
        echo "  - Deployment branches: main, releases/*"
        return
    fi

    # GitHub 로그인 확인
    if ! gh auth status &> /dev/null; then
        log_error "GitHub CLI가 인증되지 않았습니다"
        log_info "gh auth login으로 인증하세요"
        return 1
    fi

    # 저장소 정보 가져오기
    local repo=$(gh repo view --json nameWithOwner -q '.nameWithOwner')
    log_success "저장소: $repo"

    # Environment 확인
    if gh api "repos/$repo/environments/pypi" &> /dev/null; then
        log_success "GitHub Environment 'pypi' 존재"
    else
        log_warning "GitHub Environment 'pypi'가 없습니다"
        if confirm "GitHub Environment를 생성하시겠습니까?"; then
            log_info "GitHub 웹 UI에서 수동 생성해야 합니다:"
            echo "  Repository Settings > Environments > New environment"
            echo "  - Name: pypi"
            echo "  - Deployment branches: Only allow deployments to this repository from specified environments (선택사항)"
        fi
    fi
}

##############################################################################
# 7. 워크플로우 구문 검증
##############################################################################

validate_workflow_syntax() {
    log_section "7. 워크플로우 구문 검증"

    local workflow_file=".github/workflows/python-wheels.yml"

    if python3 -c "import yaml; yaml.safe_load(open('$workflow_file'))" 2>/dev/null; then
        log_success "YAML 구문 유효함"
    else
        log_error "YAML 구문 오류"
        log_info "다음 명령어로 문제 확인:"
        echo "  python3 -m yaml $workflow_file"
        return 1
    fi

    # 주요 필드 검증
    if grep -q "on:" "$workflow_file"; then
        log_success "'on' 필드 확인됨"
    fi

    if grep -q "jobs:" "$workflow_file"; then
        log_success "'jobs' 필드 확인됨"
    fi
}

##############################################################################
# 8. 설정 요약
##############################################################################

print_summary() {
    log_section "설정 완료 요약"

    echo ""
    echo -e "${GREEN}✓ 모든 전제 조건 충족됨${NC}"
    echo ""
    echo "다음 단계:"
    echo ""
    echo "1. PyPI에서 Trusted Publisher 설정 (아직 안 했다면):"
    echo "   https://pypi.org/project/mecab-ko-python/"
    echo "   Settings > Publishing > Add Trusted Publisher"
    echo ""
    echo "2. GitHub Environment 생성 (아직 안 했다면):"
    echo "   Repository Settings > Environments > New environment"
    echo "   Name: pypi"
    echo ""
    echo "3. 워크플로우 트리거:"
    echo "   a) Release 생성 시 자동 배포"
    echo "      gh release create v0.5.0"
    echo ""
    echo "   b) 수동 트리거:"
    echo "      gh workflow run python-wheels.yml -f publish_to_pypi=true"
    echo ""
    echo "4. 진행 상황 모니터링:"
    echo "   Actions > Python Wheels Build & Deploy"
    echo ""
    echo "5. 자세한 설정 방법:"
    echo "   PYTHON_WHEELS_CI_CD.md 참조"
    echo ""
}

##############################################################################
# 9. 테스트 워크플로우 실행
##############################################################################

offer_test_run() {
    log_section "9. 테스트 워크플로우 실행 (선택사항)"

    if ! command -v gh &> /dev/null; then
        log_info "GitHub CLI가 필요합니다"
        return
    fi

    if confirm "테스트용 워크플로우를 실행하시겠습니까? (빌드만 수행, 배포 안함)"; then
        log_info "워크플로우 실행 중..."
        gh workflow run python-wheels.yml

        sleep 2
        log_success "워크플로우가 대기열에 추가되었습니다"
        log_info "진행 상황 확인:"
        echo "  https://github.com/$(git config --get remote.origin.url | grep -oP '(?<=github.com[:/]).*(?=\.git)')/actions"
    fi
}

##############################################################################
# Main
##############################################################################

main() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║   Python Wheels CI/CD 설정 스크립트${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""

    check_prerequisites || exit 1
    check_workflow_file || exit 1
    check_project_config || exit 1
    test_local_build || exit 1
    check_pypi_config
    check_github_environment
    validate_workflow_syntax || exit 1

    print_summary
    offer_test_run

    echo ""
    echo -e "${GREEN}설정 스크립트 완료!${NC}"
    echo ""
}

main "$@"
