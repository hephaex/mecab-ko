# PROGRESS — mecab-ko Sprint 152 (Node/WASM CI 강화)

> 마지막 업데이트: 2026-05-21

## Sprint 152 D — Node/WASM CI continue-on-error 정리

| Task | 상태 | 결과 |
|------|------|------|
| S152-D1: e2e-ffi-tests.yml 전체 분석 | ✅ 완료 | 7개 continue-on-error 식별 |
| S152-D2: 빌드 잡 hard-gate 검증 | ✅ 완료 | node-bindings/wasm-bindings 이미 hard-gated |
| S152-D3: 불필요한 continue-on-error 제거 | ✅ 완료 | 2개 제거, 5개 정당화 유지 |
| S152-D4: YAML 문법 검증 | ✅ 완료 | yaml.safe_load 통과 |
| S152-D5: agents.md 규칙 5 추가 | ✅ 완료 | 자동 트랙 선택 (전문가 리뷰 기반) |

## 변경 내용

### 제거된 continue-on-error (2개)

#### 1. nodejs-e2e 빌드 단계 (L161-166)

**Before**:
```yaml
- name: Build Node.js binding
  working-directory: rust/crates/mecab-ko-node
  run: |
    npm install
    npm run build || echo "Build not configured yet"
  continue-on-error: true
```

**After**:
```yaml
- name: Build Node.js binding
  working-directory: rust/crates/mecab-ko-node
  # napi-rs build is hard-gated in `node-bindings` job below; this step
  # is duplicate verification on multi-OS/Node-version matrix.
  run: |
    npm install
    npm run build
```

**근거**:
- `node-bindings` 잡 (L291)에서 `npm run build`가 이미 hard-gated
- 잘못된 `|| echo "Build not configured yet"` 메시지 제거 (build는 실제로 설정됨)
- continue-on-error 제거 — 멀티 OS/Node 매트릭스에서도 hard fail

#### 2. wasm-e2e 테스트 단계 (L208)

**Before**:
```yaml
- name: Run WASM E2E tests
  working-directory: tests/e2e/wasm
  run: npm test
  continue-on-error: true
```

**After**:
```yaml
- name: Run WASM E2E tests
  working-directory: tests/e2e/wasm
  # Step-level continue-on-error redundant: job already has it.
  run: npm test
```

**근거**: 잡 레벨에 이미 `continue-on-error: true` (L180) — step 레벨은 중복.

### 정당화 유지 (5개 — 명시적 코멘트 추가)

| Line | Job | 사유 |
|------|-----|------|
| L77 | cli-tests bats Windows 설치 | chocolatey 설치 불안정 (OS 특이) |
| L178 | nodejs-e2e 테스트 단계 | E2E 환경 차이 가능 (skip 패턴 사용) |
| L185 | wasm-e2e 잡 레벨 | wasm-pack installer 불안정 |
| L218 | e2e-coverage 잡 레벨 | informational |
| L246 | e2e-coverage 업로드 | informational |

### Hard-gated 잡 검증

`test-status` 잡 (L371-412)이 강제하는 잡:
- ✅ `python-bindings` (continue-on-error 없음)
- ✅ `node-bindings` (`npm run build` hard-gated)
- ✅ `wasm-bindings` (`wasm-pack build --target bundler` hard-gated)
- ✅ `elasticsearch-plugin`

이 4개가 실패하면 `test-status`가 `exit 1` → PR 머지 차단.

## agents.md 규칙 5 신규

```markdown
5. **여러 후보 중 선택 시 도메인 전문가 에이전트 리뷰로 자동 결정**
   - 사용자에게 "어느 트랙으로 진행할까요?" 묻지 말 것
   - 적절한 도메인 전문가 에이전트 호출
   - 전문가 리뷰의 Top 권고를 자동 채택
   - 결정 근거를 PROGRESS.md에 기록
   - 예외: 비가역적 대규모 작업은 사전 confirm
```

향후 sprint planning에 적용 (Sprint 153~).

## 검증

- `python3 -c "import yaml; yaml.safe_load(...)"`: **PASS** (YAML 유효)
- 변경 파일: `.github/workflows/e2e-ffi-tests.yml` (CI workflow만 수정, 코드 변경 없음)
- 로컬 영향: 없음 (CI workflow는 push/PR 시점에만 실행)

## Sprint 153 후보 (자동 결정 예정)

- E: XSA+ETM 38건 분석 (스러운/스런/로운, ㅂ 불규칙)
- B: Full CRF Retrain (Track B 데이터 준비, 3-5 sprint)
- F: 신규 — 추가 정확도 영역 (전문가 리뷰로 식별)
