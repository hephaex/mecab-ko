#!/bin/bash
# Test runner script for MeCab-Ko

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== MeCab-Ko Test Suite ===${NC}\n"

# Change to project root
cd "$(dirname "$0")/.."

# Function to run a command and report status
run_test() {
    local name="$1"
    local command="$2"

    echo -e "${YELLOW}Running: ${name}${NC}"
    if eval "$command"; then
        echo -e "${GREEN}✓ ${name} passed${NC}\n"
        return 0
    else
        echo -e "${RED}✗ ${name} failed${NC}\n"
        return 1
    fi
}

# Track failures
FAILED=0

# 1. Format check
if ! run_test "Format check" "cargo fmt --all -- --check"; then
    FAILED=$((FAILED + 1))
    echo -e "${YELLOW}Tip: Run 'cargo fmt --all' to fix formatting${NC}\n"
fi

# 2. Clippy lints
if ! run_test "Clippy lints" "cargo clippy --all-targets --all-features -- -D warnings"; then
    FAILED=$((FAILED + 1))
fi

# 3. Unit tests
if ! run_test "Unit tests" "cargo test --lib --all-features"; then
    FAILED=$((FAILED + 1))
fi

# 4. Integration tests (fast)
if ! run_test "Integration tests (fast)" "cargo test --tests"; then
    FAILED=$((FAILED + 1))
fi

# 5. Doc tests
if ! run_test "Documentation tests" "cargo test --doc"; then
    FAILED=$((FAILED + 1))
fi

# 6. Build documentation
if ! run_test "Documentation build" "cargo doc --no-deps --all-features"; then
    FAILED=$((FAILED + 1))
fi

# Summary
echo -e "${GREEN}=== Test Summary ===${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}${FAILED} test suite(s) failed${NC}"
    exit 1
fi
