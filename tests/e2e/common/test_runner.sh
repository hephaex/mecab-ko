#!/bin/bash
# Common test runner for all E2E tests

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
E2E_DIR="${PROJECT_ROOT}/tests/e2e"

# Test results
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Build Rust binaries
build_rust() {
    log_info "Building Rust binaries..."
    cd "${PROJECT_ROOT}/rust"

    # Build CLI
    cargo build --bin mecab-ko || {
        log_warn "Failed to build CLI"
        return 1
    }

    # Build Python binding
    cd crates/mecab-ko-python
    if command -v maturin &> /dev/null; then
        maturin develop || log_warn "Failed to build Python binding"
    else
        log_warn "maturin not found, skipping Python binding build"
    fi

    # Build Node.js binding
    cd "${PROJECT_ROOT}/rust/crates/mecab-ko-node"
    if [ -f "package.json" ]; then
        npm install || log_warn "Failed to install Node.js dependencies"
        npm run build || log_warn "Failed to build Node.js binding"
    fi

    # Build WASM
    cd "${PROJECT_ROOT}/rust/crates/mecab-ko-wasm"
    if command -v wasm-pack &> /dev/null; then
        wasm-pack build --target web || log_warn "Failed to build WASM binding"
    else
        log_warn "wasm-pack not found, skipping WASM build"
    fi

    cd "${PROJECT_ROOT}"
}

# Run CLI tests
run_cli_tests() {
    log_info "Running CLI E2E tests..."

    if ! command -v bats &> /dev/null; then
        log_warn "bats not found, skipping CLI tests"
        ((TESTS_SKIPPED++))
        return 0
    fi

    cd "${E2E_DIR}/cli"

    if bats test_cli_basic.bats test_cli_output_formats.bats; then
        ((TESTS_PASSED++))
        log_info "CLI tests passed"
    else
        ((TESTS_FAILED++))
        log_error "CLI tests failed"
    fi
}

# Run Python tests
run_python_tests() {
    log_info "Running Python E2E tests..."

    if ! command -v pytest &> /dev/null; then
        log_warn "pytest not found, skipping Python tests"
        ((TESTS_SKIPPED++))
        return 0
    fi

    cd "${E2E_DIR}/python"

    # Install dependencies if needed
    if [ -f "requirements.txt" ]; then
        pip install -q -r requirements.txt || log_warn "Failed to install Python dependencies"
    fi

    if pytest -v --tb=short; then
        ((TESTS_PASSED++))
        log_info "Python tests passed"
    else
        ((TESTS_FAILED++))
        log_error "Python tests failed"
    fi
}

# Run Node.js tests
run_nodejs_tests() {
    log_info "Running Node.js E2E tests..."

    if ! command -v npm &> /dev/null; then
        log_warn "npm not found, skipping Node.js tests"
        ((TESTS_SKIPPED++))
        return 0
    fi

    cd "${E2E_DIR}/nodejs"

    # Install dependencies
    npm install || {
        log_warn "Failed to install Node.js dependencies"
        ((TESTS_SKIPPED++))
        return 0
    }

    if npm test; then
        ((TESTS_PASSED++))
        log_info "Node.js tests passed"
    else
        ((TESTS_FAILED++))
        log_error "Node.js tests failed"
    fi
}

# Run WASM tests
run_wasm_tests() {
    log_info "Running WASM E2E tests..."

    if ! command -v npm &> /dev/null; then
        log_warn "npm not found, skipping WASM tests"
        ((TESTS_SKIPPED++))
        return 0
    fi

    cd "${E2E_DIR}/wasm"

    # Install dependencies
    npm install || {
        log_warn "Failed to install WASM test dependencies"
        ((TESTS_SKIPPED++))
        return 0
    }

    if npm test; then
        ((TESTS_PASSED++))
        log_info "WASM tests passed"
    else
        ((TESTS_FAILED++))
        log_error "WASM tests failed"
    fi
}

# Generate test report
generate_report() {
    log_info "Generating test report..."

    TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED + TESTS_SKIPPED))

    echo ""
    echo "================================"
    echo "E2E Test Results"
    echo "================================"
    echo "Total tests: ${TOTAL_TESTS}"
    echo -e "${GREEN}Passed: ${TESTS_PASSED}${NC}"
    echo -e "${RED}Failed: ${TESTS_FAILED}${NC}"
    echo -e "${YELLOW}Skipped: ${TESTS_SKIPPED}${NC}"
    echo "================================"

    if [ ${TESTS_FAILED} -gt 0 ]; then
        return 1
    fi
    return 0
}

# Main execution
main() {
    log_info "Starting E2E test suite..."

    # Parse arguments
    RUN_BUILD=true
    RUN_CLI=true
    RUN_PYTHON=true
    RUN_NODEJS=true
    RUN_WASM=true

    while [[ $# -gt 0 ]]; do
        case $1 in
            --no-build)
                RUN_BUILD=false
                shift
                ;;
            --cli-only)
                RUN_PYTHON=false
                RUN_NODEJS=false
                RUN_WASM=false
                shift
                ;;
            --python-only)
                RUN_CLI=false
                RUN_NODEJS=false
                RUN_WASM=false
                shift
                ;;
            --nodejs-only)
                RUN_CLI=false
                RUN_PYTHON=false
                RUN_WASM=false
                shift
                ;;
            --wasm-only)
                RUN_CLI=false
                RUN_PYTHON=false
                RUN_NODEJS=false
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    # Build if requested
    if [ "$RUN_BUILD" = true ]; then
        build_rust || log_warn "Build failed, continuing with tests..."
    fi

    # Run tests
    [ "$RUN_CLI" = true ] && run_cli_tests
    [ "$RUN_PYTHON" = true ] && run_python_tests
    [ "$RUN_NODEJS" = true ] && run_nodejs_tests
    [ "$RUN_WASM" = true ] && run_wasm_tests

    # Generate report
    generate_report
}

main "$@"
