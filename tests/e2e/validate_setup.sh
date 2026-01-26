#!/bin/bash
# Validation script for E2E test setup

set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "================================"
echo "E2E Test Setup Validation"
echo "================================"
echo ""

PASS=0
FAIL=0
WARN=0

check_file() {
    if [ -f "$1" ]; then
        echo -e "${GREEN}✓${NC} $1"
        ((PASS++))
    else
        echo -e "${RED}✗${NC} $1 (missing)"
        ((FAIL++))
    fi
}

check_dir() {
    if [ -d "$1" ]; then
        echo -e "${GREEN}✓${NC} $1/"
        ((PASS++))
    else
        echo -e "${RED}✗${NC} $1/ (missing)"
        ((FAIL++))
    fi
}

check_executable() {
    if [ -x "$1" ]; then
        echo -e "${GREEN}✓${NC} $1 (executable)"
        ((PASS++))
    else
        echo -e "${YELLOW}!${NC} $1 (not executable)"
        ((WARN++))
    fi
}

check_command() {
    if command -v "$1" &> /dev/null; then
        echo -e "${GREEN}✓${NC} $1 (available)"
        ((PASS++))
    else
        echo -e "${YELLOW}!${NC} $1 (not found - optional)"
        ((WARN++))
    fi
}

echo "Checking directory structure..."
check_dir "cli"
check_dir "python"
check_dir "nodejs"
check_dir "wasm"
check_dir "common"
check_dir "fixtures"
echo ""

echo "Checking fixtures..."
check_file "fixtures/test_sentences.json"
check_file "fixtures/user_dict.csv"
echo ""

echo "Checking CLI tests..."
check_file "cli/test_cli_basic.bats"
check_file "cli/test_cli_output_formats.bats"
echo ""

echo "Checking Python tests..."
check_file "python/conftest.py"
check_file "python/requirements.txt"
check_file "python/test_basic_tokenization.py"
check_file "python/test_user_dict.py"
echo ""

echo "Checking Node.js tests..."
check_file "nodejs/package.json"
check_file "nodejs/vitest.config.js"
check_file "nodejs/basic.test.js"
echo ""

echo "Checking WASM tests..."
check_file "wasm/package.json"
check_file "wasm/basic.test.js"
echo ""

echo "Checking common utilities..."
check_file "common/test_runner.sh"
check_file "common/consistency_check.py"
check_file "common/benchmark.sh"
check_executable "common/test_runner.sh"
check_executable "common/consistency_check.py"
check_executable "common/benchmark.sh"
echo ""

echo "Checking documentation..."
check_file "README.md"
check_file "IMPLEMENTATION_SUMMARY.md"
check_file "Makefile"
check_file "../../docs/E2E_TESTING.md"
echo ""

echo "Checking optional tools..."
check_command "bats"
check_command "pytest"
check_command "node"
check_command "npm"
check_command "python3"
check_command "cargo"
check_command "jq"
echo ""

echo "Validating test fixtures..."
if command -v jq &> /dev/null; then
    if jq empty fixtures/test_sentences.json 2>/dev/null; then
        echo -e "${GREEN}✓${NC} test_sentences.json is valid JSON"
        ((PASS++))
    else
        echo -e "${RED}✗${NC} test_sentences.json is invalid JSON"
        ((FAIL++))
    fi

    TEST_COUNT=$(jq '.test_cases | length' fixtures/test_sentences.json 2>/dev/null || echo 0)
    echo -e "${GREEN}✓${NC} Found ${TEST_COUNT} test cases in fixtures"
else
    echo -e "${YELLOW}!${NC} jq not available, skipping JSON validation"
    ((WARN++))
fi
echo ""

echo "================================"
echo "Summary"
echo "================================"
echo -e "${GREEN}Passed:${NC} $PASS"
echo -e "${RED}Failed:${NC} $FAIL"
echo -e "${YELLOW}Warnings:${NC} $WARN"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✓ E2E test setup is complete!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Install dependencies: make install-deps"
    echo "  2. Build bindings: make build"
    echo "  3. Run tests: make test"
    echo "  4. Check consistency: make consistency-check"
    echo "  5. Run benchmarks: make benchmark"
    exit 0
else
    echo -e "${RED}✗ E2E test setup has issues!${NC}"
    echo "Please check the missing files above."
    exit 1
fi
