# Session Log: npm WASM 배포 완료

## 날짜: 2026-03-03

## 세션 개요
Sprint 16의 S16-04 작업으로 mecab-ko-wasm 패키지를 npm에 성공적으로 배포했습니다.

## 주요 작업

### 1. Git 대용량 파일 문제 해결
- **문제**: `data/mecab-ko-dic-2.1.1-20180720/matrix.def` (137MB)가 GitHub 100MB 제한 초과
- **해결**:
  - `git filter-repo`로 대용량 파일 히스토리에서 제거
  - `.gitignore` 업데이트하여 대용량 파일 제외
  ```
  data/dict-output/
  data/mecab-ko-dic-2.1.1-20180720/
  data/mecab-ko-dic.tar.gz
  mecab-jumandic/
  mecab-ipadic-neologd/
  ```
- force push로 GitHub 원격 저장소 업데이트

### 2. npm 배포 워크플로우 수정
- **문제**: Rust 1.75가 `wit-bindgen v0.51.0`의 Edition 2024 미지원
- **해결**: `.github/workflows/npm-publish-wasm.yml`에서 Rust 버전을 `stable`로 변경
  ```yaml
  env:
    RUST_VERSION: "stable"  # 1.75 → stable
  ```

### 3. npm 2FA 인증 문제 해결
- **문제**: `E403 - Two-factor authentication or granular access token with bypass 2fa enabled is required`
- **해결**:
  - npm에서 Granular Access Token 생성 (`github-actions-publish`)
  - GitHub Secret `NPM_TOKEN` 업데이트

### 4. v0.3.0 태그 배포
- `git tag v0.3.0 && git push origin v0.3.0`
- 워크플로우 자동 트리거로 npm 배포 완료

## 결과

### npm 패키지 정보
- **이름**: `mecab-ko-wasm`
- **버전**: `0.3.0`
- **크기**: 127.7 kB (unpacked)
- **라이선스**: MIT OR Apache-2.0
- **URL**: https://www.npmjs.com/package/mecab-ko-wasm

### 설치 방법
```bash
npm install mecab-ko-wasm
```

### 사용 예시
```javascript
import { initialize, tokenize, nouns, pos } from 'mecab-ko-wasm';

await initialize();

const result = tokenize('아버지가방에들어가신다');
console.log(result);

const nounList = nouns('한국어 형태소 분석기');
console.log(nounList);
```

## 파일 변경 내역

### 수정된 파일
| 파일 | 변경 내용 |
|------|----------|
| `.gitignore` | 대용량 파일 제외 패턴 추가 |
| `.github/workflows/npm-publish-wasm.yml` | Rust 버전 stable로 변경 |
| `PLAN.md` | S16-04 완료 표시 |
| `PROGRESS.md` | npm 배포 완료 정보 업데이트 |

### Git 커밋
1. `chore: update .gitignore to exclude large dictionary files`
2. `fix(ci): update Rust version to stable for Edition 2024 support`
3. `docs: update PLAN.md and PROGRESS.md for npm deployment completion (S16-04)`

### Git 태그
- `v0.3.0` - npm 배포용 태그

## 워크플로우 실행 기록

| Run ID | 상태 | 설명 |
|--------|------|------|
| 22599345247 | ❌ | Rust 1.75 Edition 2024 미지원 |
| 22599524384 | ✅ | dry-run 성공 |
| 22599610261 | ❌ | 2FA 토큰 문제 |
| 22600002466 | ✅ | dry-run=false (태그 없어서 스킵) |
| 22600043928 | ✅ | **v0.3.0 태그로 npm 배포 성공** |

## 교훈 (Lessons Learned)

### 1. Git 대용량 파일 관리
- GitHub는 100MB 파일 제한이 있음
- `git filter-repo`가 `filter-branch`보다 빠르고 안전함
- 대용량 파일은 미리 `.gitignore`에 추가할 것

### 2. npm 자동화 배포
- Granular Access Token + 2FA bypass 필요
- 태그 푸시로 자동 배포 트리거 (`refs/tags/v*`)
- dry-run으로 먼저 테스트 권장

### 3. Rust 버전 관리
- 최신 크레이트는 Edition 2024 요구 가능
- CI에서 `stable` 사용이 안전함

## Sprint 16 최종 현황

| Task | Status |
|------|--------|
| S16-01: N-best 경로 탐색 개선 | ✅ |
| S16-02: 사용자 정의 분석 모드 | ✅ |
| S16-03: PyPI 배포 | ⏸️ BLOCKED |
| S16-04: npm 배포 | ✅ |
| S16-05: Lattice 시각화 도구 | ✅ |
| S16-06: 토큰화 캐싱 | ✅ |
| S16-07: 병렬 토큰화 | ✅ |
| S16-08: v0.3.0 준비 | ✅ |

## 다음 단계
1. S16-03: PyPI 배포 (PyPI 토큰 설정 필요)
2. Sprint 17 계획 수립
3. v0.3.0 정식 릴리스 노트 작성

## 참고 링크
- npm 패키지: https://www.npmjs.com/package/mecab-ko-wasm
- GitHub Actions: https://github.com/hephaex/mecab-ko/actions
- 워크플로우: `.github/workflows/npm-publish-wasm.yml`
