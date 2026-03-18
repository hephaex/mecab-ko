# MeCab-Ko Docker Deployment Guide

A comprehensive guide to deploying MeCab-Ko using Docker with production-ready configurations.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Quick Start](#quick-start)
4. [Architecture](#architecture)
5. [CLI Container](#cli-container)
6. [API Server Container](#api-server-container)
7. [Docker Compose](#docker-compose)
8. [Production Deployment](#production-deployment)
9. [Troubleshooting](#troubleshooting)
10. [Performance Tuning](#performance-tuning)

---

## Overview

MeCab-Ko Docker images provide containerized deployment of Korean morphological analyzer:

- **CLI Image** (`mecab-ko:latest`): Command-line interface for text analysis
- **API Server Image** (`mecab-ko-api:latest`): FastAPI REST server for programmatic access

### Features

- Multi-stage builds for minimal image size
- Non-root user execution for security
- Health checks for container orchestration
- Environment variable configuration
- Resource limits and reservations
- UTF-8 locale support
- Dictionary bundled in image

### Image Sizes

- CLI Image: ~150MB
- API Server Image: ~300MB

---

## Prerequisites

### Requirements

- Docker 20.10+
- Docker Compose 1.29+ (for compose examples)
- 2GB RAM minimum
- 1GB disk space

### Optional

- Kubernetes 1.20+ (for K8s deployment)
- Container registry (Docker Hub, ECR, GCR, etc.)

### Check Docker Installation

```bash
# Verify Docker is installed
docker --version
# Output: Docker version 24.0.0, build abcdef1

# Verify Docker Compose is installed
docker compose version
# Output: Docker Compose version 2.20.0

# Verify Docker daemon is running
docker ps
```

---

## Quick Start

### 1. Build Docker Images

```bash
# Clone the repository
git clone https://github.com/hephaex/mecab-ko.git
cd mecab-ko

# Build CLI image
docker build -f docker/Dockerfile.cli -t mecab-ko:latest .

# Build API server image
docker build -f docker/Dockerfile.python-api -t mecab-ko-api:latest .

# Verify images
docker images | grep mecab-ko
```

### 2. Run CLI Container

```bash
# Simple usage with text input
echo "안녕하세요" | docker run --rm -i mecab-ko:latest

# Interactive mode
docker run --rm -it mecab-ko:latest

# Analyze text from arguments
docker run --rm mecab-ko:latest "형태소 분석 테스트"

# Output format options
docker run --rm mecab-ko:latest -O wakati "오늘 날씨가 좋습니다"
docker run --rm mecab-ko:latest -O json "JSON 출력"
```

### 3. Run API Server Container

```bash
# Start API server
docker run -d -p 8000:8000 --name mecab-api mecab-ko-api:latest

# Wait for server to be ready
sleep 5

# Check health
curl http://localhost:8000/health

# Test API endpoint
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -d '{"text":"안녕하세요"}'

# Stop server
docker stop mecab-api
docker rm mecab-api
```

### 4. Use Docker Compose (Recommended)

```bash
# Start all services
cd docker
docker compose up -d

# View logs
docker compose logs -f

# Test CLI
docker compose exec mecab-cli --help

# Test API
curl http://localhost:8000/health

# Stop all services
docker compose down
```

---

## Architecture

### Multi-Stage Build Strategy

#### CLI Image (`Dockerfile.cli`)

```
Stage 1: Builder (Rust)
  ├─ rust:1.75-bookworm base
  ├─ Install Rust dependencies
  ├─ Build mecab-ko CLI binary
  └─ Strip binary for size

Stage 2: Runtime
  ├─ debian:bookworm-slim base
  ├─ Copy binary from Stage 1
  ├─ Copy dictionary data
  └─ Final image: ~150MB
```

#### API Server Image (`Dockerfile.python-api`)

```
Stage 1: Python Builder
  ├─ python:3.11-bookworm base
  ├─ Build Python dependency wheels
  └─ Output: /wheels

Stage 2: Rust/Maturin Builder
  ├─ rust:1.75-bookworm base
  ├─ Build Python extension (mecab-ko-python)
  └─ Output: /wheels

Stage 3: Runtime
  ├─ python:3.11-slim-bookworm base
  ├─ Copy wheels from stages 1 & 2
  ├─ Install dependencies from wheels
  ├─ Copy FastAPI server code
  └─ Final image: ~300MB
```

### Layer Caching Strategy

Build files are organized for optimal Docker layer caching:

1. Base image selection
2. Dependencies installation
3. Source code copying
4. Build compilation

This allows faster rebuilds when only source code changes.

---

## CLI Container

### Basic Usage

```bash
# Run CLI in container
docker run --rm mecab-ko:latest <TEXT>

# Example: Analyze text
docker run --rm mecab-ko:latest "한국어 처리"

# Output:
# 한국 NNG,*,F,한국,*,*,*,*,*,*,*,*,*,*,*,*
# 어 NNG,*,F,어,*,*,*,*,*,*,*,*,*,*,*,*
# 처리 NNG,*,F,처리,*,*,*,*,*,*,*,*,*,*,*,*
# EOS
```

### Output Formats

```bash
# Default (MeCab format)
docker run --rm mecab-ko:latest "테스트"

# Wakati (space-separated tokens)
docker run --rm mecab-ko:latest -O wakati "테스트"
# Output: 테스 트

# JSON format
docker run --rm mecab-ko:latest -O json "테스트"

# CSV format
docker run --rm mecab-ko:latest -O csv "테스트"
```

### File Processing

```bash
# Create input file
echo "한국어 형태소 분석" > input.txt

# Process file with volume mount
docker run --rm \
  -v /path/to/input.txt:/input.txt:ro \
  mecab-ko:latest -i /input.txt

# Batch processing with directory
docker run --rm \
  -v /path/to/inputs:/inputs:ro \
  -v /path/to/outputs:/outputs:rw \
  mecab-ko:latest -i /inputs -o /outputs
```

### Interactive Mode

```bash
# Start interactive REPL
docker run --rm -it mecab-ko:latest --repl

# Type text to analyze:
# >> 안녕하세요
# 안녕 NNG,*,F,...
# >> 형태소 분석
# 형태 NNG,*,F,...
```

### Environment Variables

```bash
# Set dictionary path
docker run --rm \
  -e MECAB_DIC_DIR=/custom/dic \
  mecab-ko:latest "테스트"

# Set locale
docker run --rm \
  -e LANG=ko_KR.UTF-8 \
  -e LC_ALL=ko_KR.UTF-8 \
  mecab-ko:latest "테스트"
```

---

## API Server Container

### Starting the Server

```bash
# Basic start
docker run -d -p 8000:8000 mecab-ko-api:latest

# With custom configuration
docker run -d \
  -p 8000:8000 \
  -e WORKERS=8 \
  -e LOG_LEVEL=debug \
  mecab-ko-api:latest

# With volume mounts for logs
docker run -d \
  -p 8000:8000 \
  -v /path/to/logs:/app/logs \
  mecab-ko-api:latest
```

### API Endpoints

#### Health Check

```bash
# Check server health
curl http://localhost:8000/health

# Response:
# {
#   "status": "healthy",
#   "version": "0.5.0",
#   "dictionary_path": "/usr/share/mecab-ko-dic"
# }
```

#### Server Information

```bash
# Get server info
curl http://localhost:8000/info

# Response:
# {
#   "name": "MeCab-Ko API Server",
#   "version": "0.5.0",
#   "dictionary_path": "/usr/share/mecab-ko-dic",
#   "python_implementation": "CPython with Rust extension",
#   "description": "Korean morphological analyzer using MeCab-Ko"
# }
```

#### Text Analysis

```bash
# Analyze Korean text
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -d '{"text":"한국어 분석"}'

# Response:
# {
#   "success": true,
#   "data": {
#     "result": "한국 NNG,*,F,...\n..."
#   },
#   "processing_time_ms": 2.34
# }
```

#### Extract Morphemes

```bash
# Extract morphemes (tokens)
curl -X POST http://localhost:8000/morphs \
  -H "Content-Type: application/json" \
  -d '{"text":"오늘 날씨가 좋습니다"}'

# Response:
# {
#   "success": true,
#   "data": {
#     "morphemes": ["오늘", "날씨", "가", "좋", "습니다"],
#     "count": 5
#   },
#   "processing_time_ms": 1.23
# }
```

#### Extract Nouns

```bash
# Extract nouns
curl -X POST http://localhost:8000/nouns \
  -H "Content-Type: application/json" \
  -d '{"text":"서울에서 한국 음식을 먹었습니다"}'

# Response:
# {
#   "success": true,
#   "data": {
#     "nouns": ["서울", "한국", "음식"],
#     "count": 3
#   },
#   "processing_time_ms": 1.45
# }
```

#### Part-of-Speech Tagging

```bash
# POS tagging
curl -X POST http://localhost:8000/pos \
  -H "Content-Type: application/json" \
  -d '{"text":"나는 학생입니다"}'

# Response:
# {
#   "success": true,
#   "data": {
#     "pos_tags": [
#       {"surface": "나", "pos": "NP"},
#       {"surface": "는", "pos": "JKB"},
#       {"surface": "학생", "pos": "NNG"},
#       {"surface": "입니다", "pos": "VCP+EF"}
#     ],
#     "count": 4
#   },
#   "processing_time_ms": 1.56
# }
```

#### Batch Analysis

```bash
# Analyze multiple texts at once
curl -X POST http://localhost:8000/batch \
  -H "Content-Type: application/json" \
  -d '{
    "texts": [
      "첫 번째 문장입니다",
      "두 번째 문장입니다",
      "세 번째 문장입니다"
    ]
  }'

# Response:
# {
#   "success": true,
#   "results": [
#     {"success": true, "result": "..."},
#     {"success": true, "result": "..."},
#     {"success": true, "result": "..."}
#   ],
#   "failed": 0,
#   "total": 3,
#   "processing_time_ms": 3.45
# }
```

### Interactive API Documentation

```bash
# OpenAPI/Swagger UI
open http://localhost:8000/docs

# ReDoc
open http://localhost:8000/redoc

# OpenAPI JSON schema
curl http://localhost:8000/openapi.json
```

### Environment Variables

```bash
# Worker configuration
WORKERS=4                          # Number of Uvicorn workers

# Logging
LOG_LEVEL=info                     # Log level (debug, info, warning, error)

# Performance
MECAB_DIC_DIR=/usr/share/mecab-ko-dic

# Locale
LANG=C.UTF-8
LC_ALL=C.UTF-8

# Python
PYTHONUNBUFFERED=1                # Unbuffered output
```

### Container Networking

```bash
# Access API from another container
docker run --rm \
  --network=host \
  curlimages/curl:latest \
  curl http://localhost:8000/health

# Using container name in compose
docker run --rm \
  --network=mecab-ko-network \
  curlimages/curl:latest \
  curl http://mecab-api:8000/health
```

---

## Docker Compose

### Configuration Overview

The `docker-compose.yml` provides:

- **mecab-cli**: CLI container for batch processing
- **mecab-api**: API server container

### Starting Services

```bash
# Start all services
docker compose up -d

# Start with build
docker compose up -d --build

# Start specific service
docker compose up -d mecab-api

# View logs
docker compose logs -f

# View specific service logs
docker compose logs -f mecab-api
```

### Stopping Services

```bash
# Stop all services (keep volumes)
docker compose stop

# Stop and remove containers (keep volumes)
docker compose down

# Stop and remove everything (including volumes)
docker compose down -v
```

### Service Management

```bash
# Check service status
docker compose ps

# Execute command in running container
docker compose exec mecab-api curl http://localhost:8000/health

# View service logs with lines
docker compose logs -f --tail=100 mecab-api

# Restart service
docker compose restart mecab-api
```

### Environment Configuration

```bash
# Create .env file for compose
cat > .env << EOF
COMPOSE_PROJECT_NAME=mecab-ko
WORKERS=4
LOG_LEVEL=info
EOF

# Start with environment
docker compose up -d
```

### Volume Management

```bash
# List volumes
docker volume ls | grep mecab

# Inspect volume
docker volume inspect mecab-ko_logs

# Clean up unused volumes
docker volume prune
```

---

## Production Deployment

### Kubernetes Deployment

#### Create Deployment Manifest

```yaml
# mecab-ko-api-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mecab-ko-api
  labels:
    app: mecab-ko-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: mecab-ko-api
  template:
    metadata:
      labels:
        app: mecab-ko-api
    spec:
      containers:
      - name: mecab-ko-api
        image: mecab-ko-api:0.5.0
        ports:
        - containerPort: 8000
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "2"
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 5
          periodSeconds: 10
        env:
        - name: WORKERS
          value: "4"
        - name: LOG_LEVEL
          value: "info"
```

#### Service Exposure

```yaml
# mecab-ko-api-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: mecab-ko-api
spec:
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 8000
    protocol: TCP
  selector:
    app: mecab-ko-api
```

#### Deploy to Kubernetes

```bash
# Apply manifests
kubectl apply -f mecab-ko-api-deployment.yaml
kubectl apply -f mecab-ko-api-service.yaml

# Verify deployment
kubectl get pods -l app=mecab-ko-api
kubectl get svc mecab-ko-api

# Check logs
kubectl logs -l app=mecab-ko-api

# Scale deployment
kubectl scale deployment mecab-ko-api --replicas=5
```

### Container Registry

#### Push to Docker Hub

```bash
# Login to Docker Hub
docker login

# Tag image
docker tag mecab-ko-api:latest myregistry/mecab-ko-api:0.5.0

# Push image
docker push myregistry/mecab-ko-api:0.5.0

# Pull image in production
docker pull myregistry/mecab-ko-api:0.5.0
```

#### Push to AWS ECR

```bash
# Get login token
aws ecr get-login-password --region us-east-1 | \
  docker login --username AWS --password-stdin 123456789.dkr.ecr.us-east-1.amazonaws.com

# Tag image
docker tag mecab-ko-api:latest 123456789.dkr.ecr.us-east-1.amazonaws.com/mecab-ko-api:0.5.0

# Push image
docker push 123456789.dkr.ecr.us-east-1.amazonaws.com/mecab-ko-api:0.5.0
```

### Security Best Practices

#### Non-Root User

All images run as non-root user (`appuser` or `mecab`):

```dockerfile
RUN useradd -m -s /sbin/nologin appuser
USER appuser
```

#### Read-Only Filesystem (Kubernetes)

```yaml
spec:
  securityContext:
    readOnlyRootFilesystem: true
  volumeMounts:
  - name: tmp
    mountPath: /tmp
  volumes:
  - name: tmp
    emptyDir: {}
```

#### Network Policy

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: mecab-ko-api-network-policy
spec:
  podSelector:
    matchLabels:
      app: mecab-ko-api
  policyTypes:
  - Ingress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: api-client
    ports:
    - protocol: TCP
      port: 8000
```

### Monitoring & Logging

#### Prometheus Metrics

```bash
# Export metrics (requires instrumentation)
curl http://localhost:8000/metrics
```

#### ELK Stack Integration

```yaml
# docker-compose.yml with logging
services:
  mecab-api:
    logging:
      driver: json-file
      options:
        tag: "{{.ImageName}}|{{.Name}}|{{.ImageID}}"
```

#### Cloud Logging

```bash
# Google Cloud Logging
docker run --log-driver gcplogs \
  -p 8000:8000 \
  mecab-ko-api:latest

# AWS CloudWatch
docker run --log-driver awslogs \
  --log-opt awslogs-group=/mecab-ko/api \
  -p 8000:8000 \
  mecab-ko-api:latest
```

---

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker logs mecab-ko-api

# Check image exists
docker images | grep mecab-ko

# Rebuild image
docker build -f docker/Dockerfile.python-api -t mecab-ko-api:latest .

# Try with verbose logging
docker run -e LOG_LEVEL=debug mecab-ko-api:latest
```

### API Not Responding

```bash
# Check if container is running
docker ps | grep mecab-ko-api

# Check port mapping
docker port mecab-ko-api

# Test connection
curl -v http://localhost:8000/health

# Check container logs
docker logs mecab-ko-api

# Verify health check
docker inspect mecab-ko-api | grep -A 10 Health
```

### High Memory Usage

```bash
# Check memory usage
docker stats mecab-ko-api

# Limit memory in compose
docker compose down
# Edit docker-compose.yml - reduce memory limit
docker compose up -d

# Monitor over time
docker stats --no-stream --format "table {{.Container}}\t{{.MemUsage}}"
```

### Dictionary Not Found

```bash
# Check dictionary is in image
docker run --rm mecab-ko-api:latest ls /usr/share/mecab-ko-dic

# Verify MECAB_DIC_DIR environment variable
docker run --rm mecab-ko-api:latest env | grep MECAB_DIC_DIR

# Check volume mounts
docker inspect mecab-ko-api | grep -A 5 Mounts
```

### Build Failures

```bash
# Clear build cache
docker builder prune -a

# Rebuild with no cache
docker build --no-cache -f docker/Dockerfile.cli -t mecab-ko:latest .

# Check build context
ls -la .dockerignore
cat .dockerignore

# Verify docker has access to files
docker build -f docker/Dockerfile.cli --progress=plain -t mecab-ko:latest .
```

---

## Performance Tuning

### Worker Configuration

```bash
# Calculate optimal workers (4x CPU cores)
docker run --rm -e WORKERS=8 mecab-ko-api:latest

# Monitor performance
docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}"
```

### Caching Optimization

```bash
# Build with BuildKit for faster incremental builds
DOCKER_BUILDKIT=1 docker build -f docker/Dockerfile.cli -t mecab-ko:latest .

# Use build cache
docker build --cache-from mecab-ko:latest -t mecab-ko:latest .
```

### Image Size Optimization

```bash
# Check image size
docker images mecab-ko --format "{{.Size}}"

# Remove unused images
docker image prune -a

# Use distroless images (future enhancement)
# FROM gcr.io/distroless/python3.11-debian12
```

### Network Performance

```bash
# Use host network (Linux only, not portable)
docker run --network=host mecab-ko-api:latest

# Use macvlan for better isolation
docker network create -d macvlan --subnet=192.168.1.0/24 mecab-net
```

---

## Advanced Topics

### Custom Base Images

```dockerfile
# Use Alpine for smaller image (if dependencies available)
FROM alpine:3.18
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/mecab /usr/local/bin/
```

### Health Check Customization

```dockerfile
# Custom health check script
COPY health-check.sh /usr/local/bin/
HEALTHCHECK --interval=60s --timeout=30s --start-period=20s --retries=5 \
    CMD /usr/local/bin/health-check.sh
```

### Multi-Architecture Builds

```bash
# Build for multiple architectures
docker buildx build \
  --platform linux/amd64,linux/arm64/v8 \
  -t mecab-ko-api:latest \
  -f docker/Dockerfile.python-api \
  --push .
```

### Secrets Management

```bash
# Using Docker secrets (Swarm)
docker secret create mecab-config config.json
docker service create \
  --secret mecab-config \
  mecab-ko-api:latest

# Using environment file
docker run --env-file .env mecab-ko-api:latest

# Using ConfigMap (Kubernetes)
kubectl create configmap mecab-config --from-file=config.json
```

---

## Support & Resources

### Documentation Links

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [MeCab-Ko Repository](https://github.com/hephaex/mecab-ko)
- [FastAPI Documentation](https://fastapi.tiangolo.com/)

### Getting Help

- Open an issue on [GitHub](https://github.com/hephaex/mecab-ko/issues)
- Check existing documentation in `docs/` directory
- Review Docker logs: `docker logs <container_id>`

### Contributing

Contributions are welcome! Please follow the [Contributing Guide](../CONTRIBUTING.md).

---

## License

MeCab-Ko is distributed under the MIT OR Apache-2.0 licenses. See LICENSE files for details.

---

*Last Updated: 2026-03-18*
*Version: 0.5.0*
