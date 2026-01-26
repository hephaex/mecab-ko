# MeCab-Ko 핫 리로드 (Hot Reload) 기능

실시간 사전 업데이트를 위한 무중단 핫 리로드 시스템입니다.

## 목차

- [개요](#개요)
- [주요 기능](#주요-기능)
- [아키텍처](#아키텍처)
- [사용 방법](#사용-방법)
- [API 레퍼런스](#api-레퍼런스)
- [CLI 명령어](#cli-명령어)
- [성능 특성](#성능-특성)
- [제한사항](#제한사항)

## 개요

핫 리로드 기능은 서비스 중단 없이 사전을 실시간으로 업데이트할 수 있게 합니다.

### 주요 사용 사례

- **실시간 서비스**: 서버 재시작 없이 사전 업데이트
- **A/B 테스팅**: 버전 관리를 통한 사전 비교
- **동적 사전**: 사용자 피드백을 즉시 반영
- **개발/디버깅**: 빠른 반복 개발

## 주요 기능

### 1. 무중단 사전 교체

```rust
use mecab_ko_dict::HotReloadDictionary;

let dict = HotReloadDictionary::new("/path/to/dict")?;

// 실시간 엔트리 추가 (읽기 작업에 영향 없음)
dict.add_entry("딥러닝", "NNG", -1000, None)?;
```

### 2. 파일 변경 감지

```rust
use mecab_ko_dict::{HotReloadDictionary, FileWatcher};
use std::sync::Arc;

let dict = Arc::new(HotReloadDictionary::new("/path/to/dict")?);
let mut watcher = FileWatcher::new_default(dict.clone())?;

// 자동 리로드 시작
watcher.start()?;

// 파일 변경 시 자동으로 사전 리로드
// ...

watcher.stop()?;
```

### 3. 버전 관리 및 롤백

```rust
// 버전 1
let v1 = dict.current_version();

// 버전 2
dict.add_entry("A", "NNG", 0, None)?;

// 버전 3
dict.add_entry("B", "NNG", 0, None)?;

// 버전 1로 롤백
dict.rollback(v1)?;
```

### 4. 델타 업데이트

```rust
use mecab_ko_dict::DeltaUpdate;

// 여러 변경사항을 하나의 트랜잭션으로 처리
let delta = DeltaUpdate::builder()
    .add("딥러닝", "NNG", -1000)
    .add("머신러닝", "NNG", -1000)
    .add("자연어처리", "NNG", -1000)
    .remove("구식단어")
    .modify("기존단어", "NNG", -500)
    .build();

dict.apply_delta(delta)?;
```

## 아키텍처

### Copy-on-Write 전략

```text
┌─────────────────────────────────────────┐
│ HotReloadDictionary                     │
│  └─ RwLock<VersionedDictionary>         │
│      ├─ Version: u64                    │
│      ├─ SystemDict: Arc<...> (공유)     │
│      └─ UserDict: Arc<...> (복사)       │
└─────────────────────────────────────────┘

업데이트 시:
1. 현재 UserDict를 복사 (Arc clone)
2. 복사본에 변경 적용
3. 새 버전 생성
4. RwLock write로 원자적 교체
5. 기존 읽기 작업은 영향 받지 않음
```

### 동시성 모델

- **읽기**: `RwLock::read()` - 여러 스레드 동시 접근 가능
- **쓰기**: `RwLock::write()` - 배타적 접근, 짧은 시간만 잠금
- **메모리**: Arc를 통한 참조 카운팅, 자동 메모리 해제

## 사용 방법

### 기본 사용법

```rust
use mecab_ko_dict::HotReloadDictionary;

// 1. 사전 생성
let dict = HotReloadDictionary::new("/path/to/dict")?;

// 2. 엔트리 추가
dict.add_entry("새단어", "NNG", -1000, None)?;

// 3. 조회
let entries = dict.lookup("새단어")?;
for entry in entries {
    println!("{}: {}", entry.surface, entry.feature);
}

// 4. 엔트리 제거
dict.remove_entry("새단어")?;
```

### 멀티스레드 환경

```rust
use std::sync::Arc;
use std::thread;

let dict = Arc::new(HotReloadDictionary::new("/path/to/dict")?);

// 읽기 스레드
let dict_clone = Arc::clone(&dict);
let reader = thread::spawn(move || {
    loop {
        let _ = dict_clone.lookup("테스트");
    }
});

// 쓰기 스레드
let dict_clone = Arc::clone(&dict);
let writer = thread::spawn(move || {
    dict_clone.add_entry("새단어", "NNG", 0, None).unwrap();
});

reader.join().unwrap();
writer.join().unwrap();
```

### 파일 감시

```rust
use mecab_ko_dict::{HotReloadDictionary, FileWatcher, WatchConfig};
use std::sync::Arc;

let dict = Arc::new(HotReloadDictionary::new("/path/to/dict")?);

// 커스텀 설정
let config = WatchConfig::default()
    .debounce_ms(500)              // 500ms 디바운스
    .watch_extension("csv")        // CSV 파일도 감시
    .ignore_pattern(".backup");    // .backup 파일 무시

let mut watcher = FileWatcher::new(dict.clone(), config)?;
watcher.start()?;

// 백그라운드에서 자동 리로드
// ...

watcher.stop()?;
```

## API 레퍼런스

### HotReloadDictionary

#### 생성

- `new(dicdir: impl AsRef<Path>) -> Result<Self>`
- `new_default() -> Result<Self>`
- `with_max_history(max_history: usize) -> Self`
- `with_max_delta_queue(max_delta_queue: usize) -> Self`

#### 조회

- `lookup(&self, surface: &str) -> Result<Vec<Entry>>`
- `current_version(&self) -> Version`
- `dicdir(&self) -> &Path`

#### 엔트리 관리

- `add_entry(&self, surface: impl Into<String>, pos: impl Into<String>, cost: i16, reading: Option<String>) -> Result<Version>`
- `remove_entry(&self, surface: &str) -> Result<(Version, usize)>`
- `update_entry<F>(&self, surface: &str, update_fn: F) -> Result<Version>`

#### 델타 업데이트

- `apply_delta(&self, delta: DeltaUpdate) -> Result<Version>`
- `delta_history(&self) -> Result<Vec<DeltaUpdate>>`

#### 버전 관리

- `reload_system_dict(&self) -> Result<Version>`
- `rollback(&self, target_version: Version) -> Result<()>`
- `version_history(&self) -> Result<Vec<VersionInfo>>`

#### 사전 관리

- `export_user_dict(&self) -> Result<UserDictionary>`
- `import_user_dict(&self, user_dict: UserDictionary) -> Result<Version>`

### DeltaUpdate

#### 빌더

```rust
let delta = DeltaUpdate::builder()
    .add(surface, pos, cost)
    .add_with_reading(surface, pos, cost, reading)
    .remove(surface)
    .modify(surface, pos, cost)
    .build();
```

#### 메서드

- `addition_count(&self) -> usize`
- `removal_count(&self) -> usize`
- `modification_count(&self) -> usize`
- `total_changes(&self) -> usize`

### FileWatcher

#### 생성

- `new(dict: Arc<HotReloadDictionary>, config: WatchConfig) -> Result<Self>`
- `new_default(dict: Arc<HotReloadDictionary>) -> Result<Self>`

#### 제어

- `start(&mut self) -> Result<()>`
- `stop(&mut self) -> Result<()>`
- `is_watching(&self) -> bool`

## CLI 명령어

### 기본 명령어

```bash
# 사전 리로드
mecab-ko dict reload

# 엔트리 추가
mecab-ko dict add "딥러닝" NNG --cost -1000
mecab-ko dict add "챗GPT" NNP --cost -2000 --reading "챗지피티"

# 엔트리 제거
mecab-ko dict remove "구식단어"

# 사용자 사전 목록
mecab-ko dict list
mecab-ko dict list --pattern "딥"

# 버전 정보
mecab-ko dict version
mecab-ko dict version --history

# 롤백
mecab-ko dict rollback 3

# 사전 내보내기/가져오기
mecab-ko dict export user_dict.csv
mecab-ko dict import user_dict.csv

# 사전 초기화
mecab-ko dict clear --yes

# 사전 정보
mecab-ko dict info
```

### 사전 경로 지정

```bash
# 특정 경로의 사전 사용
mecab-ko dict add "단어" NNG --dicdir /custom/dict/path

# 환경변수로 지정
export MECAB_DICDIR=/custom/dict/path
mecab-ko dict add "단어" NNG
```

## 성능 특성

### 읽기 성능

- **Lock Contention**: RwLock 사용으로 최소화
- **복잡도**: O(1) - Arc 역참조만 필요
- **지연**: 나노초 단위 (lock 획득 시간)
- **확장성**: 읽기 스레드 수에 선형 확장

### 쓰기 성능

- **복잡도**: O(N) - N은 사용자 사전 크기
- **메모리**: 사용자 사전 크기만큼 추가 할당
- **지연**: 마이크로초 단위 (작은 사전 기준)
- **영향**: 기존 읽기 작업에 영향 없음

### 델타 업데이트 최적화

```rust
// 비효율적: 여러 번의 쓰기 잠금
dict.add_entry("A", "NNG", 0, None)?;
dict.add_entry("B", "NNG", 0, None)?;
dict.add_entry("C", "NNG", 0, None)?;

// 효율적: 한 번의 쓰기 잠금
let delta = DeltaUpdate::builder()
    .add("A", "NNG", 0)
    .add("B", "NNG", 0)
    .add("C", "NNG", 0)
    .build();
dict.apply_delta(delta)?;
```

### 메모리 사용

- **버전당 메모리**: 사용자 사전 크기 (Arc 공유로 최소화)
- **히스토리 제한**: 기본 10개 버전 (설정 가능)
- **자동 정리**: Arc 참조 카운트 0이 되면 자동 해제

### 벤치마크 (참고용)

```text
환경: Intel Core i7, 16GB RAM
사용자 사전: 1,000 엔트리

읽기 (lookup):           ~50ns
쓰기 (add_entry):        ~2μs
델타 업데이트 (100건):   ~200μs
버전 롤백:               ~100ns
```

## 제한사항

### 1. 시스템 사전

- **읽기 전용**: 시스템 사전은 파일 리로드로만 업데이트 가능
- **리로드 비용**: 전체 사전 재로드 필요 (수백 ms)

### 2. 메모리

- **버전 히스토리**: 메모리에 유지 (기본 10개)
- **델타 큐**: 메모리에 유지 (기본 100개)
- **제한 설정**: `with_max_history()`, `with_max_delta_queue()`

### 3. 동시성

- **쓰기 직렬화**: 동시 쓰기는 직렬화됨
- **Lock Starvation**: 많은 쓰기 시 읽기 대기 가능
- **권장**: 델타 업데이트로 쓰기 횟수 최소화

### 4. 파일 감시

- **플랫폼 의존**: OS별 파일 시스템 이벤트 사용
- **디바운싱**: 짧은 시간 내 여러 이벤트 무시
- **네트워크 파일**: NFS/CIFS에서 동작 안 할 수 있음

## 예제

### 1. 웹 서버 통합

```rust
use axum::{Router, routing::post, Json};
use mecab_ko_dict::HotReloadDictionary;
use std::sync::Arc;

#[derive(Deserialize)]
struct AddEntryRequest {
    surface: String,
    pos: String,
    cost: i16,
}

async fn add_entry_handler(
    dict: Arc<HotReloadDictionary>,
    Json(req): Json<AddEntryRequest>,
) -> Result<Json<u64>, String> {
    let version = dict
        .add_entry(&req.surface, &req.pos, req.cost, None)
        .map_err(|e| e.to_string())?;
    Ok(Json(version))
}

#[tokio::main]
async fn main() {
    let dict = Arc::new(HotReloadDictionary::new_default().unwrap());

    let app = Router::new()
        .route("/dict/add", post(add_entry_handler))
        .with_state(dict);

    // 서버 실행...
}
```

### 2. 주기적 사전 업데이트

```rust
use mecab_ko_dict::HotReloadDictionary;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() {
    let dict = Arc::new(HotReloadDictionary::new_default().unwrap());
    let dict_clone = Arc::clone(&dict);

    // 백그라운드 태스크: 1시간마다 사전 리로드
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            if let Err(e) = dict_clone.reload_system_dict() {
                eprintln!("Failed to reload dictionary: {}", e);
            }
        }
    });

    // 메인 서비스...
}
```

### 3. 테스트 환경

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_feedback_integration() {
        let dict = HotReloadDictionary::new_default().unwrap();

        // 사용자 피드백으로 신조어 추가
        dict.add_entry("챗GPT", "NNP", -2000, None).unwrap();

        // 즉시 조회 가능
        let entries = dict.lookup("챗GPT").unwrap();
        assert_eq!(entries.len(), 1);

        // 잘못된 엔트리 제거
        dict.remove_entry("오타단어").unwrap();

        // 비용 조정
        dict.update_entry("챗GPT", |e| e.cost = -3000).unwrap();
    }
}
```

## 문제 해결

### Q: "Failed to acquire write lock" 에러

**원인**: 쓰기 잠금 획득 실패 (드물게 발생)

**해결**:
```rust
// 재시도 로직 추가
for _ in 0..3 {
    match dict.add_entry("word", "NNG", 0, None) {
        Ok(v) => break,
        Err(_) => std::thread::sleep(Duration::from_millis(10)),
    }
}
```

### Q: 메모리 사용량 증가

**원인**: 버전 히스토리 축적

**해결**:
```rust
// 히스토리 크기 제한
let dict = HotReloadDictionary::new("/path")
    .unwrap()
    .with_max_history(5);  // 5개 버전만 유지
```

### Q: 파일 감시가 동작하지 않음

**원인**: 플랫폼별 파일 시스템 이벤트 지원 차이

**해결**:
```rust
// 수동 리로드로 대체
loop {
    std::thread::sleep(Duration::from_secs(60));
    dict.reload_system_dict()?;
}
```

## 라이선스

MIT OR Apache-2.0
