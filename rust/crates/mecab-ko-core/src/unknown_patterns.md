# Unknown Word Pattern Detection

## Overview

The enhanced unknown word handler now recognizes multiple patterns and adjusts costs accordingly to improve tokenization accuracy for modern Korean text with mixed content.

## Supported Patterns

### 1. Plain Pattern
Default pattern for single-category consecutive characters.

**Examples:**
- `hello` (lowercase English)
- `테스트` (Korean only)
- `123` (numbers only)

**Cost Adjustment:**
- Length penalty: +100 per character beyond 5 characters
- Encourages breaking long unknown words

### 2. ProperNoun Pattern
Detects words starting with uppercase letter (potential proper nouns).

**Examples:**
- `Apple`
- `Google`
- `Microsoft`

**Cost Adjustment:**
- Cost reduction: -500
- More likely to be kept as a single token

**POS Tag:**
- Estimated as `NNP` (고유명사) instead of `SL` (외래어)

### 3. CamelCase Pattern
Detects mixed-case words with internal uppercase letters.

**Examples:**
- `iPhone`
- `HelloWorld`
- `YouTube`

**Cost Adjustment:**
- Cost reduction: -300
- Recognizes brand names and technical terms

**POS Tag:**
- Estimated as `NNP` (고유명사)

### 4. HangulAlphaMix Pattern
Detects Korean-English mixed words.

**Examples:**
- `API키`
- `프로그램명`
- `데이터베이스이름`

**Cost Adjustment:**
- Cost increase: +200
- Slightly penalizes mixed patterns

**POS Tag:**
- Korean portion estimated as `NNG` (일반명사)

### 5. NumberUnit Pattern
Detects numbers with units or Korean counters.

**Examples:**
- `15kg`
- `3개`
- `100원`

**Cost Adjustment:**
- Cost reduction: -200
- Natural pattern in Korean text

### 6. Emoji Pattern
Detects Unicode emoji characters.

**Examples:**
- `😀`
- `🚀`
- `❤️`

**Cost Adjustment:**
- Cost increase: +1000
- High penalty discourages emoji in normal text flow

## Cost Calculation Algorithm

```rust
fn adjust_cost_by_pattern(base_cost: i16, pattern: WordPattern, length: usize) -> i16 {
    let mut cost = base_cost as i32;

    // Apply pattern-specific adjustment
    match pattern {
        Plain => {
            if length > 5 {
                cost += (length - 5) * 100;  // Length penalty
            }
        }
        ProperNoun => cost -= 500,
        CamelCase => cost -= 300,
        HangulAlphaMix => cost += 200,
        NumberUnit => cost -= 200,
        Emoji => cost += 1000,
    }

    // Clamp to i16 range
    cost.clamp(i16::MIN, i16::MAX) as i16
}
```

## POS Tag Estimation

The handler estimates more appropriate POS tags based on patterns:

| Pattern | Category | Base POS | Estimated POS |
|---------|----------|----------|---------------|
| ProperNoun | ALPHA | SL | NNP |
| CamelCase | ALPHA | SL | NNP |
| HangulAlphaMix | HANGUL | UNKNOWN | NNG |
| Others | Any | (unchanged) | (unchanged) |

## Integration with Viterbi Algorithm

The enhanced unknown handler generates candidates with adjusted costs that influence the Viterbi path selection:

1. **Pattern Detection**: Each candidate's pattern is detected
2. **Cost Adjustment**: Base cost modified by pattern and length
3. **POS Estimation**: More accurate POS tag assigned
4. **Lattice Integration**: Candidates added to lattice with adjusted costs
5. **Viterbi Selection**: Lower cost paths preferred in final tokenization

## Examples

### Example 1: Proper Noun
```
Input: "Apple은 혁신적이다"
Unknown candidate for "Apple":
  - Base cost: 4000 (SL)
  - Pattern: ProperNoun
  - Adjusted cost: 3500
  - Estimated POS: NNP
  - Result: Kept as single token
```

### Example 2: Long Unknown Word
```
Input: "abcdefghijk"
Unknown candidate:
  - Base cost: 4000
  - Pattern: Plain
  - Length: 11 characters
  - Penalty: +600 (for 6 chars beyond limit)
  - Adjusted cost: 4600
  - Result: Higher cost may encourage breaking
```

### Example 3: CamelCase Brand
```
Input: "iPhone15를 샀다"
Unknown candidate for "iPhone":
  - Base cost: 4000
  - Pattern: CamelCase
  - Adjusted cost: 3700
  - Estimated POS: NNP
  - Result: Recognized as brand name
```

### Example 4: Number with Unit
```
Input: "무게는 15kg이다"
Unknown candidate for "15kg":
  - Base cost: 3000 (SN base) + 4000 (SL for "kg")
  - Pattern: NumberUnit
  - Adjusted cost: Lower overall
  - Result: Natural unit expression
```

## Performance Impact

- **Pattern Detection**: O(n) where n is word length (typically < 20)
- **Cost Adjustment**: O(1) constant time
- **Memory**: No additional allocations for pattern detection
- **Overall Impact**: Negligible (<1% tokenization time)

## Future Enhancements

Potential improvements for future versions:

1. **Context-aware cost adjustment**: Consider surrounding tokens
2. **Statistical pattern learning**: Learn from corpus which patterns are more common
3. **Domain-specific patterns**: Technical terms, medical terms, legal terms
4. **Compound pattern detection**: Korean compound nouns with complex patterns
5. **Unicode normalization**: Handle different emoji representations
6. **Abbreviation detection**: Recognize common abbreviations (API, HTTP, SQL, etc.)

## Testing

Comprehensive test coverage includes:

- Pattern detection accuracy tests
- Cost adjustment verification tests
- POS estimation tests
- Integration tests with lattice/tokenizer
- Edge case tests (empty strings, single chars, very long words)
- Mixed pattern tests

Run tests with:
```bash
cargo test --package mecab-ko-core unknown
```
