#!/bin/bash
# Generate test coverage report for MeCab-Ko

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== MeCab-Ko Coverage Report ===${NC}\n"

# Change to project root
cd "$(dirname "$0")/.."

# Check if tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo -e "${YELLOW}cargo-tarpaulin not found. Installing...${NC}"
    cargo install cargo-tarpaulin
fi

# Create coverage directory
mkdir -p coverage

# Run coverage
echo -e "${YELLOW}Generating coverage report...${NC}"
cargo tarpaulin \
    --workspace \
    --tests \
    --out Html \
    --out Xml \
    --output-dir coverage \
    --exclude-files 'target/*' \
    --exclude-files 'tests/*' \
    --timeout 300

# Display summary
echo -e "\n${GREEN}Coverage report generated!${NC}"
echo "HTML report: coverage/index.html"
echo "XML report: coverage/cobertura.xml"

# Try to open HTML report
if command -v xdg-open &> /dev/null; then
    xdg-open coverage/index.html
elif command -v open &> /dev/null; then
    open coverage/index.html
else
    echo "Open coverage/index.html in your browser to view the report"
fi
