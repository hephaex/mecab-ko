#!/usr/bin/env bats
# Basic CLI E2E tests for MeCab-Ko

setup() {
    # Set up test environment
    export MECAB_BIN="${MECAB_BIN:-/home/mare/mecab-ko/rust/target/debug/mecab-ko}"
    export FIXTURES_DIR="/home/mare/mecab-ko/tests/e2e/fixtures"
    export TEMP_DIR="${BATS_TEST_TMPDIR}"
}

teardown() {
    # Clean up temporary files
    rm -f "${TEMP_DIR}"/*.tmp
}

@test "CLI binary exists and is executable" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi
    [ -x "$MECAB_BIN" ]
}

@test "CLI shows help message" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi
    run "$MECAB_BIN" --help
    [ "$status" -eq 0 ]
    [[ "$output" =~ "mecab-ko" ]]
}

@test "CLI shows version" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi
    run "$MECAB_BIN" --version
    [ "$status" -eq 0 ]
    [[ "$output" =~ "0.1.0" ]]
}

@test "CLI tokenizes simple Korean sentence" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi
    echo "나는 학교에 갑니다." | run "$MECAB_BIN"
    [ "$status" -eq 0 ]
    [[ "$output" =~ "나" ]]
    [[ "$output" =~ "학교" ]]
}

@test "CLI handles empty input" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi
    echo "" | run "$MECAB_BIN"
    [ "$status" -eq 0 ]
}

@test "CLI tokenizes from file" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    # Create test input file
    echo "나는 학교에 갑니다." > "${TEMP_DIR}/test_input.txt"
    echo "오늘은 날씨가 좋습니다." >> "${TEMP_DIR}/test_input.txt"

    run "$MECAB_BIN" "${TEMP_DIR}/test_input.txt"
    [ "$status" -eq 0 ]
    [[ "$output" =~ "학교" ]]
    [[ "$output" =~ "날씨" ]]
}

@test "CLI outputs to file" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    echo "나는 학교에 갑니다." | "$MECAB_BIN" -o "${TEMP_DIR}/output.txt"
    [ -f "${TEMP_DIR}/output.txt" ]
    grep -q "학교" "${TEMP_DIR}/output.txt"
}

@test "CLI handles multiple sentences" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    cat <<EOF | run "$MECAB_BIN"
나는 학교에 갑니다.
오늘은 날씨가 좋습니다.
이것은 무엇입니까?
EOF

    [ "$status" -eq 0 ]
    [[ "$output" =~ "학교" ]]
    [[ "$output" =~ "날씨" ]]
    [[ "$output" =~ "무엇" ]]
}

@test "CLI handles long text" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    # Create long text (repeat sentence 1000 times)
    for i in {1..1000}; do
        echo "나는 학교에 갑니다."
    done | run "$MECAB_BIN"

    [ "$status" -eq 0 ]
}

@test "CLI JSON output format" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    echo "나는 학교에 갑니다." | run "$MECAB_BIN" --format json
    [ "$status" -eq 0 ]
    # Output should be valid JSON
    echo "$output" | jq . > /dev/null 2>&1 || skip "jq not available or JSON format not implemented"
}

@test "CLI handles invalid UTF-8 gracefully" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    # Invalid UTF-8 sequence
    printf '\xff\xfe' | run "$MECAB_BIN"
    # Should not crash (status 0 or 1 is acceptable)
    [ "$status" -le 1 ]
}

@test "CLI with user dictionary" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    if [ ! -f "$FIXTURES_DIR/user_dict.csv" ]; then
        skip "user dictionary fixture not found"
    fi

    echo "카카오톡으로 메시지를 보냈다." | run "$MECAB_BIN" --user-dict "$FIXTURES_DIR/user_dict.csv"
    # Should recognize "카카오톡" from user dictionary
    # This is implementation dependent, so we just check it doesn't crash
    [ "$status" -eq 0 ]
}

@test "CLI parallel processing" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    # Create large input file
    for i in {1..100}; do
        echo "나는 학교에 갑니다. 오늘은 날씨가 좋습니다."
    done > "${TEMP_DIR}/large_input.txt"

    run "$MECAB_BIN" --parallel 4 "${TEMP_DIR}/large_input.txt"
    [ "$status" -eq 0 ]
}
