# MeCab-Ko Docker Makefile
# Provides convenient commands for building and managing Docker images/containers

.PHONY: help build build-cli build-api build-no-cache clean run-cli run-api logs test push pull

# Variables
DOCKER := docker
COMPOSE := docker compose
IMAGE_NAME := mecab-ko
API_IMAGE_NAME := mecab-ko-api
VERSION := 0.5.0
REGISTRY :=
DOCKER_BUILDKIT := 1

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
NC := \033[0m

# ============================================================================
# Help
# ============================================================================

help:
	@echo "$(BLUE)MeCab-Ko Docker Management$(NC)"
	@echo ""
	@echo "$(GREEN)Build Commands:$(NC)"
	@echo "  make build            Build all Docker images"
	@echo "  make build-cli        Build CLI image"
	@echo "  make build-api        Build API server image"
	@echo "  make build-no-cache   Build all images without cache"
	@echo ""
	@echo "$(GREEN)Docker Compose Commands:$(NC)"
	@echo "  make up               Start all services"
	@echo "  make down             Stop all services"
	@echo "  make ps               Show running services"
	@echo "  make logs             View logs (all services)"
	@echo "  make logs-api         View API logs only"
	@echo "  make logs-cli         View CLI logs only"
	@echo ""
	@echo "$(GREEN)Container Commands:$(NC)"
	@echo "  make run-cli          Run CLI container (interactive)"
	@echo "  make run-api          Run API server container"
	@echo "  make stop-cli         Stop CLI container"
	@echo "  make stop-api         Stop API container"
	@echo ""
	@echo "$(GREEN)Testing & Validation:$(NC)"
	@echo "  make test-cli         Test CLI container"
	@echo "  make test-api         Test API server"
	@echo "  make test-all         Run all tests"
	@echo "  make lint             Run linting checks"
	@echo ""
	@echo "$(GREEN)Cleanup Commands:$(NC)"
	@echo "  make clean            Remove containers"
	@echo "  make clean-all        Remove containers and images"
	@echo "  make prune            Prune unused Docker resources"
	@echo ""
	@echo "$(GREEN)Registry Commands:$(NC)"
	@echo "  make tag              Tag images for registry"
	@echo "  make push             Push images to registry"
	@echo "  make pull             Pull images from registry"
	@echo ""
	@echo "$(GREEN)Utility Commands:$(NC)"
	@echo "  make shell-api        Interactive shell in API container"
	@echo "  make shell-cli        Interactive shell in CLI container"
	@echo "  make info             Show image information"
	@echo "  make stats            Show container resource usage"
	@echo "  make examples         Show usage examples"

# ============================================================================
# Build Commands
# ============================================================================

build: build-cli build-api
	@echo "$(GREEN)✓ All images built successfully$(NC)"

build-cli:
	@echo "$(BLUE)Building CLI image...$(NC)"
	$(DOCKER) build \
		--progress=plain \
		-f Dockerfile.cli \
		-t $(IMAGE_NAME):latest \
		-t $(IMAGE_NAME):$(VERSION) \
		..
	@echo "$(GREEN)✓ CLI image built$(NC)"

build-api:
	@echo "$(BLUE)Building API server image...$(NC)"
	$(DOCKER) build \
		--progress=plain \
		-f Dockerfile.python-api \
		-t $(API_IMAGE_NAME):latest \
		-t $(API_IMAGE_NAME):$(VERSION) \
		..
	@echo "$(GREEN)✓ API image built$(NC)"

build-no-cache:
	@echo "$(BLUE)Building images without cache...$(NC)"
	$(DOCKER) build --no-cache -f Dockerfile.cli -t $(IMAGE_NAME):latest ..
	$(DOCKER) build --no-cache -f Dockerfile.python-api -t $(API_IMAGE_NAME):latest ..
	@echo "$(GREEN)✓ All images rebuilt$(NC)"

# ============================================================================
# Docker Compose Commands
# ============================================================================

up:
	@echo "$(BLUE)Starting services with Docker Compose...$(NC)"
	$(COMPOSE) up -d
	@echo "$(GREEN)✓ Services started$(NC)"
	@echo ""
	@echo "Service URLs:"
	@echo "  API: http://localhost:8000"
	@echo "  Docs: http://localhost:8000/docs"

up-build:
	@echo "$(BLUE)Building and starting services...$(NC)"
	$(COMPOSE) up -d --build
	@echo "$(GREEN)✓ Services ready$(NC)"

down:
	@echo "$(BLUE)Stopping services...$(NC)"
	$(COMPOSE) down
	@echo "$(GREEN)✓ Services stopped$(NC)"

ps:
	@echo "$(BLUE)Running services:$(NC)"
	@$(COMPOSE) ps

logs:
	@$(COMPOSE) logs -f

logs-api:
	@$(COMPOSE) logs -f mecab-api

logs-cli:
	@$(COMPOSE) logs -f mecab-cli

# ============================================================================
# Container Commands
# ============================================================================

run-cli:
	@echo "$(BLUE)Starting CLI container...$(NC)"
	$(DOCKER) run --rm -it $(IMAGE_NAME):latest --repl

run-api:
	@echo "$(BLUE)Starting API server...$(NC)"
	$(DOCKER) run -d -p 8000:8000 --name mecab-api-dev $(API_IMAGE_NAME):latest
	@sleep 2
	@echo "$(GREEN)✓ API server running at http://localhost:8000$(NC)"

stop-cli:
	@echo "$(BLUE)Stopping CLI container...$(NC)"
	$(DOCKER) stop $$($(DOCKER) ps -q -f ancestor=$(IMAGE_NAME):latest) || true

stop-api:
	@echo "$(BLUE)Stopping API server...$(NC)"
	$(DOCKER) stop mecab-api-dev || true
	$(DOCKER) rm mecab-api-dev || true
	@echo "$(GREEN)✓ API server stopped$(NC)"

# ============================================================================
# Testing Commands
# ============================================================================

test-cli: build-cli
	@echo "$(BLUE)Testing CLI image...$(NC)"
	@echo "Testing basic functionality..."
	$(DOCKER) run --rm $(IMAGE_NAME):latest --version
	$(DOCKER) run --rm $(IMAGE_NAME):latest --help
	@echo "Testing analysis..."
	@echo "한국어 분석" | $(DOCKER) run --rm -i $(IMAGE_NAME):latest > /dev/null
	@echo "$(GREEN)✓ CLI tests passed$(NC)"

test-api: build-api
	@echo "$(BLUE)Testing API server image...$(NC)"
	@echo "Starting API server..."
	$(DOCKER) run -d -p 8001:8000 --name mecab-api-test $(API_IMAGE_NAME):latest
	@sleep 5
	@echo "Testing health endpoint..."
	@curl -s http://localhost:8001/health | grep -q "healthy" || exit 1
	@echo "Testing analyze endpoint..."
	@curl -s -X POST http://localhost:8001/analyze \
		-H "Content-Type: application/json" \
		-d '{"text":"테스트"}' | grep -q "success" || exit 1
	@echo "Cleaning up..."
	$(DOCKER) stop mecab-api-test
	$(DOCKER) rm mecab-api-test
	@echo "$(GREEN)✓ API tests passed$(NC)"

test-all: test-cli test-api
	@echo "$(GREEN)✓ All tests passed$(NC)"

lint:
	@echo "$(BLUE)Running lint checks...$(NC)"
	@echo "Checking Dockerfiles..."
	@docker run --rm -i hadolint/hadolint < Dockerfile.cli || true
	@docker run --rm -i hadolint/hadolint < Dockerfile.python-api || true
	@echo "$(GREEN)✓ Lint checks complete$(NC)"

# ============================================================================
# Cleanup Commands
# ============================================================================

clean:
	@echo "$(BLUE)Cleaning up containers...$(NC)"
	$(DOCKER) rm -f $$($(DOCKER) ps -a -q -f ancestor=$(IMAGE_NAME):latest) || true
	$(DOCKER) rm -f $$($(DOCKER) ps -a -q -f ancestor=$(API_IMAGE_NAME):latest) || true
	@echo "$(GREEN)✓ Containers cleaned$(NC)"

clean-all: clean
	@echo "$(BLUE)Removing images...$(NC)"
	$(DOCKER) rmi $(IMAGE_NAME):latest $(IMAGE_NAME):$(VERSION) || true
	$(DOCKER) rmi $(API_IMAGE_NAME):latest $(API_IMAGE_NAME):$(VERSION) || true
	@echo "$(GREEN)✓ Images removed$(NC)"

prune:
	@echo "$(BLUE)Pruning unused Docker resources...$(NC)"
	$(DOCKER) system prune -f --volumes
	@echo "$(GREEN)✓ Pruned$(NC)"

# ============================================================================
# Registry Commands
# ============================================================================

tag:
	@echo "$(BLUE)Tagging images for registry...$(NC)"
	@[ ! -z "$(REGISTRY)" ] || (echo "$(RED)Error: REGISTRY not set$(NC)" && exit 1)
	$(DOCKER) tag $(IMAGE_NAME):latest $(REGISTRY)/$(IMAGE_NAME):$(VERSION)
	$(DOCKER) tag $(IMAGE_NAME):latest $(REGISTRY)/$(IMAGE_NAME):latest
	$(DOCKER) tag $(API_IMAGE_NAME):latest $(REGISTRY)/$(API_IMAGE_NAME):$(VERSION)
	$(DOCKER) tag $(API_IMAGE_NAME):latest $(REGISTRY)/$(API_IMAGE_NAME):latest
	@echo "$(GREEN)✓ Images tagged$(NC)"

push: tag
	@echo "$(BLUE)Pushing images to registry...$(NC)"
	@[ ! -z "$(REGISTRY)" ] || (echo "$(RED)Error: REGISTRY not set$(NC)" && exit 1)
	$(DOCKER) push $(REGISTRY)/$(IMAGE_NAME):$(VERSION)
	$(DOCKER) push $(REGISTRY)/$(IMAGE_NAME):latest
	$(DOCKER) push $(REGISTRY)/$(API_IMAGE_NAME):$(VERSION)
	$(DOCKER) push $(REGISTRY)/$(API_IMAGE_NAME):latest
	@echo "$(GREEN)✓ Images pushed$(NC)"

pull:
	@echo "$(BLUE)Pulling images from registry...$(NC)"
	@[ ! -z "$(REGISTRY)" ] || (echo "$(RED)Error: REGISTRY not set$(NC)" && exit 1)
	$(DOCKER) pull $(REGISTRY)/$(IMAGE_NAME):latest
	$(DOCKER) pull $(REGISTRY)/$(API_IMAGE_NAME):latest
	@echo "$(GREEN)✓ Images pulled$(NC)"

# ============================================================================
# Utility Commands
# ============================================================================

shell-api:
	@echo "$(BLUE)Starting shell in API container...$(NC)"
	$(DOCKER) run -it --rm -p 8000:8000 $(API_IMAGE_NAME):latest /bin/sh

shell-cli:
	@echo "$(BLUE)Starting shell in CLI container...$(NC)"
	$(DOCKER) run -it --rm $(IMAGE_NAME):latest /bin/sh

info:
	@echo "$(BLUE)Docker Image Information:$(NC)"
	@echo ""
	@echo "CLI Image:"
	$(DOCKER) images $(IMAGE_NAME) --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"
	@echo ""
	@echo "API Server Image:"
	$(DOCKER) images $(API_IMAGE_NAME) --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"

stats:
	@echo "$(BLUE)Container Resource Usage:$(NC)"
	$(DOCKER) stats --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}"

examples:
	@bash examples.sh all

# ============================================================================
# Default target
# ============================================================================

.DEFAULT_GOAL := help
