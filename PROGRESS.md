# PROGRESS — mecab-ko Sprint 159 (NIKL Modu 인프라 준비)

> 마지막 업데이트: 2026-05-21

## Sprint 159 F — NIKL Modu silver dataset 통합 (인프라 준비)

| Task | 상태 | 결과 |
|------|------|------|
| S159-F1: 기존 변환 스크립트 패턴 조사 | ✅ 완료 | convert_ud_kaist/gsd.py 참조 |
| S159-F2: convert_nikl_modu.py 작성 | ✅ 완료 | NIKL JSON → TSV 변환 |
| S159-F3: accuracy test 추가 (skip 패턴) | ✅ 완료 | `test_nikl_modu_dual_metric` |
| S159-F4: skip 동작 검증 | ✅ 완료 | 파일 미존재 시 안내 메시지 출력 + pass |
| S159-F5: 다운로드/설정 문서 작성 | ✅ 완료 | `docs/eval/nikl_modu_setup.md` |
| S159-F6: `.gitignore` 추가 | ✅ 완료 | nikl_modu_*.tsv 재배포 금지 |
| S159-F7: 사용자 다운로드 (수동) | ⏸ 사용자 작업 | https://kli.korean.go.kr 등록 필요 |

## 변경 내용

### 1. 변환 스크립트 (`tools/convert_nikl_modu.py`)

- NIKL Modu JSON → mecab-ko TSV 형식 변환
- compound POS (X+Y) 처리, 미확인 태그 경고
- position-based text reconstruction
- 5000 sentences 기본 (옵션 조정 가능)

### 2. Accuracy 테스트 (skip 패턴)

```rust
fn test_nikl_modu_dual_metric() {
    let eval_path = project_root.join("data/eval/nikl_modu_sample.tsv");
    if !eval_path.exists() {
        println!("Skipping: ...");  // 다운로드 안내
        return;
    }
    // 정상 dual metric 측정
}
```

dataset 미존재 시 자동 skip, 5-gate CI에 영향 없음.

### 3. 문서 (`docs/eval/nikl_modu_setup.md`)

- 다운로드 방법 (kli.korean.go.kr 등록 → 학술 승인)
- 변환 명령
- 평가 실행 방법
- License 안내 (재배포 금지)

### 4. `.gitignore` 보호

```
data/eval/nikl_modu_*.tsv
```

학술 license 데이터 실수 commit 방지.

## NIKL Modu 정보

- **규모**: 371,571 sentences
- **POS scheme**: Sejong-compatible (직접 호환)
- **도메인**: 신문 / 웹 / 구어 / 문어 (multi-domain)
- **License**: 학술 사용 전용 (재배포 금지)
- **포털**: https://kli.korean.go.kr

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (411 pass)
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings`: clean
- `test_nikl_modu_dual_metric --ignored`: PASS (skip 정상)
- 5-gate sample.tsv: 영향 없음

## 변경 파일

- `tools/convert_nikl_modu.py` (신규, 154줄)
- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`: `test_nikl_modu_dual_metric` 추가 (skip 패턴)
- `docs/eval/nikl_modu_setup.md` (신규)
- `.gitignore`: NIKL Modu pattern 추가
- `PLAN.md`, `PROGRESS.md` 갱신

## 사용자 작업 (Sprint 159 이후)

1. https://kli.korean.go.kr 학술 등록
2. NIKL Modu 형태분석 다운로드 (JSON)
3. 변환:
   ```bash
   python3 tools/convert_nikl_modu.py <input.json> data/eval/nikl_modu_sample.tsv
   ```
4. 평가 실행:
   ```bash
   cd rust && cargo test --package mecab-ko-core --test accuracy_eval \
     -- test_nikl_modu_dual_metric --nocapture --ignored
   ```

## Sprint 160 후보

다음 sprint 결정은 사용자 선택:

### NIKL Modu 다운로드 완료 시
- 측정 + practical 동치/normalize 추가 후보 발굴
- 6번째 silver gate 활성 (로컬 only)

### CRF Retrain 결정 시
- Track B (Full CRF Retrain) 3-5 sprint 시작
- 잠재 lift +1~5pp

### 정확도 외 영역 전환
- 문서 정리
- CLI/API 사용성
- 성능 최적화
