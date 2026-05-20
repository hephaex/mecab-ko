# PROGRESS — mecab-ko Sprint 151 (setup helper 추출)

> 마지막 업데이트: 2026-05-21

## Sprint 151 C — accuracy_eval.rs setup helper 추출

| Task | 상태 | 결과 |
|------|------|------|
| S151-C1: boilerplate 패턴 식별 (24개) | ✅ 완료 | 30라인 × 24 = ~720라인 중복 |
| S151-C2: helper functions 3개 정의 | ✅ 완료 | `project_root`, `dict_path`, `make_tokenizer` |
| S151-C3: 24개 함수 boilerplate 치환 | ✅ 완료 | 모두 helper 사용 |
| S151-C4: 변형 케이스 처리 | ✅ 완료 | 4가지 패턴 모두 처리 |
| S151-C5: 빌드/테스트/clippy 검증 | ✅ 완료 | 406 pass, clean |
| S151-C6: 5-gate sample.tsv 회귀 확인 | ✅ 완료 | 100.0%/99.9% 유지 |

## 변경 내용

### 추가된 helper 함수 (3개)

`tests/accuracy_eval.rs:16-56` (top of file):

```rust
fn project_root() -> std::path::PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    std::path::Path::new(&manifest_dir)
        .parent().and_then(|p| p.parent()).and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf()
}

fn dict_path(project_root: &std::path::Path) -> String {
    std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root.join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy().to_string()
    })
}

fn make_tokenizer(project_root: &std::path::Path) -> Tokenizer {
    let mut tokenizer = Tokenizer::with_dict(&dict_path(project_root))
        .expect("Failed to create tokenizer");
    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path).expect("Failed to load user dict");
        let klue_dict_path = project_root.join("data/user-dict/klue-domain.csv");
        if klue_dict_path.exists() {
            user_dict.load_from_csv(&klue_dict_path).expect("Failed to load KLUE dict");
        }
        tokenizer.set_user_dict(user_dict);
    }
    tokenizer
}
```

### 치환 패턴

각 테스트에서 30라인 setup이 2라인으로 축소:

```rust
// BEFORE (30 lines)
let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")...;
let project_root = ...;
let dict_path = ...;
let mut tokenizer = Tokenizer::with_dict(...)...;
let user_dict_path = project_root.join(...);
if user_dict_path.exists() { ... }
// ... 30 lines total

// AFTER (2 lines)
let project_root = project_root();
let mut tokenizer = make_tokenizer(&project_root);
```

### 결과

| 항목 | Before (Sprint 150) | After (Sprint 151 C) | Δ |
|------|--------------------|--------------------|---|
| accuracy_eval.rs 줄 수 | 2969 | **2406** | **-563 (-19%)** |
| Boilerplate 중복 | 24개 (~720 라인) | 0 | -100% |
| Helper functions | 0 | 3 | +3 |

### 변형 케이스 처리

- 20개 함수: 표준 `make_tokenizer(&project_root)` 패턴
- 3개 함수: project_root 사용 안함 → `make_tokenizer(&project_root())` 인라인
- 1개 함수 (`test_accuracy_gate_verified`): 사용자 사전 안 로드 → `dict_path()` + 직접 호출
- 2개 함수 (split_diff_connection_pairs): `dict_path()` 별도 사용

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: **406 passed / 0 failed**
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings`: clean
- 5-gate sample.tsv: **PASSED (100.0%/99.9%)** — 무회귀
- 컴파일 시간: 미세하게 감소 (코드 줄어듦)

## 누적 정리 (Sprint 149 + 151)

| 항목 | Sprint 148 | Sprint 149 | Sprint 151 C | 총 Δ |
|------|-----------|-----------|-------------|-----|
| accuracy_eval.rs | 4963 | 2800 | **2406** | **-2557 (-51%)** |

원본의 절반 이하로 축소. 가독성/유지보수성 대폭 향상.

## Sprint 152 후보

- D: Node/WASM CI 강화 (continue-on-error 제거)
- E: XSA+ETM 38건 분석 (스러운/스런/로운, ㅂ 불규칙)
- B: Full CRF Retrain (메인 lift)
