# 웹 서버 통합 튜토리얼

MeCab-Ko를 웹 애플리케이션에 통합하는 방법을 알아봅니다.

## 목차

1. [Actix-web 통합](#actix-web-통합)
2. [Axum 통합](#axum-통합)
3. [REST API 설계](#rest-api-설계)
4. [성능 최적화](#성능-최적화)
5. [Docker 배포](#docker-배포)

## Actix-web 통합

### 프로젝트 설정

```bash
cargo new mecab-server
cd mecab-server
```

`Cargo.toml`:
```toml
[package]
name = "mecab-server"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4"
actix-rt = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
mecab-ko = "0.2"
tokio = { version = "1", features = ["full"] }
```

### 기본 서버 구현

```rust
use actix_web::{web, App, HttpResponse, HttpServer};
use mecab_ko::Tokenizer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// 요청/응답 구조체
#[derive(Deserialize)]
struct AnalyzeRequest {
    text: String,
    #[serde(default)]
    output_format: Option<String>,
}

#[derive(Serialize)]
struct Token {
    surface: String,
    pos: String,
    start: usize,
    end: usize,
}

#[derive(Serialize)]
struct AnalyzeResponse {
    tokens: Vec<Token>,
    morphs: Vec<String>,
    nouns: Vec<String>,
}

// 애플리케이션 상태
struct AppState {
    tokenizer: Tokenizer,
}

// 핸들러
async fn analyze(
    state: web::Data<Arc<AppState>>,
    req: web::Json<AnalyzeRequest>,
) -> HttpResponse {
    let tokens = state.tokenizer.tokenize(&req.text);

    let response = AnalyzeResponse {
        tokens: tokens
            .iter()
            .map(|t| Token {
                surface: t.surface.clone(),
                pos: t.pos.clone(),
                start: t.start,
                end: t.end,
            })
            .collect(),
        morphs: tokens.iter().map(|t| t.surface.clone()).collect(),
        nouns: tokens
            .iter()
            .filter(|t| t.pos.starts_with("NN"))
            .map(|t| t.surface.clone())
            .collect(),
    };

    HttpResponse::Ok().json(response)
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 토크나이저 초기화 (한 번만)
    let tokenizer = Tokenizer::new().expect("Failed to initialize tokenizer");
    let state = Arc::new(AppState { tokenizer });

    println!("Server running at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/health", web::get().to(health))
            .route("/analyze", web::post().to(analyze))
    })
    .workers(num_cpus::get())  // CPU 코어 수만큼 워커
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
```

### 테스트

```bash
# 서버 실행
cargo run --release

# 분석 요청
curl -X POST http://localhost:8080/analyze \
  -H "Content-Type: application/json" \
  -d '{"text": "안녕하세요, 오늘 날씨가 좋네요!"}'
```

**응답:**
```json
{
  "tokens": [
    {"surface": "안녕", "pos": "NNG", "start": 0, "end": 6},
    {"surface": "하", "pos": "XSV", "start": 6, "end": 9},
    ...
  ],
  "morphs": ["안녕", "하", "세요", ",", "오늘", "날씨", "가", "좋", "네요", "!"],
  "nouns": ["안녕", "오늘", "날씨"]
}
```

## Axum 통합

### 프로젝트 설정

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
mecab-ko = "0.2"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Axum 서버 구현

```rust
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use mecab_ko::Tokenizer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

#[derive(Clone)]
struct AppState {
    tokenizer: Arc<Tokenizer>,
}

#[derive(Deserialize)]
struct AnalyzeRequest {
    text: String,
}

#[derive(Serialize)]
struct TokenResponse {
    surface: String,
    pos: String,
    start: usize,
    end: usize,
}

#[derive(Serialize)]
struct AnalyzeResponse {
    success: bool,
    data: AnalyzeData,
}

#[derive(Serialize)]
struct AnalyzeData {
    tokens: Vec<TokenResponse>,
    token_count: usize,
    noun_count: usize,
}

async fn analyze(
    State(state): State<AppState>,
    Json(payload): Json<AnalyzeRequest>,
) -> Result<Json<AnalyzeResponse>, StatusCode> {
    let tokens = state.tokenizer.tokenize(&payload.text);

    let token_responses: Vec<TokenResponse> = tokens
        .iter()
        .map(|t| TokenResponse {
            surface: t.surface.clone(),
            pos: t.pos.clone(),
            start: t.start,
            end: t.end,
        })
        .collect();

    let noun_count = tokens
        .iter()
        .filter(|t| t.pos.starts_with("NN"))
        .count();

    Ok(Json(AnalyzeResponse {
        success: true,
        data: AnalyzeData {
            token_count: token_responses.len(),
            noun_count,
            tokens: token_responses,
        },
    }))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "mecab-ko-api"
    }))
}

#[tokio::main]
async fn main() {
    // 로깅 설정
    tracing_subscriber::fmt::init();

    // 토크나이저 초기화
    let tokenizer = Arc::new(
        Tokenizer::new().expect("Failed to initialize tokenizer")
    );

    let state = AppState { tokenizer };

    // 라우터 설정
    let app = Router::new()
        .route("/health", get(health))
        .route("/analyze", post(analyze))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // 서버 시작
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    println!("Server running at http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
}
```

## REST API 설계

### API 엔드포인트

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | 헬스 체크 |
| POST | `/analyze` | 텍스트 분석 |
| POST | `/batch` | 배치 분석 |
| POST | `/nouns` | 명사 추출 |
| POST | `/pos` | 품사 태깅 |

### 배치 분석 엔드포인트

```rust
#[derive(Deserialize)]
struct BatchRequest {
    texts: Vec<String>,
}

#[derive(Serialize)]
struct BatchResponse {
    results: Vec<AnalyzeData>,
    total_texts: usize,
    total_tokens: usize,
}

async fn batch_analyze(
    State(state): State<AppState>,
    Json(payload): Json<BatchRequest>,
) -> Json<BatchResponse> {
    let results: Vec<AnalyzeData> = payload
        .texts
        .iter()
        .map(|text| {
            let tokens = state.tokenizer.tokenize(text);
            AnalyzeData {
                token_count: tokens.len(),
                noun_count: tokens.iter().filter(|t| t.pos.starts_with("NN")).count(),
                tokens: tokens
                    .iter()
                    .map(|t| TokenResponse {
                        surface: t.surface.clone(),
                        pos: t.pos.clone(),
                        start: t.start,
                        end: t.end,
                    })
                    .collect(),
            }
        })
        .collect();

    let total_tokens: usize = results.iter().map(|r| r.token_count).sum();

    Json(BatchResponse {
        total_texts: results.len(),
        total_tokens,
        results,
    })
}
```

### 에러 처리

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
    code: String,
}

enum AppError {
    InvalidInput(String),
    AnalysisFailed(String),
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::InvalidInput(msg) => {
                (StatusCode::BAD_REQUEST, "INVALID_INPUT", msg)
            }
            AppError::AnalysisFailed(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "ANALYSIS_FAILED", msg)
            }
            AppError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg)
            }
        };

        let body = Json(ErrorResponse {
            success: false,
            error: message,
            code: code.to_string(),
        });

        (status, body).into_response()
    }
}
```

## 성능 최적화

### 토크나이저 풀링

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

struct TokenizerPool {
    tokenizer: Arc<Tokenizer>,
    semaphore: Arc<Semaphore>,
}

impl TokenizerPool {
    fn new(max_concurrent: usize) -> Self {
        Self {
            tokenizer: Arc::new(Tokenizer::new().unwrap()),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    async fn analyze(&self, text: &str) -> Vec<Token> {
        let _permit = self.semaphore.acquire().await.unwrap();
        self.tokenizer.tokenize(text)
    }
}
```

### 캐싱 (Redis)

```rust
use redis::{AsyncCommands, Client as RedisClient};
use serde_json;

struct CachedTokenizer {
    tokenizer: Tokenizer,
    redis: RedisClient,
    cache_ttl: usize,
}

impl CachedTokenizer {
    async fn analyze(&self, text: &str) -> Vec<TokenResponse> {
        let cache_key = format!("mecab:{}", md5::compute(text).0);

        // 캐시 확인
        let mut conn = self.redis.get_async_connection().await.unwrap();
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(tokens) = serde_json::from_str(&cached) {
                return tokens;
            }
        }

        // 분석 실행
        let tokens = self.tokenizer.tokenize(text);
        let response: Vec<TokenResponse> = tokens
            .iter()
            .map(|t| TokenResponse {
                surface: t.surface.clone(),
                pos: t.pos.clone(),
                start: t.start,
                end: t.end,
            })
            .collect();

        // 캐시 저장
        let json = serde_json::to_string(&response).unwrap();
        let _: () = conn
            .set_ex(&cache_key, json, self.cache_ttl)
            .await
            .unwrap();

        response
    }
}
```

### 요청 제한 (Rate Limiting)

```rust
use tower_governor::{
    governor::GovernorConfigBuilder,
    GovernorLayer,
};

fn create_rate_limiter() -> GovernorLayer {
    let config = GovernorConfigBuilder::default()
        .per_second(100)  // 초당 100 요청
        .burst_size(50)   // 버스트 50
        .finish()
        .unwrap();

    GovernorLayer::with_config(config)
}

// Router에 적용
let app = Router::new()
    .route("/analyze", post(analyze))
    .layer(create_rate_limiter());
```

## Docker 배포

### Dockerfile

```dockerfile
# Build stage
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# 릴리스 빌드
RUN cargo build --release --bin mecab-server

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 바이너리 복사
COPY --from=builder /app/target/release/mecab-server /app/

# 사전 데이터 복사 (필요한 경우)
# COPY --from=builder /app/data/dict /app/data/dict

# 비루트 사용자
RUN useradd -m appuser
USER appuser

EXPOSE 8080

ENV RUST_LOG=info

CMD ["./mecab-server"]
```

### docker-compose.yml

```yaml
version: '3.8'

services:
  mecab-api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - MECAB_DIC_DIR=/app/data/dict
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '2'
          memory: 512M
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - mecab-api

volumes:
  redis-data:
```

### Nginx 설정

```nginx
upstream mecab_backend {
    least_conn;
    server mecab-api:8080;
}

server {
    listen 80;

    location /api/ {
        proxy_pass http://mecab_backend/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;

        # 타임아웃 설정
        proxy_connect_timeout 5s;
        proxy_send_timeout 10s;
        proxy_read_timeout 30s;
    }

    location /health {
        proxy_pass http://mecab_backend/health;
    }
}
```

### 배포 명령

```bash
# 이미지 빌드
docker-compose build

# 서비스 시작
docker-compose up -d

# 스케일 아웃
docker-compose up -d --scale mecab-api=5

# 로그 확인
docker-compose logs -f mecab-api

# 상태 확인
docker-compose ps
```

## 클라이언트 예제

### Python 클라이언트

```python
import requests
from typing import List, Dict

class MecabClient:
    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url

    def analyze(self, text: str) -> Dict:
        response = requests.post(
            f"{self.base_url}/analyze",
            json={"text": text}
        )
        response.raise_for_status()
        return response.json()

    def batch_analyze(self, texts: List[str]) -> Dict:
        response = requests.post(
            f"{self.base_url}/batch",
            json={"texts": texts}
        )
        response.raise_for_status()
        return response.json()

# 사용
client = MecabClient()
result = client.analyze("안녕하세요")
print(result)
```

### JavaScript 클라이언트

```javascript
class MecabClient {
  constructor(baseUrl = 'http://localhost:8080') {
    this.baseUrl = baseUrl;
  }

  async analyze(text) {
    const response = await fetch(`${this.baseUrl}/analyze`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text })
    });
    return response.json();
  }

  async batchAnalyze(texts) {
    const response = await fetch(`${this.baseUrl}/batch`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ texts })
    });
    return response.json();
  }
}

// 사용
const client = new MecabClient();
const result = await client.analyze('안녕하세요');
console.log(result);
```

## 다음 단계

- [성능 튜닝](../advanced/performance-tuning.md): 서버 최적화
- [Elasticsearch 통합](../advanced/elasticsearch.md): 검색 엔진 연동
- [벤치마크 가이드](../benchmarks/guide.md): 성능 측정
