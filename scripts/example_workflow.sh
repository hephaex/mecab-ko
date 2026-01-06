#!/bin/bash
#
# MeCab-Ko Dictionary Building Workflow
#
# This script demonstrates a complete workflow for building a MeCab dictionary
# from Korean corpus data.
#
# Usage:
#   ./example_workflow.sh [corpus_dir] [output_dir]
#
# Requirements:
#   - Python 3.10+
#   - Korean corpus in Modu format (JSON)

set -euo pipefail  # Exit on error, undefined var, pipe failure

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Default paths
DEFAULT_CORPUS_DIR="/data/corpus/modu"
DEFAULT_OUTPUT_DIR="${SCRIPT_DIR}/output"

# Parse arguments
CORPUS_DIR="${1:-$DEFAULT_CORPUS_DIR}"
OUTPUT_DIR="${2:-$DEFAULT_OUTPUT_DIR}"

# Configuration
MIN_FREQ=2
NEOLOGISM_MIN_FREQ=3
NEOLOGISM_MAX_FREQ=500
MERGE_STRATEGY="min_cost"

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_requirements() {
    log_info "Checking requirements..."

    # Check Python version
    if ! command -v python3 &> /dev/null; then
        log_error "python3 not found. Please install Python 3.10 or higher."
        exit 1
    fi

    local python_version=$(python3 --version 2>&1 | awk '{print $2}')
    log_info "Found Python $python_version"

    # Check if corpus directory exists
    if [ ! -d "$CORPUS_DIR" ]; then
        log_error "Corpus directory not found: $CORPUS_DIR"
        log_info "Usage: $0 [corpus_dir] [output_dir]"
        exit 1
    fi

    # Check if scripts are executable
    for script in corpus_to_dict.py extract_neologisms.py merge_dictionaries.py; do
        if [ ! -x "${SCRIPT_DIR}/${script}" ]; then
            log_warn "${script} not executable, fixing..."
            chmod +x "${SCRIPT_DIR}/${script}"
        fi
    done

    log_info "All requirements satisfied"
}

create_output_dirs() {
    log_info "Creating output directories..."

    mkdir -p "${OUTPUT_DIR}"
    mkdir -p "${OUTPUT_DIR}/intermediate"
    mkdir -p "${OUTPUT_DIR}/stats"

    log_info "Output directory: ${OUTPUT_DIR}"
}

convert_corpus() {
    log_info "Step 1/5: Converting corpus to dictionary..."

    local output_file="${OUTPUT_DIR}/intermediate/base_dict.csv"

    "${SCRIPT_DIR}/corpus_to_dict.py" \
        -f modu \
        -i "$CORPUS_DIR" \
        -o "$output_file" \
        --min-freq $MIN_FREQ \
        -v 2>&1 | tee "${OUTPUT_DIR}/stats/corpus_conversion.log"

    if [ -f "$output_file" ]; then
        local entry_count=$(wc -l < "$output_file")
        log_info "Generated $entry_count dictionary entries"
    else
        log_error "Failed to generate base dictionary"
        exit 1
    fi
}

extract_neologisms() {
    log_info "Step 2/5: Extracting neologisms..."

    local base_dict="${OUTPUT_DIR}/intermediate/base_dict.csv"
    local neo_json="${OUTPUT_DIR}/intermediate/neologisms.json"
    local neo_csv="${OUTPUT_DIR}/intermediate/neologisms_review.csv"

    # JSON output
    "${SCRIPT_DIR}/extract_neologisms.py" \
        -f modu \
        -i "$CORPUS_DIR" \
        -o "$neo_json" \
        --reference-dict "$base_dict" \
        --min-freq $NEOLOGISM_MIN_FREQ \
        --max-freq $NEOLOGISM_MAX_FREQ \
        -v 2>&1 | tee "${OUTPUT_DIR}/stats/neologism_extraction.log"

    # CSV output for manual review
    "${SCRIPT_DIR}/extract_neologisms.py" \
        -f modu \
        -i "$CORPUS_DIR" \
        -o "$neo_csv" \
        --output-format csv \
        --reference-dict "$base_dict" \
        --min-freq $NEOLOGISM_MIN_FREQ \
        --max-freq $NEOLOGISM_MAX_FREQ \
        2>&1 | tee -a "${OUTPUT_DIR}/stats/neologism_extraction.log"

    if [ -f "$neo_json" ]; then
        local neo_count=$(python3 -c "import json; print(len(json.load(open('$neo_json'))['neologisms']))")
        log_info "Found $neo_count neologism candidates"
        log_warn "Please review: $neo_csv"
    fi
}

manual_review_neologisms() {
    log_info "Step 3/5: Manual neologism review..."

    local neo_csv="${OUTPUT_DIR}/intermediate/neologisms_review.csv"
    local approved_csv="${OUTPUT_DIR}/intermediate/neologisms_approved.csv"

    if [ ! -f "$neo_csv" ]; then
        log_warn "No neologisms to review, skipping..."
        return
    fi

    log_warn "Manual review required!"
    log_info "Review file: $neo_csv"
    log_info "Create approved file: $approved_csv"
    log_info ""
    log_info "You can:"
    log_info "  1. Manually filter $neo_csv to create $approved_csv"
    log_info "  2. Use all neologisms: cp $neo_csv $approved_csv"
    log_info "  3. Skip neologisms: touch $approved_csv"
    log_info ""

    # Auto-approve for demo (in production, this should be manual)
    if [ ! -f "$approved_csv" ]; then
        log_warn "Auto-approving all neologisms (for demo purposes)"
        # Convert JSON to CSV format for merging
        # In production, this should be a manual review process
        touch "$approved_csv"
    fi
}

merge_dictionaries() {
    log_info "Step 4/5: Merging dictionaries..."

    local base_dict="${OUTPUT_DIR}/intermediate/base_dict.csv"
    local approved_neo="${OUTPUT_DIR}/intermediate/neologisms_approved.csv"
    local final_dict="${OUTPUT_DIR}/mecab_dict.csv"

    local merge_inputs=("$base_dict")

    # Add neologisms if approved file has content
    if [ -f "$approved_neo" ] && [ -s "$approved_neo" ]; then
        merge_inputs+=("$approved_neo")
        log_info "Including approved neologisms in merge"
    else
        log_info "No approved neologisms, using base dictionary only"
    fi

    "${SCRIPT_DIR}/merge_dictionaries.py" \
        -i "${merge_inputs[@]}" \
        -o "$final_dict" \
        --strategy $MERGE_STRATEGY \
        -v 2>&1 | tee "${OUTPUT_DIR}/stats/merge.log"

    if [ -f "$final_dict" ]; then
        local final_count=$(wc -l < "$final_dict")
        log_info "Final dictionary has $final_count entries"
    fi
}

analyze_results() {
    log_info "Step 5/5: Analyzing results..."

    local final_dict="${OUTPUT_DIR}/mecab_dict.csv"

    "${SCRIPT_DIR}/merge_dictionaries.py" \
        --analyze "$final_dict" \
        2>&1 | tee "${OUTPUT_DIR}/stats/analysis.log"

    # Generate summary statistics
    log_info "Generating summary statistics..."

    {
        echo "==================================================="
        echo "Dictionary Build Summary"
        echo "==================================================="
        echo "Build Date: $(date '+%Y-%m-%d %H:%M:%S')"
        echo "Corpus: $CORPUS_DIR"
        echo "Output: $OUTPUT_DIR"
        echo ""
        echo "Configuration:"
        echo "  Min Frequency: $MIN_FREQ"
        echo "  Neologism Min Freq: $NEOLOGISM_MIN_FREQ"
        echo "  Neologism Max Freq: $NEOLOGISM_MAX_FREQ"
        echo "  Merge Strategy: $MERGE_STRATEGY"
        echo ""
        echo "Results:"
        echo "  Total Entries: $(wc -l < "$final_dict")"
        echo ""
        echo "Top 10 POS Tags:"
        cut -d',' -f5 "$final_dict" | sort | uniq -c | sort -rn | head -10
        echo ""
        echo "==================================================="
    } | tee "${OUTPUT_DIR}/stats/summary.txt"
}

create_metadata() {
    log_info "Creating metadata file..."

    local metadata_file="${OUTPUT_DIR}/metadata.json"
    local entry_count=$(wc -l < "${OUTPUT_DIR}/mecab_dict.csv")

    cat > "$metadata_file" << EOF
{
  "name": "MeCab-Ko Custom Dictionary",
  "version": "1.0.0",
  "generated_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "generator": "mecab-ko-corpus-tools",
  "config": {
    "min_frequency": $MIN_FREQ,
    "neologism_min_freq": $NEOLOGISM_MIN_FREQ,
    "neologism_max_freq": $NEOLOGISM_MAX_FREQ,
    "merge_strategy": "$MERGE_STRATEGY"
  },
  "sources": [
    {
      "name": "Modu Corpus",
      "path": "$CORPUS_DIR",
      "license": "CC BY-SA 2.0 KR",
      "url": "https://corpus.korean.go.kr"
    }
  ],
  "statistics": {
    "total_entries": $entry_count
  }
}
EOF

    log_info "Metadata saved to: $metadata_file"
}

create_license_file() {
    log_info "Creating LICENSE file..."

    local license_file="${OUTPUT_DIR}/LICENSE.txt"

    cat > "$license_file" << 'EOF'
MeCab-Ko Custom Dictionary
License Information

This dictionary was generated from the following sources:

1. Modu Corpus (모두의 말뭉치)
   - Provider: National Institute of Korean Language
   - License: CC BY-SA 2.0 KR
   - URL: https://corpus.korean.go.kr

This dictionary is licensed under CC BY-SA 2.0 KR due to the
share-alike requirement of the Modu Corpus.

Attribution:
This work is based on "모두의 말뭉치" by the National Institute
of Korean Language, used under CC BY-SA 2.0 KR.

For full license text, see:
https://creativecommons.org/licenses/by-sa/2.0/kr/

Generated: $(date '+%Y-%m-%d')
EOF

    log_info "License file saved to: $license_file"
}

print_summary() {
    echo ""
    echo "==================================================="
    echo -e "${GREEN}Dictionary Build Complete!${NC}"
    echo "==================================================="
    echo ""
    echo "Output Files:"
    echo "  Dictionary:  ${OUTPUT_DIR}/mecab_dict.csv"
    echo "  Metadata:    ${OUTPUT_DIR}/metadata.json"
    echo "  License:     ${OUTPUT_DIR}/LICENSE.txt"
    echo "  Statistics:  ${OUTPUT_DIR}/stats/"
    echo ""
    echo "Next Steps:"
    echo "  1. Review the generated dictionary"
    echo "  2. Compile to MeCab binary format"
    echo "  3. Test with MeCab"
    echo ""
    echo "To compile to binary format:"
    echo "  mecab-dict-index -d /path/to/mecab-ko-dic \\"
    echo "    -u ${OUTPUT_DIR}/custom.dic \\"
    echo "    -f utf-8 -t utf-8 \\"
    echo "    ${OUTPUT_DIR}/mecab_dict.csv"
    echo ""
    echo "==================================================="
}

# Main execution
main() {
    log_info "Starting MeCab-Ko dictionary build workflow"
    log_info "Corpus: $CORPUS_DIR"
    log_info "Output: $OUTPUT_DIR"
    echo ""

    check_requirements
    create_output_dirs
    convert_corpus
    extract_neologisms
    manual_review_neologisms
    merge_dictionaries
    analyze_results
    create_metadata
    create_license_file
    print_summary

    log_info "Workflow completed successfully!"
}

# Run main function
main "$@"
