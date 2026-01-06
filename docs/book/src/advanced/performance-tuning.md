# 성능 튜닝

MeCab-Ko의 성능을 최적화하는 방법을 소개합니다.

## 벤치마크 기준

테스트 환경:
- CPU: AMD Ryzen 9 5950X (16 cores)
- RAM: 64GB DDR4-3200
- OS: Ubuntu 22.04 LTS
- Rust: 1.75.0

기준 성능:
- 단일 스레드: ~15MB/s
- 멀티 스레드 (16 cores): ~180MB/s
- 메모리 사용량: ~120MB (사전 로딩 포함)

## Tagger 설정 최적화

### 띄어쓰기 패널티 조정

```rust
use mecab_ko::{Tagger, TaggerConfig};

let mut config = TaggerConfig::default();

// 높은 패널티 = 더 많은 띄어쓰기 (빠름, 정확도 낮음)
config.space_penalty = -500;

// 낮은 패널티 = 적은 띄어쓰기 (느림, 정확도 높음)
config.space_penalty = -2000;

// 기본값: -1000 (균형)
config.space_penalty = -1000;
```

성능 영향:
- `-500`: 속도 +20%, 정확도 -5%
- `-1000`: 기준
- `-2000`: 속도 -15%, 정확도 +3%

### 최대 그룹화 크기

```rust
let mut config = TaggerConfig::default();

// 작은 크기 = 빠름, 긴 복합어 처리 약함
config.max_grouping_size = 12;

// 큰 크기 = 느림, 긴 복합어 처리 강함
config.max_grouping_size = 48;

// 기본값: 24 (권장)
config.max_grouping_size = 24;
```

### 부분 처리 모드

```rust
let mut config = TaggerConfig::default();

// 부분 처리 비활성화 (기본값, 더 빠름)
config.partial = false;

// 부분 처리 활성화 (미등록어 처리 강화, 느림)
config.partial = true;
```

## Tagger 재사용

### 잘못된 사용

```rust
// Bad: 매번 Tagger 생성 (매우 느림)
for text in texts.iter() {
    let tagger = Tagger::new(TaggerConfig::default())?; // 비효율적!
    let result = tagger.parse(text)?;
}
```

성능: ~0.5MB/s

### 올바른 사용

```rust
// Good: Tagger 재사용
let tagger = Tagger::new(TaggerConfig::default())?;
for text in texts.iter() {
    let result = tagger.parse(text)?;
}
```

성능: ~15MB/s (30배 빠름)

## 병렬 처리

### Rayon을 사용한 병렬화

```rust
use rayon::prelude::*;
use std::sync::Arc;

let tagger = Arc::new(Tagger::new(TaggerConfig::default())?);

let results: Vec<_> = texts
    .par_iter()
    .map(|text| {
        let tagger = Arc::clone(&tagger);
        tagger.parse(text)
    })
    .collect::<Result<_, _>>()?;
```

성능 (16 cores): ~180MB/s (12배 빠름)

### 수동 스레드 풀

```rust
use std::thread;
use std::sync::mpsc;

fn parallel_analyze(texts: Vec<String>, num_threads: usize) -> Vec<String> {
    let chunk_size = texts.len() / num_threads;
    let (tx, rx) = mpsc::channel();

    for chunk in texts.chunks(chunk_size) {
        let tx = tx.clone();
        let chunk = chunk.to_vec();

        thread::spawn(move || {
            let tagger = Tagger::new(TaggerConfig::default()).unwrap();
            for text in chunk {
                let result = tagger.parse(&text).unwrap();
                tx.send(result).unwrap();
            }
        });
    }

    drop(tx);
    rx.iter().collect()
}
```

## 배치 처리

### 내장 배치 API

```rust
let tagger = Tagger::new(TaggerConfig::default())?;

// 배치 처리 (더 효율적)
let results = tagger.parse_batch(&texts)?;
```

성능: +15% vs 개별 처리

### 커스텀 배치 크기

```rust
fn process_in_batches(
    tagger: &Tagger,
    texts: &[String],
    batch_size: usize,
) -> Result<Vec<String>, Error> {
    texts
        .chunks(batch_size)
        .flat_map(|batch| tagger.parse_batch(batch))
        .collect()
}

// 최적 배치 크기: 100-1000
let results = process_in_batches(&tagger, &texts, 500)?;
```

## 메모리 최적화

### 사전 메모리 매핑

```rust
let mut config = TaggerConfig::default();
config.use_mmap = true; // 메모리 사용량 감소

let tagger = Tagger::new(config)?;
```

메모리 사용량:
- 일반 로딩: ~120MB
- mmap: ~20MB (6배 감소)

성능 영향: -5% (메모리 절약 우선시)

### 사전 압축

사전 빌드 시 압축 활성화:

```bash
mecab-ko-dict-build --compression --compression-level 6
```

사전 크기:
- 비압축: ~250MB
- 압축 (level 6): ~85MB (3배 감소)

로딩 시간:
- 비압축: ~50ms
- 압축: ~80ms

### String Interning

반복되는 문자열을 인터닝:

```rust
use string_cache::DefaultAtom as Atom;

let mut seen = HashMap::new();

for node in nodes {
    let surface = seen
        .entry(node.surface.clone())
        .or_insert_with(|| Atom::from(node.surface.clone()));
}
```

메모리 절약: ~30% (대용량 데이터)

## I/O 최적화

### 버퍼링

```rust
use std::io::{BufRead, BufReader};
use std::fs::File;

let file = File::open("large_file.txt")?;
let reader = BufReader::with_capacity(1024 * 1024, file); // 1MB 버퍼

for line in reader.lines() {
    let line = line?;
    let result = tagger.parse(&line)?;
}
```

성능: +40% vs 버퍼 없음

### 비동기 I/O (Tokio)

```rust
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("large_file.txt").await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        let result = tagger.parse(&line)?;
        // 처리
    }

    Ok(())
}
```

## 출력 포맷 최적화

### Wakati 모드

품사 정보가 불필요한 경우:

```rust
let mut config = TaggerConfig::default();
config.output_format = OutputFormat::Wakati; // 빠름

let tagger = Tagger::new(config)?;
```

성능: +25% vs MeCab 포맷

### 커스텀 포맷

필요한 정보만 출력:

```rust
config.output_format = OutputFormat::Custom("%m\n".to_string());
```

성능: +30% vs 전체 feature 출력

## 프로파일링

### CPU 프로파일링

```bash
# perf를 사용한 프로파일링
cargo build --release
perf record --call-graph dwarf ./target/release/mecab-ko large_file.txt
perf report
```

### 메모리 프로파일링

```bash
# heaptrack 사용
heaptrack ./target/release/mecab-ko large_file.txt
heaptrack_gui heaptrack.mecab-ko.*.gz
```

### Flamegraph

```bash
cargo install flamegraph
cargo flamegraph --bin mecab-ko -- large_file.txt
```

## 컴파일 최적화

### Release 프로필

Cargo.toml:

```toml
[profile.release]
opt-level = 3              # 최대 최적화
lto = "fat"                # Link-Time Optimization
codegen-units = 1          # 단일 코드 생성 유닛
panic = "abort"            # panic 시 abort (작은 바이너리)
strip = true               # 디버그 심볼 제거
```

바이너리 크기: 15MB → 8MB
성능: +10%

### 타겟 CPU 최적화

```bash
# 현재 CPU에 최적화
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

성능: +5-15% (CPU 의존적)

### PGO (Profile-Guided Optimization)

```bash
# 1. 계측 빌드
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" cargo build --release

# 2. 대표 워크로드 실행
./target/release/mecab-ko large_corpus.txt

# 3. PGO 빌드
llvm-profdata merge -o /tmp/pgo-data/merged.profdata /tmp/pgo-data
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata" cargo build --release
```

성능: +15-20%

## 벤치마크

### Criterion 벤치마크

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mecab_ko::{Tagger, TaggerConfig};

fn bench_parse(c: &mut Criterion) {
    let tagger = Tagger::new(TaggerConfig::default()).unwrap();
    let text = "형태소 분석 벤치마크 테스트입니다.";

    c.bench_function("parse", |b| {
        b.iter(|| tagger.parse(black_box(text)))
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
```

실행:

```bash
cargo bench
```

### 처리량 측정

```rust
use std::time::Instant;

let tagger = Tagger::new(TaggerConfig::default())?;
let text = std::fs::read_to_string("large_corpus.txt")?;
let size = text.len();

let start = Instant::now();
tagger.parse(&text)?;
let elapsed = start.elapsed();

let throughput = size as f64 / elapsed.as_secs_f64() / 1_000_000.0;
println!("처리량: {:.2} MB/s", throughput);
```

## 실전 최적화 사례

### 웹 서버 최적화

```rust
use actix_web::{web, App, HttpServer};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 전역 Tagger (Arc로 공유)
    let tagger = Arc::new(Tagger::new(TaggerConfig::default()).unwrap());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tagger.clone()))
            .route("/analyze", web::post().to(analyze))
    })
    .workers(16) // CPU 코어 수
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
```

처리량: ~10,000 req/s

### 스트리밍 처리

```rust
use tokio::io::AsyncBufReadExt;

async fn stream_analyze(reader: impl AsyncBufRead + Unpin) {
    let tagger = Tagger::new(TaggerConfig::default()).unwrap();
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await.unwrap() {
        if !line.trim().is_empty() {
            tokio::task::spawn_blocking({
                let tagger = tagger.clone();
                let line = line.clone();
                move || tagger.parse(&line)
            })
            .await
            .unwrap()
            .unwrap();
        }
    }
}
```

## 성능 체크리스트

- [ ] Tagger 재사용 (매번 생성하지 않기)
- [ ] 병렬 처리 활용 (Rayon, 스레드 풀)
- [ ] 배치 처리 사용
- [ ] 메모리 매핑 활성화 (대용량 사전)
- [ ] 적절한 출력 포맷 선택
- [ ] Release 프로필 최적화
- [ ] 버퍼링 I/O 사용
- [ ] 불필요한 feature 비활성화
- [ ] PGO 적용 (중요 워크로드)
- [ ] 프로파일링으로 병목 지점 확인

## 성능 비교

| 구현 | 속도 (MB/s) | 메모리 (MB) |
|------|-------------|-------------|
| MeCab (C++) | 18 | 150 |
| **MeCab-Ko (Rust)** | **15** | **120** |
| Lindera | 12 | 180 |
| Kiwi | 22 | 200 |

## 참고 자료

- [Rust 성능 책](https://nnethercote.github.io/perf-book/)
- [Criterion.rs](https://github.com/bheisler/criterion.rs)
- [Rayon 문서](https://docs.rs/rayon/)
