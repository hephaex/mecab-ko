# Integration Testing Guide

## Overview

This document describes the comprehensive integration test suite for the MeCab-Ko Elasticsearch plugin. The test suite ensures reliability, correctness, and performance of the plugin in real Elasticsearch cluster environments.

## Test Architecture

### Test Framework

- **Framework**: JUnit 4 with Elasticsearch Test Framework
- **Base Class**: `ESIntegTestCase` for cluster-based integration tests
- **Build Tool**: Gradle with custom integration test source set

### Test Organization

```
src/integTest/
├── java/com/mecab/ko/elasticsearch/
│   ├── MecabKoAnalyzerIT.java      # Analyzer tests
│   ├── MecabKoTokenizerIT.java     # Tokenizer tests
│   ├── MecabKoFilterIT.java        # Filter tests
│   └── MecabKoIndexIT.java         # Indexing/search tests
└── resources/test-data/
    ├── korean_samples.json         # Korean test documents
    └── mixed_samples.json          # Mixed language test documents
```

## Running Tests

### Quick Start

```bash
# All integration tests
./gradlew integrationTest

# Specific test class
./gradlew integrationTest --tests MecabKoAnalyzerIT

# Specific test method
./gradlew integrationTest --tests MecabKoAnalyzerIT.testBasicAnalysis

# Parallel execution
./gradlew integrationTest --parallel --max-workers=4
```

### Test Variants

```bash
# Quick tests (skip slow tests)
./gradlew quickTest

# With coverage
./gradlew integrationTest jacocoTestReport

# Verbose output
./gradlew integrationTest --info

# Debug mode
./gradlew integrationTest --debug-jvm
```

### Environment Configuration

```bash
# Set Elasticsearch JVM options
export ES_JAVA_OPTS="-Xms1g -Xmx1g"

# Disable security manager (for debugging)
export ES_TESTS_SECURITY_MANAGER=false

# Enable assertions
export GRADLE_OPTS="-ea"
```

## Test Categories

### 1. MecabKoAnalyzerIT

Tests the complete analyzer configuration and behavior.

#### Test Cases

**Basic Functionality**
- `testMecabKoAnalyzerBasic()` - Basic analyzer setup and usage
- `testMecabKoTokenizer()` - Tokenizer configuration
- `testMecabKoPartOfSpeechFilter()` - POS filtering

**Decompound Modes**
- `testDecompoundModeNone()` - Keep compound nouns intact
- `testDecompoundModeDiscard()` - Output only decomposed morphemes
- `testDecompoundModeMixed()` - Output both forms
- `testMultipleAnalyzersInSameIndex()` - Multiple modes in one index

**Advanced Features**
- `testCustomStopTags()` - Custom POS tag filtering
- `testAnalyzerWithUserDictionary()` - User dictionary integration
- `testLargeDocumentAnalysis()` - Large document handling (10KB+)
- `testSpecialCharactersHandling()` - Special characters and symbols

**Performance & Concurrency**
- `testPerformanceBenchmark()` - Throughput and latency testing
- `testConcurrentAnalysis()` - Concurrent request handling

**Edge Cases**
- `testEmptyAndWhitespaceInput()` - Empty/whitespace handling
- `testNoriCompatibility()` - Nori alias compatibility

### 2. MecabKoTokenizerIT

Tests tokenizer-specific functionality.

#### Test Cases

**Basic Tokenization**
- `testBasicTokenization()` - Basic tokenization
- `testDecompoundModeNone/Discard/Mixed()` - Mode-specific behavior

**Input Handling**
- `testMixedKoreanEnglishText()` - Mixed language text
- `testSpecialCharacters()` - Special characters, URLs, emails
- `testEmptyAndWhitespace()` - Empty input handling
- `testLongText()` - Long text processing

**Token Attributes**
- `testOffsetAccuracy()` - Character offset correctness
- `testPositionIncrements()` - Position increment accuracy
- `testOutputUnknownUnigrams()` - Unknown word handling

**Concurrency**
- `testConcurrentAnalysis()` - Thread safety

### 3. MecabKoFilterIT

Tests token filter functionality.

#### Test Cases

**POS Filtering**
- `testPartOfSpeechStopFilterBasic()` - Basic POS filtering
- `testPartOfSpeechFilterNounsOnly()` - Noun extraction
- `testPartOfSpeechFilterVerbsAndAdjectives()` - Verb/adjective filtering
- `testPartOfSpeechFilterEmptyStopTags()` - No filtering

**Filter Combinations**
- `testMultipleFiltersChained()` - Multiple filters in sequence
- `testFilterWithDecompoundModes()` - Filter + decompound modes

**Attribute Preservation**
- `testFilterPreservesOffsets()` - Offset preservation
- `testReadingFormFilter()` - Reading form conversion

**Edge Cases**
- `testFilterWithSpecialCharacters()` - Special character handling
- `testFilterPerformance()` - Filter performance

### 4. MecabKoIndexIT

Tests end-to-end indexing and search scenarios.

#### Test Cases

**Basic Operations**
- `testBasicIndexingAndSearch()` - Index and search
- `testDecompoundedSearch()` - Search with decompounding
- `testMultiFieldMapping()` - Multi-field mappings

**Bulk Operations**
- `testBulkIndexing()` - Bulk indexing (100+ documents)
- `testMixedLanguageIndexing()` - Mixed language documents

**Query Types**
- `testPhraseQuery()` - Phrase queries
- `testBooleanQuery()` - Boolean queries
- `testFuzzyQuery()` - Fuzzy queries

**Advanced Features**
- `testHighlighting()` - Search result highlighting
- `testAggregations()` - Aggregation queries

**Document Operations**
- `testUpdateDocument()` - Document updates
- `testDeleteDocument()` - Document deletion

## Test Data

### Korean Samples (korean_samples.json)

15 realistic Korean documents covering:
- NLP and natural language processing
- Search engines and indexing
- AI and machine learning
- Data analysis and visualization
- Cloud computing
- Web development
- Security and encryption
- Mobile applications
- Databases
- Software testing
- DevOps and CI/CD
- Container orchestration
- API design
- Performance optimization

### Mixed Language Samples (mixed_samples.json)

20 documents with Korean-English code-switching:
- Technical documentation
- Programming concepts
- Framework and library usage
- Cloud services
- Database operations
- DevOps practices
- API design patterns

## Assertions and Validation

### Common Assertions

```java
// Token existence
assertThat(tokens, is(not(empty())));

// Token count
assertEquals(expectedCount, tokens.size());

// Token content
assertTrue("Should contain token",
    tokens.stream().anyMatch(t -> t.getTerm().equals("한국어")));

// POS tag filtering
assertFalse("Should not contain J tags",
    tokens.stream().anyMatch(t -> t.getType().startsWith("J")));

// Offset validity
assertThat(token.getStartOffset(), greaterThanOrEqualTo(0));
assertThat(token.getEndOffset(), lessThanOrEqualTo(text.length()));
```

### Performance Assertions

```java
// Latency threshold
assertTrue("Analysis should be fast", avgTime < 100); // 100ms

// Throughput verification
assertThat("Should handle many requests", requestCount, greaterThan(100));
```

## Continuous Integration

### GitHub Actions Workflow

The workflow (`.github/workflows/elasticsearch-plugin-tests.yml`) includes:

1. **Unit Tests** - Fast component tests
2. **Integration Tests** - Full cluster tests
3. **Compatibility Tests** - Multiple ES versions (8.11.3, 8.12.0, 8.13.0)
4. **Code Coverage** - JaCoCo coverage reports
5. **Performance Tests** - Benchmark execution
6. **Docker Tests** - Plugin installation in Docker container

### CI Triggers

- Push to main/master/develop branches
- Pull requests
- Manual workflow dispatch
- Changes to plugin or Rust code

### Artifacts

CI uploads:
- Test reports (HTML)
- Coverage reports (XML/HTML)
- Test logs (on failure)
- Performance results (30-day retention)

## Test Reports

### Viewing Reports Locally

```bash
# After running tests
./gradlew integrationTest

# Open HTML report
open build/reports/tests/integrationTest/index.html

# View coverage
open build/reports/jacoco/test/html/index.html
```

### Report Contents

- Test execution time
- Pass/fail status
- Standard output/error
- Stack traces (on failure)
- Test duration statistics

## Debugging Tests

### Enable Debug Logging

```bash
# Gradle debug
./gradlew integrationTest --debug

# Elasticsearch debug logs
./gradlew integrationTest -Dtests.es.logger.level=DEBUG

# Specific logger
./gradlew integrationTest -Dlogger.com.mecab.ko=DEBUG
```

### Remote Debugging

```bash
# Start with debug port
./gradlew integrationTest --debug-jvm

# Then attach debugger to port 5005
```

### Test Cluster Inspection

```bash
# Keep cluster alive after test
./gradlew integrationTest -Dtests.cluster.keep_alive=true

# Then query cluster
curl http://localhost:9200/_cat/plugins
```

## Performance Benchmarks

### Expected Performance

| Operation | Expected Time | Notes |
|-----------|---------------|-------|
| Basic analysis | < 10ms | Single sentence |
| Large document (10KB) | < 100ms | 10,000+ characters |
| Bulk indexing | > 1,000 docs/sec | 100 document batch |
| Concurrent requests | Linear scaling | Up to CPU cores |

### Running Benchmarks

```bash
# All performance tests
./gradlew integrationTest --tests "*Performance*"

# Specific benchmark
./gradlew integrationTest --tests "*.testPerformanceBenchmark"
```

### Benchmark Metrics

- Throughput (requests/sec)
- Latency (ms per request)
- P50, P95, P99 percentiles
- Memory usage
- CPU utilization

## Best Practices

### Writing New Tests

1. **Use Descriptive Names**: `testFeatureWithSpecificScenario()`
2. **One Assertion Per Concept**: Focus tests on single behavior
3. **Clean Up Resources**: Use `@After` or try-with-resources
4. **Document Edge Cases**: Comment why edge cases are important
5. **Use Test Data Files**: For complex inputs

### Test Structure

```java
public void testFeature() throws Exception {
    // 1. Setup - Create index with settings
    CreateIndexRequestBuilder builder = client().admin().indices()
        .prepareCreate("test");
    builder.setSettings(...);
    builder.get();
    ensureGreen("test");

    // 2. Execute - Perform operation
    AnalyzeAction.Response response = client().admin().indices()
        .prepareAnalyze("test", inputText)
        .setAnalyzer("mecab_ko")
        .get();

    // 3. Verify - Check results
    List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
    assertThat(tokens, is(not(empty())));

    // 4. Log - Output for debugging
    logger.info("Test produced {} tokens", tokens.size());
}
```

### Test Data Management

- Keep test data in JSON files
- Use realistic, diverse samples
- Include edge cases (empty, large, special chars)
- Document data source and purpose

## Troubleshooting

### Common Issues

**Test Cluster Startup Failure**
```
Error: Failed to start Elasticsearch cluster
Solution: Check available memory, increase heap size
export ES_JAVA_OPTS="-Xms1g -Xmx1g"
```

**Native Library Not Found**
```
Error: UnsatisfiedLinkError
Solution: Build Rust library first
cd ../rust && cargo build --release
```

**Port Already in Use**
```
Error: Address already in use
Solution: Kill existing Elasticsearch process
pkill -f elasticsearch
```

**Tests Hanging**
```
Issue: Tests don't complete
Solution: Check for infinite loops, add timeouts
@Test(timeout = 30000) // 30 seconds
```

### Getting Help

- Check test logs: `build/reports/tests/`
- Review CI failures: GitHub Actions
- Enable debug logging
- Ask in discussions: https://github.com/mecab-ko/mecab-ko/discussions

## Coverage Goals

### Target Coverage

- Line coverage: > 80%
- Branch coverage: > 70%
- Method coverage: > 85%

### Current Coverage

```bash
# Generate report
./gradlew test jacocoTestReport

# View summary
./gradlew jacocoTestCoverageVerification
```

## Contributing Tests

When contributing:

1. Add tests for new features
2. Maintain existing test coverage
3. Run full test suite before PR
4. Document complex test scenarios
5. Follow existing test patterns

## Resources

- [Elasticsearch Test Framework](https://www.elastic.co/guide/en/elasticsearch/reference/current/testing-framework.html)
- [JUnit 4 Documentation](https://junit.org/junit4/)
- [Gradle Testing Guide](https://docs.gradle.org/current/userguide/java_testing.html)
- [Project Issue Tracker](https://github.com/mecab-ko/mecab-ko/issues)
