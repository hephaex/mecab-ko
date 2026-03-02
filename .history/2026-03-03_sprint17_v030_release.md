# Sprint 17 - v0.3.0 정식 릴리스 (2026-03-03)

## 세션 개요
Sprint 17 시작 및 S17-01: v0.3.0 정식 릴리스 완료

## 완료된 작업

### S17-01: v0.3.0 정식 릴리스 ✅

#### crates.io 발행 (6개 크레이트)
의존성 순서대로 발행 완료:
1. `mecab-ko-hangul` v0.3.0
2. `mecab-ko-dict` v0.3.0
3. `mecab-ko-core` v0.3.0
4. `mecab-ko-dict-validator` v0.3.0
5. `mecab-ko-dict-builder` v0.3.0
6. `mecab-ko` v0.3.0 (facade)

#### 버전 업데이트 과정
1. workspace 버전 0.2.0 → 0.3.0 (rust/Cargo.toml)
2. 개별 크레이트 path dependency 버전 동기화
3. `mecab-ko-dict-builder` optional dependency 버전 수정 (누락 발견 및 수정)

#### GitHub Release
- v0.3.0 태그 및 릴리스 이미 존재 (Sprint 16에서 생성)
- 릴리스 노트 업데이트 완료

#### CHANGELOG.md
- `[Unreleased] - v0.3.0` → `[0.3.0] - 2026-03-03` 확정

## 기술적 결정

### 버전 동기화 전략
- 모든 workspace 멤버가 동일한 버전 사용 (`version.workspace = true`)
- crates.io 발행 시 의존성 순서 준수 필수
- optional dependency도 버전 확인 필요

### 발행 순서
```
hangul (의존성 없음)
  ↓
dict (hangul 의존)
  ↓
core (hangul, dict 의존)
  ↓
dict-validator (hangul 의존)
  ↓
dict-builder (dict 의존)
  ↓
mecab-ko facade (core, dict, hangul, builder 의존)
```

## 변경된 파일
- `rust/Cargo.toml` - workspace 버전 0.3.0
- `rust/crates/*/Cargo.toml` - 의존성 버전 동기화
- `CHANGELOG.md` - v0.3.0 릴리스 날짜 확정
- `PLAN.md` - S17-01 완료 표시, 버전 테이블 업데이트
- `PROGRESS.md` - Sprint 17 진행 상황 추가

## 커밋
```
chore: bump all crate versions to 0.3.0 for crates.io publish
```

## 배포 현황

| 플랫폼 | 패키지 | 버전 | 상태 |
|--------|--------|------|------|
| crates.io | mecab-ko | 0.3.0 | ✅ |
| crates.io | mecab-ko-core | 0.3.0 | ✅ |
| crates.io | mecab-ko-dict | 0.3.0 | ✅ |
| crates.io | mecab-ko-hangul | 0.3.0 | ✅ |
| crates.io | mecab-ko-dict-builder | 0.3.0 | ✅ |
| crates.io | mecab-ko-dict-validator | 0.3.0 | ✅ |
| npm | mecab-ko-wasm | 0.3.0 | ✅ |
| PyPI | mecab-ko-python | - | BLOCKED |

## 다음 작업
- S17-02: PyPI 배포 (토큰 필요)
- S17-03: 스트리밍 API 개선
- S17-04: Migration Guide v0.2.0 → v0.3.0

## 학습 포인트
1. crates.io 발행 시 optional dependency 버전도 확인 필수
2. workspace 버전 관리 시 `version.workspace = true` 패턴 권장
3. 의존성 순서 준수하여 발행 (leaf → root)
