#!/bin/bash
# Run tests with full dictionary
# Usage: ./scripts/test-full-dict.sh [path-to-dict]
#
# When no path is given, defaults to <repo-root>/data/dict-output.
# The directory must contain a pre-built MeCab-Ko dictionary
# (sys.dic, matrix.bin, unk.bin, entries.bin, …).
#
# Build the dictionary first if needed:
#   cd rust
#   cargo run -p mecab-ko-dict-builder -- \
#       --csv-dir ../data/mecab-ko-dic-2.1.1-20180720 \
#       --output-dir ../data/dict-output

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

DICT_PATH="${1:-${REPO_ROOT}/data/dict-output}"

if [ ! -d "${DICT_PATH}" ]; then
    echo "Error: Dictionary directory not found at ${DICT_PATH}"
    echo "Build it first:"
    echo "  cd rust"
    echo "  cargo run -p mecab-ko-dict-builder -- \\"
    echo "      --csv-dir ../data/mecab-ko-dic-2.1.1-20180720 \\"
    echo "      --output-dir ../data/dict-output"
    exit 1
fi

echo "Running full-dict tests with: ${DICT_PATH}"
export MECAB_KO_FULL_DICT="${DICT_PATH}"

cd "${REPO_ROOT}/rust"
cargo test --workspace \
    --exclude mecab-ko-python \
    --exclude mecab-ko-node \
    --exclude mecab-ko-wasm \
    -- --include-ignored
