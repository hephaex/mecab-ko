# Security Vulnerability Fix Summary

## Date: 2026-01-27

## Critical Vulnerabilities Resolved

### 1. SQLAlchemy SQL Injection Vulnerabilities (CRITICAL)

**Location:** `/mecab-ko-dic/utils/requirements.txt`

**Vulnerabilities Fixed:**
- **CVE-2019-7164** (CVSS 9.8 - Critical)
  - SQL Injection via order_by parameter
  - Affected versions: < 1.2.18
  
- **CVE-2019-7548** (CVSS 7.8 - High)
  - SQL Injection via group_by parameter
  - Affected versions: < 1.2.19

**Fix Applied:**
```diff
- sqlalchemy==0.9.8
+ sqlalchemy>=2.0.0
```

**Impact:** Upgraded from severely outdated SQLAlchemy 0.9.8 (released 2014) to 2.0.0+, which includes all security patches and modern best practices.

---

## Rust Dependency Updates (Unmaintained Crates)

### 2. indicatif - Progress Bar Library

**Status:** Updated from 0.17 to 0.18.3
**Result:** Removed unmaintained dependency `number_prefix`

**Changes:**
- `rust/Cargo.toml`: `indicatif = "0.18"`
- Removed: `number_prefix 0.4.0` (RUSTSEC-2025-0119)

### 3. tabled - Table Formatting Library

**Status:** Updated from 0.15 to 0.20.0
**Result:** Removed unmaintained dependency `proc-macro-error`

**Changes:**
- `rust/crates/mecab-ko-profiler/Cargo.toml`: `tabled = "0.20"`
- Removed: `proc-macro-error 1.0.4` (RUSTSEC-2024-0370)
- Added: `proc-macro-error2 2.0.1` (maintained fork)

### 4. bincode - Binary Serialization

**Status:** Acknowledged but retained
**Advisory:** RUSTSEC-2025-0141 (unmaintained)

**Rationale:**
- bincode 1.3 is stable and functional
- bincode 3.0 exists but is minimally maintained (archived Aug 2025)
- Migration to alternatives (postcard, ciborium, rmp-serde) planned for Phase 2
- Updated documentation with migration plan

---

## Verification

### Cargo Audit Results

**Before:**
- 3 warnings (bincode, number_prefix, proc-macro-error)

**After:**
- 1 warning (bincode - documented with migration plan)

### Build Status

```bash
$ cargo build --workspace
   Finished `dev` profile [unoptimized + debuginfo] target(s)

$ cargo build --package mecab-ko-profiler --package mecab-ko-dict-builder
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.90s
```

All packages using updated dependencies compile successfully.

---

## GitHub Dependabot Alerts

The two critical Dependabot alerts (#3 and #4) for SQLAlchemy vulnerabilities should now be automatically resolved once this change is pushed to GitHub.

**Expected Results:**
- Alert #3 (CVE-2019-7164): Will be closed automatically
- Alert #4 (CVE-2019-7548): Will be closed automatically

---

## Files Modified

1. `/mecab-ko-dic/utils/requirements.txt` - SQLAlchemy update
2. `/rust/Cargo.toml` - indicatif update and bincode documentation
3. `/rust/crates/mecab-ko-profiler/Cargo.toml` - tabled update
4. `/rust/Cargo.lock` - Dependency resolution

---

## Next Steps

1. Commit these changes
2. Push to GitHub to resolve Dependabot alerts
3. Monitor for automatic closure of alerts #3 and #4
4. Plan bincode migration for Phase 2 development

---

## References

- [SQLAlchemy Security Advisory GHSA-38fc-9xqv-7f7q](https://github.com/advisories/GHSA-38fc-9xqv-7f7q)
- [SQLAlchemy Security Advisory GHSA-887w-45rq-vxgf](https://github.com/advisories/GHSA-887w-45rq-vxgf)
- [Bincode Status Discussion](https://users.rust-lang.org/t/whats-going-on-with-bincode/136942)
- [RustSec Advisory Database](https://rustsec.org/)
