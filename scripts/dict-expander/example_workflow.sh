#!/bin/bash
# MeCab-Ko Dictionary Expander - Example Workflow
#
# This script demonstrates a complete workflow for expanding
# a MeCab dictionary using all available tools.

set -e  # Exit on error

echo "======================================================"
echo "MeCab-Ko Dictionary Expander - Example Workflow"
echo "======================================================"
echo

# Configuration
OUTPUT_DIR="./output"
MECAB_DICT_BASE="${MECAB_DICT_BASE:-/path/to/mecab-ko-dic/seed}"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Step 1: Expand Proper Nouns
echo "Step 1: Expanding proper nouns..."
echo "------------------------------------------------------"

# Example 1: Extract person names from Wikipedia (sample)
echo "  1.1: Fetching person names from Wikipedia..."
python expand_proper_nouns.py \
    --source wikipedia \
    --category "대한민국의_배우" \
    --type person \
    --limit 50 \
    -o "$OUTPUT_DIR/person_actors.csv" 2>&1 | head -20

# Example 2: Extract place names from public data
echo "  1.2: Fetching place names..."
python expand_proper_nouns.py \
    --source public_data \
    --type place \
    --limit 20 \
    -o "$OUTPUT_DIR/places.csv"

# Example 3: Extract organization names
echo "  1.3: Fetching organization names..."
python expand_proper_nouns.py \
    --source public_data \
    --type organization \
    --limit 20 \
    -o "$OUTPUT_DIR/organizations.csv"

echo "  ✓ Proper nouns expanded"
echo

# Step 2: Generate Compound Nouns
echo "Step 2: Generating compound nouns..."
echo "------------------------------------------------------"

# Create sample base nouns file for demonstration
cat > "$OUTPUT_DIR/base_nouns.txt" <<EOF
컴퓨터
과학
인공
지능
자연
언어
처리
기계
학습
데이터
EOF

echo "  2.1: Generating compounds with suffixes..."
python expand_compounds.py \
    --input "$OUTPUT_DIR/base_nouns.txt" \
    --suffixes \
    -o "$OUTPUT_DIR/compound_suffixes.csv"

# Create sample pattern file
cat > "$OUTPUT_DIR/compound_patterns.txt" <<EOF
# Common IT compound nouns
컴퓨터 과학
인공 지능
자연 언어 처리
기계 학습
데이터 과학
EOF

echo "  2.2: Generating from patterns..."
if [ -f "$MECAB_DICT_BASE/NNG.csv" ]; then
    python expand_compounds.py \
        --dict "$MECAB_DICT_BASE" \
        --patterns "$OUTPUT_DIR/compound_patterns.txt" \
        -o "$OUTPUT_DIR/pattern_compounds.csv"
else
    echo "  (Skipping: MECAB_DICT_BASE not found)"
fi

echo "  ✓ Compound nouns generated"
echo

# Step 3: Generate Conjugations
echo "Step 3: Generating verb conjugations..."
echo "------------------------------------------------------"

# Create sample verbs file
cat > "$OUTPUT_DIR/sample_verbs.txt" <<EOF
하다
가다
오다
보다
먹다
읽다
쓰다
듣다
EOF

echo "  3.1: Generating common conjugations..."
python expand_conjugations.py \
    --input "$OUTPUT_DIR/sample_verbs.txt" \
    --patterns common \
    -o "$OUTPUT_DIR/conjugations.csv"

echo "  ✓ Conjugations generated"
echo

# Step 4: Generate Abbreviations
echo "Step 4: Generating abbreviations..."
echo "------------------------------------------------------"

# Create sample abbreviation mapping
cat > "$OUTPUT_DIR/abbrev_map.txt" <<EOF
# Common Korean abbreviations
KBS=한국방송공사
MBC=문화방송
SBS=서울방송
AI=인공지능
NLP=자연어처리
ML=기계학습
EOF

echo "  4.1: Generating from abbreviation map..."
python expand_abbreviations.py \
    --map "$OUTPUT_DIR/abbrev_map.txt" \
    -o "$OUTPUT_DIR/abbreviations_map.csv"

# If we have compounds, extract abbreviations from them
if [ -f "$OUTPUT_DIR/pattern_compounds.csv" ]; then
    echo "  4.2: Extracting abbreviations from compounds..."
    python expand_abbreviations.py \
        --dict "$OUTPUT_DIR/pattern_compounds.csv" \
        --extract-initials \
        -o "$OUTPUT_DIR/abbreviations_extracted.csv"
fi

echo "  ✓ Abbreviations generated"
echo

# Step 5: Merge All Outputs
echo "Step 5: Merging all outputs..."
echo "------------------------------------------------------"

FINAL_OUTPUT="$OUTPUT_DIR/expanded_dictionary.csv"

# Add header comment
cat > "$FINAL_OUTPUT" <<EOF
# MeCab-Ko Expanded Dictionary
# Generated: $(date)
#
# This file contains automatically generated dictionary entries from:
# - Proper nouns (person names, places, organizations)
# - Compound nouns
# - Verb conjugations
# - Abbreviations
#
# Format: surface,left_id,right_id,cost,pos,semantic,has_jongseong,reading,type,first_pos,last_pos,expression
EOF

# Merge all CSV files
for file in "$OUTPUT_DIR"/*.csv; do
    if [ -f "$file" ] && [ "$file" != "$FINAL_OUTPUT" ]; then
        cat "$file" >> "$FINAL_OUTPUT"
    fi
done

echo "  ✓ Merged to $FINAL_OUTPUT"
echo

# Step 6: Generate Statistics
echo "Step 6: Statistics..."
echo "------------------------------------------------------"

total_lines=$(grep -v '^#' "$FINAL_OUTPUT" | wc -l)
echo "  Total entries: $total_lines"

# Count by type
echo "  Entries by semantic category:"
for category in "인명" "지명" "기관" "약어"; do
    count=$(grep -c ",$category," "$FINAL_OUTPUT" 2>/dev/null || echo "0")
    echo "    - $category: $count"
done

echo "  Entries by POS:"
for pos in "NNP" "NNG" "VV" "VA"; do
    count=$(grep -c ",$pos," "$FINAL_OUTPUT" 2>/dev/null || echo "0")
    echo "    - $pos: $count"
done

echo

# Step 7: Sample Output
echo "Step 7: Sample entries..."
echo "------------------------------------------------------"
echo "  First 10 entries from expanded dictionary:"
grep -v '^#' "$FINAL_OUTPUT" | head -10
echo

# Done
echo "======================================================"
echo "✓ Dictionary expansion complete!"
echo "======================================================"
echo
echo "Output files:"
echo "  - Proper nouns: $OUTPUT_DIR/person_actors.csv, places.csv, organizations.csv"
echo "  - Compounds: $OUTPUT_DIR/compound_suffixes.csv, pattern_compounds.csv"
echo "  - Conjugations: $OUTPUT_DIR/conjugations.csv"
echo "  - Abbreviations: $OUTPUT_DIR/abbreviations_*.csv"
echo "  - Final merged: $FINAL_OUTPUT"
echo
echo "Next steps:"
echo "  1. Review the generated entries in $FINAL_OUTPUT"
echo "  2. Merge with existing dictionary using:"
echo "     cd ../../scripts"
echo "     python merge_dictionaries.py \\"
echo "       --base $MECAB_DICT_BASE \\"
echo "       --additional ../tools/dict-expander/$FINAL_OUTPUT \\"
echo "       --output /path/to/output/merged_dict"
echo
