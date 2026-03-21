# MeCab-Ko Docker Image
# Multi-stage build for minimal image size

# Stage 1: Build
FROM rust:1.83-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY rust/ ./rust/

# Download MeCab-Ko dictionary
RUN apt-get update && apt-get install -y curl && \
    curl -L https://bitbucket.org/eunjeon/mecab-ko-dic/downloads/mecab-ko-dic-2.1.1-20180720.tar.gz \
    -o /tmp/mecab-ko-dic.tar.gz && \
    mkdir -p ./data && \
    tar -xzf /tmp/mecab-ko-dic.tar.gz -C ./data && \
    mv ./data/mecab-ko-dic-2.1.1-20180720 ./data/mecab-ko-dic && \
    rm /tmp/mecab-ko-dic.tar.gz && \
    apt-get remove -y curl && apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

# Build release binary
WORKDIR /app/rust
RUN cargo build --release --bin mecab

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

# Copy dictionary data
COPY --from=builder /app/data/mecab-ko-dic /usr/share/mecab-ko-dic

# Set environment
ENV MECAB_DIC_DIR=/usr/share/mecab-ko-dic

# Switch to non-root user
USER mecab
WORKDIR /home/mecab

# Default command
ENTRYPOINT ["mecab"]
CMD ["--help"]
