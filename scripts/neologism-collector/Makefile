# Makefile for neologism collector

.PHONY: help install test lint format clean run docker-build docker-up docker-down

help:
	@echo "MeCab-Ko Neologism Collector - Make Commands"
	@echo ""
	@echo "Setup:"
	@echo "  make install      Install dependencies"
	@echo "  make init         Initialize database"
	@echo ""
	@echo "Development:"
	@echo "  make test         Run tests"
	@echo "  make lint         Run linters"
	@echo "  make format       Format code"
	@echo "  make typecheck    Run type checking"
	@echo ""
	@echo "Running:"
	@echo "  make run          Run pipeline once"
	@echo "  make scheduler    Start scheduler"
	@echo "  make stats        Show statistics"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-build Build Docker image"
	@echo "  make docker-up    Start Docker container"
	@echo "  make docker-down  Stop Docker container"
	@echo ""
	@echo "Maintenance:"
	@echo "  make cleanup      Clean up old data"
	@echo "  make clean        Remove generated files"

install:
	pip install -r requirements.txt

init:
	python main.py init

test:
	pytest -v --cov=src --cov-report=term-missing

lint:
	ruff check src/ tests/
	black --check src/ tests/

format:
	black src/ tests/
	ruff check --fix src/ tests/

typecheck:
	mypy src/

run:
	python main.py pipeline

scheduler:
	python main.py scheduler

stats:
	python main.py stats

cleanup:
	python main.py cleanup

docker-build:
	docker-compose build

docker-up:
	docker-compose up -d

docker-down:
	docker-compose down

docker-logs:
	docker-compose logs -f

clean:
	find . -type d -name __pycache__ -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete
	find . -type f -name "*.pyo" -delete
	find . -type d -name "*.egg-info" -exec rm -rf {} +
	find . -type d -name ".pytest_cache" -exec rm -rf {} +
	find . -type d -name ".mypy_cache" -exec rm -rf {} +
	find . -type d -name ".ruff_cache" -exec rm -rf {} +
	rm -rf htmlcov/
	rm -rf .coverage
	rm -rf dist/
	rm -rf build/
