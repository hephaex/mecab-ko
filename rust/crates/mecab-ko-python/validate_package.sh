#!/bin/bash
# Package structure validation script for mecab-ko-python

set -e

echo "🔍 Validating mecab-ko-python package structure..."
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check function
check_file() {
    if [ -f "$1" ]; then
        echo -e "${GREEN}✓${NC} $1 exists"
        return 0
    else
        echo -e "${RED}✗${NC} $1 is missing"
        return 1
    fi
}

check_dir() {
    if [ -d "$1" ]; then
        echo -e "${GREEN}✓${NC} $1/ exists"
        return 0
    else
        echo -e "${RED}✗${NC} $1/ is missing"
        return 1
    fi
}

echo "📦 Checking package files..."
check_file "pyproject.toml"
check_file "Cargo.toml"
check_file "README.md"
check_file "MANIFEST.in"
check_file "LICENSE-MIT"
check_file "LICENSE-APACHE"
echo ""

echo "📄 Checking documentation..."
check_file "CHANGELOG.md"
check_file "CONTRIBUTING.md"
check_file "PYPI_RELEASE.md"
check_file "BND-003-IMPLEMENTATION.md"
check_file "INSTALL.md"
check_file "IMPLEMENTATION.md"
check_file "QUICKSTART.md"
check_file "SUMMARY.md"
echo ""

echo "🐍 Checking Python package structure..."
check_dir "python/mecab_ko"
check_file "python/mecab_ko/__init__.py"
check_file "python/mecab_ko/__init__.pyi"
check_file "python/mecab_ko/py.typed"
echo ""

echo "🧪 Checking test files..."
check_file "pytest.ini"
check_file "requirements-dev.txt"
check_dir "tests"
check_file "tests/test_mecab.py"
echo ""

echo "🦀 Checking Rust source..."
check_dir "src"
check_file "src/lib.rs"
echo ""

echo "📝 Checking examples..."
check_dir "examples"
echo ""

echo "🔧 Validating pyproject.toml..."
if grep -q "name = \"mecab-ko-python\"" pyproject.toml; then
    echo -e "${GREEN}✓${NC} Package name is correct: mecab-ko-python"
else
    echo -e "${RED}✗${NC} Package name is incorrect"
fi

if grep -q "module-name = \"mecab_ko\"" pyproject.toml; then
    echo -e "${GREEN}✓${NC} Module name is correct: mecab_ko"
else
    echo -e "${RED}✗${NC} Module name is incorrect"
fi

if grep -q "requires-python = \">=3.8\"" pyproject.toml; then
    echo -e "${GREEN}✓${NC} Python version requirement is correct"
else
    echo -e "${RED}✗${NC} Python version requirement is incorrect"
fi
echo ""

echo "🔧 Validating Cargo.toml..."
if grep -q "name = \"mecab-ko-python\"" Cargo.toml; then
    echo -e "${GREEN}✓${NC} Cargo package name is correct"
else
    echo -e "${RED}✗${NC} Cargo package name is incorrect"
fi

if grep -q "crate-type = \[\"cdylib\"\]" Cargo.toml; then
    echo -e "${GREEN}✓${NC} Crate type is correct (cdylib)"
else
    echo -e "${RED}✗${NC} Crate type is incorrect"
fi
echo ""

echo "📋 Checking GitHub Actions workflow..."
if [ -f "../../.github/workflows/pypi-publish.yml" ] || [ -f "../../../.github/workflows/pypi-publish.yml" ]; then
    echo -e "${GREEN}✓${NC} PyPI publish workflow exists"
else
    echo -e "${YELLOW}⚠${NC}  PyPI publish workflow not found in expected location"
fi
echo ""

echo "🧹 Checking code formatting..."
if command -v cargo &> /dev/null; then
    echo "Running cargo fmt check..."
    if cargo fmt -- --check 2>&1 | grep -q "Diff"; then
        echo -e "${YELLOW}⚠${NC}  Code needs formatting (run: cargo fmt)"
    else
        echo -e "${GREEN}✓${NC} Code is formatted correctly"
    fi
else
    echo -e "${YELLOW}⚠${NC}  cargo not found, skipping format check"
fi
echo ""

echo "🔍 Checking for common issues..."

# Check for unwrap() in library code
if grep -r "unwrap()" src/ 2>/dev/null | grep -v "test" | grep -v "//"; then
    echo -e "${YELLOW}⚠${NC}  Found unwrap() calls in library code (should use proper error handling)"
else
    echo -e "${GREEN}✓${NC} No unwrap() calls found in library code"
fi

# Check for expect() in library code
if grep -r "expect(" src/ 2>/dev/null | grep -v "test" | grep -v "//"; then
    echo -e "${YELLOW}⚠${NC}  Found expect() calls in library code (should use proper error handling)"
else
    echo -e "${GREEN}✓${NC} No expect() calls found in library code"
fi

echo ""
echo "✨ Validation complete!"
echo ""
echo "📦 Next steps:"
echo "  1. Build the package: maturin build --release"
echo "  2. Test locally: maturin develop && pytest"
echo "  3. Check the wheel: ls -lh target/wheels/"
echo "  4. Validate with twine: twine check target/wheels/*"
echo "  5. Create a git tag: git tag v0.1.0 && git push origin v0.1.0"
echo ""
