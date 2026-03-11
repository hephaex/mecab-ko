# Makefile for MeCab-Ko E2E Tests

.PHONY: all test clean install-deps help
.PHONY: test-cli test-python test-nodejs test-wasm
.PHONY: build build-cli build-python build-nodejs build-wasm
.PHONY: benchmark consistency-check coverage

# Default target
all: test

help: ## Show this help message
	@echo "MeCab-Ko E2E Test Suite"
	@echo ""
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-20s %s\n", $$1, $$2}'

# Build targets
build: build-cli build-python build-nodejs build-wasm ## Build all bindings

build-cli: ## Build CLI binary
	@echo "Building CLI binary..."
	cd ../../rust && cargo build --release --bin mecab-ko

build-python: ## Build Python binding
	@echo "Building Python binding..."
	cd ../../rust/crates/mecab-ko-python && \
		pip install maturin && \
		maturin develop --release

build-nodejs: ## Build Node.js binding
	@echo "Building Node.js binding..."
	cd ../../rust/crates/mecab-ko-node && \
		npm install && \
		npm run build

build-wasm: ## Build WASM binding
	@echo "Building WASM binding..."
	cd ../../rust/crates/mecab-ko-wasm && \
		wasm-pack build --target web

# Test targets
test: ## Run all E2E tests
	@./common/test_runner.sh

test-cli: build-cli ## Run CLI tests only
	@./common/test_runner.sh --cli-only

test-python: build-python ## Run Python tests only
	@./common/test_runner.sh --python-only

test-nodejs: build-nodejs ## Run Node.js tests only
	@./common/test_runner.sh --nodejs-only

test-wasm: build-wasm ## Run WASM tests only
	@./common/test_runner.sh --wasm-only

test-no-build: ## Run all tests without rebuilding
	@./common/test_runner.sh --no-build

# Individual test suites
test-cli-basic: ## Run basic CLI tests
	cd cli && bats test_cli_basic.bats

test-cli-formats: ## Run CLI output format tests
	cd cli && bats test_cli_output_formats.bats

test-python-basic: ## Run basic Python tests
	cd python && pytest test_basic_tokenization.py -v

test-python-dict: ## Run Python user dict tests
	cd python && pytest test_user_dict.py -v

# Analysis targets
benchmark: build-cli ## Run performance benchmarks
	@./common/benchmark.sh

consistency-check: build ## Check consistency across bindings
	@./common/consistency_check.py

coverage: ## Run tests with coverage
	@echo "Running coverage for Python tests..."
	cd python && pytest --cov --cov-report=html --cov-report=term

# Installation targets
install-deps: install-cli-deps install-python-deps install-nodejs-deps ## Install all test dependencies

install-cli-deps: ## Install CLI test dependencies (bats)
	@echo "Installing CLI test dependencies..."
	@if command -v apt-get > /dev/null; then \
		sudo apt-get update && sudo apt-get install -y bats; \
	elif command -v brew > /dev/null; then \
		brew install bats-core; \
	else \
		echo "Please install bats manually: https://github.com/bats-core/bats-core"; \
	fi

install-python-deps: ## Install Python test dependencies
	@echo "Installing Python test dependencies..."
	cd python && pip install -r requirements.txt

install-nodejs-deps: ## Install Node.js test dependencies
	@echo "Installing Node.js test dependencies..."
	cd nodejs && npm install
	cd wasm && npm install

# Cleanup targets
clean: clean-python clean-nodejs clean-wasm ## Clean all test artifacts

clean-python: ## Clean Python test artifacts
	cd python && rm -rf .pytest_cache __pycache__ htmlcov .coverage

clean-nodejs: ## Clean Node.js test artifacts
	cd nodejs && rm -rf node_modules coverage
	cd wasm && rm -rf node_modules coverage

clean-wasm: ## Clean WASM test artifacts
	cd ../../rust/crates/mecab-ko-wasm && rm -rf pkg target

clean-all: clean ## Clean everything including build artifacts
	cd ../../rust && cargo clean

# CI targets
ci: install-deps build test ## Run full CI pipeline

ci-quick: test-no-build ## Quick CI check (no rebuild)

# Watch mode (requires entr)
watch-python: ## Watch and run Python tests on change
	@echo "Watching Python tests (requires entr)..."
	find python -name '*.py' | entr -c make test-python-basic

watch-cli: ## Watch and run CLI tests on change
	@echo "Watching CLI tests (requires entr)..."
	find cli -name '*.bats' | entr -c make test-cli-basic

# Report targets
report: ## Generate test report
	@echo "Generating test report..."
	@./common/test_runner.sh > test_report.txt 2>&1 || true
	@cat test_report.txt

# Debugging targets
debug-cli: ## Debug CLI with sample input
	@echo "나는 학교에 갑니다." | ../../rust/target/release/mecab-ko

debug-python: ## Debug Python binding with sample input
	@python3 -c "import mecab_ko; print(mecab_ko.Tagger().parse('나는 학교에 갑니다.'))"

# Docker support
docker-test: ## Run tests in Docker
	docker run --rm -v "$$PWD/../..:/workspace" -w /workspace/tests/e2e \
		rust:latest \
		bash -c "apt-get update && apt-get install -y bats python3-pip && make ci"
