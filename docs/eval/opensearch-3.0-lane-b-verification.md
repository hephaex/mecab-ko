# Lane B — OpenSearch 3.0.0 + Lucene 10 Compatibility Verification (worker-2)

Scope: `search-plugins/opensearch` analysis Tokenizer/Filters/Analyzer, plugin class, and
version metadata. Verification method: static bytecode API cross-reference against the
actual resolved dependency jars (offline; OpenSearch/Lucene are `compileOnly`). No
project-wide build was run.

## Reference artifacts (Gradle cache)
- `org.apache.lucene:lucene-core:10.1.0` (classfile major 65 = Java 21)
- `org.apache.lucene:lucene-analysis-common:10.1.0`
- `org.opensearch:opensearch:3.0.0` (classfile major 65 = Java 21)

Note: 10.1.0 is the resolved patch cached locally; build pins 10.0.0. Lucene does not
remove/rename public API within a 10.x minor line, so presence in 10.1.0 confirms presence
in 10.0.0 for the symbols below. All checked classes are compiled to Java 21 bytecode,
which independently establishes the JDK-21 baseline.

## Per-file result

### MecabKoTokenizer.java — COMPATIBLE (no change)
Extends `org.apache.lucene.analysis.Tokenizer`.
- `Tokenizer` has `protected <init>()V` and `<init>(AttributeFactory)V`; implicit `super()` OK.
- Protected field `input : java.io.Reader` present → `delegate.resetTokenizer(input)` valid.
- `reset()V`/`end()V`/`close()V` present and declare `throws IOException`; `correctOffset(I)I` present.
- Attributes used: `CharTermAttribute.setEmpty()/append(CharSequence|String)`, `OffsetAttribute.setOffset(II)`,
  `PositionIncrementAttribute.setPositionIncrement(I)`, `TypeAttribute.setType(String)` — all present.

### MecabKoPartOfSpeechStopFilter.java — COMPATIBLE (no change)
Extends `org.apache.lucene.analysis.FilteringTokenFilter` (lives in lucene-core in 10.x).
- `<init>(TokenStream)V` present; `accept()Z` is `protected abstract` **and declares `throws IOException`**,
  so the override `protected boolean accept() throws IOException` compiles.
- `TypeAttribute.type()` present.

### MecabKoReadingFormFilter.java — COMPATIBLE (no change)
Extends `org.apache.lucene.analysis.TokenFilter`.
- `<init>(TokenStream)V` present; protected field `input : TokenStream` present → `input.incrementToken()` valid.
- `KeywordAttribute.isKeyword()Z`, `CharTermAttribute.setEmpty()/append(...)` present.

### MecabKoAnalyzer.java — COMPATIBLE (no change)
Extends `org.apache.lucene.analysis.Analyzer`.
- Abstract `createComponents(String)Analyzer$TokenStreamComponents` matches override signature.
- `TokenStreamComponents.<init>(Tokenizer, TokenStream)V` and `<init>(Tokenizer)V` present.

### plugin/MecabKoPlugin.java — COMPATIBLE (no change)
Implements `org.opensearch.plugins.AnalysisPlugin`.
- `getAnalyzers()Map`, `getTokenizers()Map`, `getTokenFilters()Map` present (overrides match).
- Functional `AnalysisModule$AnalysisProvider.get(IndexSettings, Environment, String, Settings)Object`
  present → the 4-arg lambdas match.
- `Plugin` base exposes `<init>()V`; `MecabKoPlugin(Settings)` calls implicit `super()` — valid Java.
  (Constructor-injection of `Settings` is resolved by OpenSearch's reflective plugin loader at runtime.)

### Shared dependency (`:common`, not in any lane's edit scope) — COMPATIBLE
`ReadingFormAttributeImpl extends org.apache.lucene.util.AttributeImpl`:
- Overrides exactly the abstract set in Lucene 10 `AttributeImpl`: `clear()V`, `copyTo(AttributeImpl)V`,
  `reflectWith(AttributeReflector)V`. `AttributeReflector.reflect(Class, String, Object)V` matches.
- No abstract `newInstance()` required. No removed API used.

## Change applied
- `src/main/resources/plugin-descriptor.properties`: `java.version=17` → `java.version=21`.
  Rationale: build compiles `sourceCompatibility=targetCompatibility=VERSION_21` (bytecode major 65);
  OpenSearch 3.0.0 baseline minimum JDK is 21 (aligned with Lucene 10 requiring JDK 21). Declaring
  `17` would pass the descriptor gate on JDK 17-20 yet fail at class load with
  `UnsupportedClassVersionError`. `build.gradle.kts` was already `VERSION_21` (correct); no change.

Sources: OpenSearch docs "Breaking changes" (minimum supported JDK is JDK 21);
OpenSearch issue #14011 (align 3.0 JDK baseline to JDK-21 with Lucene 10); local jar bytecode major 65.

## Observation for Lane C / leader (out of Lane B file scope)
OpenSearch 3.0 replaces the Java Security Manager with a Java-agent-based framework. The plugin ships
`src/main/resources/plugin-security.policy`; whether/how a JSM-style policy is still honored under the
new agent should be validated during Lane C module verification. Not modified here (outside Lane B targets).
