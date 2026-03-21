# MeCab-Ko Docker Image
# Multi-stage build for minimal image size

# Stage 1: Build
FROM rust:1.83-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY rust/ ./rust/

# Download MeCab-Ko dictionary CSV
RUN curl -L https://bitbucket.org/eunjeon/mecab-ko-dic/downloads/mecab-ko-dic-2.1.1-20180720.tar.gz \
    -o /tmp/mecab-ko-dic.tar.gz && \
    mkdir -p ./data && \
    tar -xzf /tmp/mecab-ko-dic.tar.gz -C ./data && \
    rm /tmp/mecab-ko-dic.tar.gz

# Build dictionary builder and CLI
WORKDIR /app/rust
RUN cargo build --release --bin mecab-ko-dict-builder --bin mecab

# Compile dictionary from CSV to binary format
RUN mkdir -p /app/dict-output && \
    ./target/release/mecab-ko-dict-builder \
    --input /app/data/mecab-ko-dic-2.1.1-20180720 \
    --output /app/dict-output \
    --compression 3 \
    --encoding auto \
    --verbose

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash mecab

# Copy binary
COPY --from=builder /app/rust/target/release/mecab /usr/local/bin/mecab

# Copy compiled dictionary (binary format)
COPY --from=builder /app/dict-output /usr/share/mecab-ko-dic

# Set environment
ENV MECAB_DICDIR=/usr/share/mecab-ko-dic

# Switch to non-root user
USER mecab
WORKDIR /home/mecab

# Default command
ENTRYPOINT ["mecab"]
CMD ["--help"]
