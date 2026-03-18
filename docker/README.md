# MeCab-Ko Docker Deployment

Quick reference for MeCab-Ko Docker images and deployment.

## Docker Images

### CLI Image (Dockerfile.cli)
- **Purpose**: Command-line interface for morphological analysis
- **Size**: ~150MB
- **Base**: debian:bookworm-slim
- **Build**: Multi-stage (Rust builder + runtime)
- **Tag**: `mecab-ko:latest`

### API Server Image (Dockerfile.python-api)
- **Purpose**: REST API server with FastAPI
- **Size**: ~300MB
- **Base**: python:3.11-slim-bookworm
- **Build**: Three-stage (Python + Rust + runtime)
- **Tag**: `mecab-ko-api:latest`

## Quick Start

### Using Docker Compose (Recommended)
```bash
docker compose up -d
curl http://localhost:8000/health
```

### Using Makefile
```bash
make build
make up
make test-all
```

### Manual Build
```bash
docker build -f Dockerfile.cli -t mecab-ko:latest ..
docker build -f Dockerfile.python-api -t mecab-ko-api:latest ..
```

## API Endpoints

- `GET /health` - Health check
- `GET /info` - Server info
- `POST /analyze` - Full analysis
- `POST /morphs` - Extract tokens
- `POST /nouns` - Extract nouns
- `POST /pos` - POS tagging
- `POST /batch` - Batch processing

## Documentation

See [`docs/docker/README.md`](../../docs/docker/README.md) for complete deployment guide:

- Architecture and design
- Production deployment (Kubernetes, registries)
- Security best practices
- Troubleshooting
- Performance tuning
- 40+ usage examples

## Management

### Makefile Targets
```bash
make build              # Build images
make up                # Start services
make down              # Stop services
make test-all          # Run tests
make logs              # View logs
make clean-all         # Cleanup
make help              # Show all targets
```

## Examples

See [`examples.sh`](examples.sh) for 40+ usage examples:

```bash
bash examples.sh cli                # CLI examples
bash examples.sh api                # API examples
bash examples.sh compose            # Docker Compose examples
bash examples.sh performance        # Performance testing
bash examples.sh all                # All examples
```

## Environment Variables

- `MECAB_DIC_DIR` - Dictionary path (default: /usr/share/mecab-ko-dic)
- `WORKERS` - API workers (default: 4)
- `LOG_LEVEL` - Logging level (default: info)
- `LANG` / `LC_ALL` - Locale (default: C.UTF-8)

## Files

- `Dockerfile.cli` - CLI image (63 lines)
- `Dockerfile.python-api` - API server image (92 lines)
- `api_server.py` - FastAPI application (550+ lines)
- `docker-compose.yml` - Compose orchestration (168 lines)
- `requirements.txt` - Python dependencies (7 lines)
- `nginx.conf` - Reverse proxy configuration
- `Makefile` - Automation targets
- `examples.sh` - Usage examples
- `.gitignore` - Version control

## Resources

- [Complete Documentation](../../docs/docker/README.md)
- [Project README](../../README.md)
- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [Docker Documentation](https://docs.docker.com/)

## License

MIT OR Apache-2.0

