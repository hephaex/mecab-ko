#!/bin/bash
# Integration Test Quick Reference
# Location: /home/mare/mecab-ko/elasticsearch-plugin

set -e

echo "=== MeCab-Ko Elasticsearch Plugin - Integration Test Commands ==="
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_section() {
    echo -e "${GREEN}=== $1 ===${NC}"
}

print_command() {
    echo -e "${YELLOW}$1${NC}"
    echo "  $2"
    echo ""
}

print_section "Basic Commands"

print_command "./gradlew integrationTest" \
    "Run all integration tests"

print_command "./gradlew integrationTest --tests MecabKoAnalyzerIT" \
    "Run analyzer tests only"

print_command "./gradlew integrationTest --tests MecabKoTokenizerIT" \
    "Run tokenizer tests only"

print_command "./gradlew integrationTest --tests MecabKoFilterIT" \
    "Run filter tests only"

print_command "./gradlew integrationTest --tests MecabKoIndexIT" \
    "Run indexing/search tests only"

print_section "Specific Test Methods"

print_command "./gradlew integrationTest --tests '*.testBasicIndexingAndSearch'" \
    "Run specific test method"

print_command "./gradlew integrationTest --tests '*Performance*'" \
    "Run all performance tests"

print_section "Test Reports"

print_command "./gradlew integrationTest jacocoTestReport" \
    "Run tests with coverage"

print_command "./gradlew testSummary" \
    "Print test summary"

print_command "open build/reports/tests/integrationTest/index.html" \
    "View test report (macOS)"

print_command "xdg-open build/reports/tests/integrationTest/index.html" \
    "View test report (Linux)"

print_command "open build/reports/jacoco/test/html/index.html" \
    "View coverage report"

print_section "Advanced Options"

print_command "./gradlew integrationTest --info" \
    "Verbose output"

print_command "./gradlew integrationTest --debug" \
    "Debug output"

print_command "./gradlew integrationTest --debug-jvm" \
    "Remote debugging (port 5005)"

print_command "./gradlew integrationTest --parallel --max-workers=4" \
    "Parallel execution"

print_command "./gradlew integrationTest --rerun-tasks" \
    "Force re-run all tests"

print_section "Environment Setup"

print_command "export ES_JAVA_OPTS=\"-Xms1g -Xmx1g\"" \
    "Set Elasticsearch heap size"

print_command "export ES_TESTS_SECURITY_MANAGER=false" \
    "Disable security manager"

print_command "export GRADLE_OPTS=\"-Dorg.gradle.logging.level=debug\"" \
    "Gradle debug logging"

print_section "Prerequisites"

print_command "cd ../rust && cargo build --release" \
    "Build native library first"

print_command "cd ../elasticsearch-plugin" \
    "Navigate to plugin directory"

print_section "CI/CD"

print_command "git push origin main" \
    "Trigger GitHub Actions CI"

print_command "gh workflow run elasticsearch-plugin-tests.yml" \
    "Manually trigger CI (requires gh CLI)"

print_section "Cleanup"

print_command "./gradlew clean" \
    "Clean build directory"

print_command "pkill -f elasticsearch" \
    "Kill running Elasticsearch processes"

print_command "rm -rf build/testclusters" \
    "Remove test cluster data"

print_section "Quick Start"

echo "1. Build native library:"
echo "   cd ../rust && cargo build --release"
echo ""
echo "2. Run integration tests:"
echo "   cd ../elasticsearch-plugin"
echo "   ./gradlew integrationTest"
echo ""
echo "3. View results:"
echo "   open build/reports/tests/integrationTest/index.html"
echo ""

print_section "Test Files"

echo "Test Classes:"
echo "  - src/integTest/java/com/mecab/ko/elasticsearch/MecabKoAnalyzerIT.java"
echo "  - src/integTest/java/com/mecab/ko/elasticsearch/MecabKoTokenizerIT.java"
echo "  - src/integTest/java/com/mecab/ko/elasticsearch/MecabKoFilterIT.java"
echo "  - src/integTest/java/com/mecab/ko/elasticsearch/MecabKoIndexIT.java"
echo ""
echo "Test Data:"
echo "  - src/integTest/resources/test-data/korean_samples.json"
echo "  - src/integTest/resources/test-data/mixed_samples.json"
echo ""

print_section "Documentation"

echo "- TESTING.md - Comprehensive testing guide"
echo "- INTEGRATION_TESTS_SUMMARY.md - Implementation summary"
echo "- README.md - General documentation"
echo ""

print_section "Help"

echo "For more information:"
echo "  - Read TESTING.md for detailed guide"
echo "  - Check GitHub Actions for CI results"
echo "  - View test reports in build/reports/"
echo ""
