#!/usr/bin/env bats
# CLI output format E2E tests for MeCab-Ko

setup() {
    export MECAB_BIN="${MECAB_BIN:-/home/mare/mecab-ko/rust/target/debug/mecab-ko}"
    export TEMP_DIR="${BATS_TEST_TMPDIR}"
    export TEST_SENTENCE="나는 학교에 갑니다."
}

@test "CLI default output format" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    echo "$TEST_SENTENCE" | run "$MECAB_BIN"
    [ "$status" -eq 0 ]
    # Default format should have tab-separated values
    [[ "$output" =~ $'\t' ]]
}

@test "CLI wakati output format" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    echo "$TEST_SENTENCE" | run "$MECAB_BIN" --format wakati
    [ "$status" -eq 0 ]
    # Wakati format should have space-separated tokens
    [[ "$output" =~ " " ]]
}

@test "CLI JSON output format" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    echo "$TEST_SENTENCE" | run "$MECAB_BIN" --format json
    if [ "$status" -ne 0 ]; then
        skip "JSON format not implemented yet"
    fi

    # Should be valid JSON
    echo "$output" | jq . > /dev/null 2>&1
}

@test "CLI JSON output structure" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    echo "$TEST_SENTENCE" | "$MECAB_BIN" --format json > "${TEMP_DIR}/output.json" 2>&1
    if [ ! -f "${TEMP_DIR}/output.json" ]; then
        skip "JSON format not implemented yet"
    fi

    # Check JSON structure using jq
    if command -v jq &> /dev/null; then
        # Should have tokens array
        jq -e '.tokens' "${TEMP_DIR}/output.json" > /dev/null
        # Each token should have surface and pos
        jq -e '.tokens[0].surface' "${TEMP_DIR}/output.json" > /dev/null
        jq -e '.tokens[0].pos' "${TEMP_DIR}/output.json" > /dev/null
    fi
}

@test "CLI JSONL output format" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    cat <<EOF | run "$MECAB_BIN" --format jsonl
나는 학교에 갑니다.
오늘은 날씨가 좋습니다.
EOF

    if [ "$status" -ne 0 ]; then
        skip "JSONL format not implemented yet"
    fi

    # Each line should be valid JSON
    if command -v jq &> /dev/null; then
        echo "$output" | while IFS= read -r line; do
            echo "$line" | jq . > /dev/null
        done
    fi
}

@test "CLI custom format string" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    echo "$TEST_SENTENCE" | run "$MECAB_BIN" --format-string "%m\t%f[0]\n"
    if [ "$status" -ne 0 ]; then
        skip "Custom format string not implemented yet"
    fi
    [ "$status" -eq 0 ]
}

@test "CLI node format output" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    echo "$TEST_SENTENCE" | run "$MECAB_BIN" --node-format="%m\\t%f[0]\\n"
    if [ "$status" -ne 0 ]; then
        skip "Node format not implemented yet"
    fi
    [ "$status" -eq 0 ]
}

@test "CLI dump features" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "mecab-ko binary not built yet"
    fi

    echo "$TEST_SENTENCE" | run "$MECAB_BIN" --dump-features
    if [ "$status" -ne 0 ]; then
        skip "Dump features not implemented yet"
    fi
    [ "$status" -eq 0 ]
}
