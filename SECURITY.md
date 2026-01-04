# Security Policy

> **Project**: MeCab-Ko - Korean Morphological Analyzer in Rust  
> **Maintainer**: hephaex (hephaex@gmail.com)  
> **Repository**: https://github.com/hephaex/mecab-ko

---

## 지원 버전

현재 보안 업데이트가 지원되는 버전:

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | ✅ 지원             |
| 0.x.x   | ⚠️ 중요 취약점만    |
| < 0.1   | ❌ 지원 종료        |

---

## 보안 취약점 신고

### 신고 방법

보안 취약점을 발견하셨다면, **공개 이슈로 등록하지 마시고** 다음 방법으로 비공개 신고해 주세요:

1. **이메일**: hephaex@gmail.com
   - 제목: `[SECURITY] mecab-ko 취약점 신고`

2. **GitHub Security Advisory**:
   - https://github.com/hephaex/mecab-ko/security/advisories/new

### 신고 내용

보고서에 다음 정보를 포함해 주세요:

```
1. 취약점 유형
   - 예: Buffer overflow, Memory leak, Denial of Service 등

2. 영향 범위
   - 영향받는 컴포넌트/모듈
   - 영향받는 버전

3. 재현 방법
   - 단계별 재현 절차
   - 테스트 코드 (가능한 경우)

4. 예상 영향
   - 심각도 평가 (Critical/High/Medium/Low)
   - 잠재적 공격 시나리오

5. 제안된 해결책 (선택)
   - 수정 방향 제안
   - 패치 (가능한 경우)
```

### 응답 시간

| 단계 | 목표 시간 |
|------|-----------|
| 신고 확인 | 24시간 이내 |
| 초기 평가 | 72시간 이내 |
| 수정 계획 공유 | 7일 이내 |
| 패치 릴리스 | 30일 이내 (심각도에 따라) |

### 공개 정책

- 패치 릴리스 후 CVE 발급 (해당 시)
- 보안 권고 공개
- 신고자 크레딧 (원하는 경우)

---

## 보안 개발 원칙

### Rust 안전성 활용

MeCab-Ko는 Rust의 메모리 안전성을 최대한 활용합니다:

```rust
// ✅ Safe Rust 우선
pub fn tokenize(input: &str) -> Vec<Token> {
    // 소유권과 빌림 검사를 통한 안전한 메모리 관리
}

// ⚠️ unsafe는 최소화하고 명시적으로 문서화
/// # Safety
/// `ptr`은 반드시 유효한 UTF-8 데이터를 가리켜야 함
pub unsafe fn from_raw_ptr(ptr: *const u8, len: usize) -> &str {
    // SAFETY: 호출자가 ptr의 유효성을 보장
    std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len))
}
```

### 입력 검증

모든 외부 입력은 검증됩니다:

```rust
pub fn load_dictionary(path: &Path) -> Result<Dictionary, DictError> {
    // 경로 검증
    if !path.exists() {
        return Err(DictError::NotFound(path.to_path_buf()));
    }
    
    // 파일 형식 검증
    let header = read_header(path)?;
    if header.magic != DICT_MAGIC {
        return Err(DictError::InvalidFormat);
    }
    
    // 버전 검증
    if header.version > SUPPORTED_VERSION {
        return Err(DictError::UnsupportedVersion(header.version));
    }
    
    // ... 안전한 로딩
}
```

### 의존성 관리

```toml
# Cargo.toml에서 의존성 버전 고정
[dependencies]
serde = "=1.0.193"
unicode-normalization = "=0.1.22"

# 보안 취약점 스캔
[dev-dependencies]
cargo-audit = "0.18"
```

---

## 보안 검사 항목

### 자동화된 검사

CI/CD 파이프라인에서 자동 실행:

```yaml
# 보안 스캔 워크플로우
security-audit:
  - cargo audit                    # 의존성 취약점
  - cargo deny check               # 라이선스 및 보안
  - cargo clippy                   # 정적 분석
  - cargo miri test                # 메모리 안전성 (선택)
```

### 수동 검토

코드 리뷰 시 확인 사항:

| 검사 항목 | 설명 |
|-----------|------|
| unsafe 사용 | 필요성 및 안전성 근거 확인 |
| 외부 입력 | 검증 및 새니타이징 |
| 파일 I/O | 경로 순회 공격 방지 |
| 정수 연산 | 오버플로우 처리 |
| 에러 처리 | 정보 누출 방지 |
| 의존성 | 신뢰할 수 있는 소스 |

---

## 위협 모델

### 고려 위협

| 위협 | 대응 |
|------|------|
| 악의적 사전 파일 | 포맷 검증, 크기 제한 |
| DoS (긴 입력) | 입력 길이 제한, 타임아웃 |
| 메모리 고갈 | 할당 제한, 스트리밍 처리 |
| 경로 순회 | 경로 정규화, 샌드박싱 |

### 범위 외

다음은 현재 위협 모델에서 제외:

- 물리적 접근
- 운영체제 취약점
- 네트워크 공격 (라이브러리는 네트워크 미사용)

---

## 보안 릴리스

### 핫픽스 프로세스

중요 보안 취약점 발견 시:

1. **분석** (1-2일)
   - 영향 범위 확인
   - 심각도 평가

2. **수정** (1-7일)
   - 패치 개발
   - 내부 검증

3. **릴리스** (1일)
   - 패치 버전 릴리스
   - 보안 권고 발행

4. **공개** (패치 후 7-14일)
   - CVE 발급
   - 상세 내용 공개

### 버전 명명

보안 패치 버전: `x.y.z` → `x.y.z+1`

예: `1.2.3` → `1.2.4` (보안 패치)

---

## 의존성 보안

### 정기 감사

```bash
# 주간 자동 실행
cargo audit

# 업데이트가 필요한 의존성 확인
cargo outdated
```

### 의존성 정책

| 기준 | 정책 |
|------|------|
| 알려진 취약점 | 즉시 업데이트 또는 제거 |
| 유지보수 중단 | 대안 검토, 3개월 내 교체 |
| 라이선스 | MIT, Apache-2.0, BSD만 허용 |

---

## 보안 기능 (선택적)

### 퍼징 테스트

```bash
# cargo-fuzz로 퍼징 테스트
cargo +nightly fuzz run tokenize_fuzz
```

### WASM 샌드박싱

WASM 빌드는 자동으로 샌드박스 환경에서 실행:

```rust
// WASM에서는 파일 시스템 접근 불가
#[cfg(target_arch = "wasm32")]
pub fn load_dictionary() -> Dictionary {
    // 임베디드 사전만 사용
    Dictionary::embedded()
}
```

---

## 연락처

- **보안 이슈**: hephaex@gmail.com
- **일반 문의**: GitHub Issues

---

*Last Updated: 2026-01-04*  
*Maintainer: hephaex (hephaex@gmail.com)*
