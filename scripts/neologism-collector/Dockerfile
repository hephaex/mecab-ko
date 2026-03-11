# Multi-stage build for neologism collector

# Stage 1: Builder
FROM python:3.11-slim as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    g++ \
    make \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements
COPY requirements.txt .

# Install Python dependencies
RUN pip install --no-cache-dir --user -r requirements.txt

# Stage 2: Runtime
FROM python:3.11-slim

LABEL maintainer="MeCab-Ko Team"
LABEL description="Neologism Collector for MeCab-Ko"

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    # For KoNLPy (optional)
    openjdk-17-jre-headless \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 -s /bin/bash appuser

WORKDIR /app

# Copy Python dependencies from builder
COPY --from=builder /root/.local /home/appuser/.local

# Copy application files
COPY --chown=appuser:appuser . .

# Create necessary directories
RUN mkdir -p /app/data /app/logs /app/data/export && \
    chown -R appuser:appuser /app/data /app/logs

# Switch to non-root user
USER appuser

# Add local bin to PATH
ENV PATH=/home/appuser/.local/bin:$PATH
ENV PYTHONPATH=/app
ENV PYTHONUNBUFFERED=1

# Initialize database on first run
RUN python main.py init || true

# Expose metrics port (future use)
EXPOSE 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD python -c "import sqlite3; conn = sqlite3.connect('/app/data/neologisms.db'); conn.close()" || exit 1

# Default command: run scheduler
CMD ["python", "main.py", "scheduler"]
