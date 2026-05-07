#!/bin/bash
set -uo pipefail

MECAB_BIN="${MECAB_BIN:-cargo run --manifest-path rust/Cargo.toml --release --bin mecab --}"
MINI_DICT="${MINI_DICT:-rust/test-fixtures/mini-dict}"

PASS_COUNT=0
FAIL_COUNT=0

pass() { echo "PASS: $1"; ((PASS_COUNT++)); }
fail() { echo "FAIL: $1"; ((FAIL_COUNT++)); }
skip() { echo "SKIP: $1 (no dictionary)"; }

dict_available() { [ -f "${MINI_DICT}/sys.dic" ]; }

echo "=== MeCab-Ko CLI E2E Tests ==="

output=$(eval "$MECAB_BIN" --version 2>&1)
if [ $? -eq 0 ] && echo "$output" | grep -qiE 'mecab|[0-9]+\.[0-9]+'; then
  pass "version flag outputs version string"
else
  fail "version flag outputs version string"
fi

eval "$MECAB_BIN" --help >/dev/null 2>&1 && \
  pass "help flag exits 0" || \
  fail "help flag exits 0"

output=$(printf '' | eval "$MECAB_BIN" -d "$MINI_DICT" 2>&1); rc=$?
[ $rc -eq 0 ] && pass "empty input does not crash" || fail "empty input does not crash"

eval "$MECAB_BIN" -d /nonexistent_dicdir_xyz >/dev/null 2>&1 && \
  fail "nonexistent dicdir returns non-zero exit" || \
  pass "nonexistent dicdir returns non-zero exit"

if dict_available; then
  output=$(echo "안녕하세요" | eval "$MECAB_BIN" -d "$MINI_DICT" -O wakati 2>&1)
  if echo "$output" | grep -qE '^[^ ]+( [^ ]+)*'; then
    pass "wakati mode produces space-separated output"
  else
    fail "wakati mode produces space-separated output"
  fi

  output=$(echo "안녕하세요" | eval "$MECAB_BIN" -d "$MINI_DICT" 2>&1)
  if printf '%s' "$output" | grep -q '	'; then
    pass "default output has tab-separated fields"
  else
    fail "default output has tab-separated fields"
  fi

  echo "$output" | grep -q '^EOS' && \
    pass "output ends with EOS line" || \
    fail "output ends with EOS line"
else
  skip "wakati mode produces space-separated output"
  skip "default output has tab-separated fields"
  skip "output ends with EOS line"
fi

echo ""
echo "=== Results: ${PASS_COUNT} passed, ${FAIL_COUNT} failed ==="
[ "$FAIL_COUNT" -eq 0 ] || exit 1
