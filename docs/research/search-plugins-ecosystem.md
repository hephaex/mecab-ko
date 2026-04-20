# Korean Morphological Analyzer Plugins: ES/OS Ecosystem Research

Date: 2026-04-20

## Summary

The ES/OS analysis plugin landscape for Korean has consolidated around Nori (Lucene-native, pure Java). Competing JNI-based alternatives (Seunjeon/mecab-ko-lucene-analyzer) are abandoned at ES 6.x. Elasticsearch 8.x introduced a "stable plugin API" that decouples plugins from ES version pinning — this is the correct target API for mecab-ko-elasticsearch. OpenSearch 3.x moved to Lucene 10 + JDK 21, replacing the Java Security Manager with a Java-agent model, which changes JNI security requirements. A Nori drop-in replacement is achievable: the surface API is narrow (4 components, 3 settings, 35 POS tags).

---

## 1. Elasticsearch 8.x Nori Plugin

### 1.1 Plugin Components

The Nori plugin exposes exactly four named components:

| ES name | Lucene class | Type |
|---|---|---|
| `nori` | KoreanAnalyzer | Analyzer |
| `nori_tokenizer` | KoreanTokenizer | Tokenizer |
| `nori_part_of_speech` | KoreanPartOfSpeechStopFilter | TokenFilter |
| `nori_readingform` | KoreanReadingFormFilter | TokenFilter |

A fifth undocumented filter `nori_number` appeared in master branch docs (normalizes Korean number words).

### 1.2 nori_tokenizer Settings

| Setting | Type | Default | Values |
|---|---|---|---|
| `decompound_mode` | enum | `discard` | `none`, `discard`, `mixed` |
| `user_dictionary` | file path | — | Path relative to config dir |
| `user_dictionary_rules` | string array | — | Inline rules (ES 7.6+) |
| `discard_punctuation` | bool | `true` | `true`, `false` |

**user_dictionary_rules format** (inline, preferred for cloud):
```json
["c++", "C쁠쁠", "세종", "세종시 세종 시"]
```
First token = custom noun. Compound segmentation follows after space.

**user_dictionary file format** (one entry per line):
```
세종 세종시
c++
```

### 1.3 nori_part_of_speech Default stoptags

Default stoptags list (18 tags):
```
E, IC, J, MAG, MAJ, MM, SP, SSC, SSO, SC, SE, XPN, XSA, XSN, XSV, UNA, NA, VSV
```

### 1.4 nori_analyzer Inherited Settings

When using `"type": "nori"` the analyzer accepts:
- `decompound_mode` (from nori_tokenizer)
- `user_dictionary` (from nori_tokenizer)
- `stoptags` (from nori_part_of_speech)

### 1.5 Token Attributes (Lucene AttributeSource)

KoreanTokenizer sets these attributes beyond the standard set:

| Attribute Class | Content |
|---|---|
| `CharTermAttribute` | Surface form (term text) |
| `OffsetAttribute` | startOffset, endOffset (char positions) |
| `PositionIncrementAttribute` | Position increment (1 for normal, 0 for decompound siblings) |
| `PositionLengthAttribute` | Position length |
| `TypeAttribute` | word type: `<HANGUL>`, `<HANJA>`, etc. |
| `PartOfSpeechAttribute` | Korean POS tag (nori-specific) |
| `ReadingAttribute` | Reading/pronunciation (nori-specific) |

The two Korean-specific attributes (`PartOfSpeechAttribute`, `ReadingAttribute`) are in package `org.apache.lucene.analysis.ko`.

### 1.6 Complete POS Tag Set (Lucene 9.x/10.x `POS.Tag`)

35 tags total (Sejong corpus tagset):

| Tag | Description |
|---|---|
| NNG | General Noun |
| NNP | Proper Noun |
| NNB | Dependent noun |
| NNBC | Dependent noun (bound) |
| NR | Numeral |
| NP | Pronoun |
| VV | Verb |
| VA | Adjective |
| VX | Auxiliary Verb or Adjective |
| VCP | Positive designator |
| VCN | Negative designator |
| MM | Determiner |
| MAG | General Adverb |
| MAJ | Conjunctive adverb |
| IC | Interjection |
| J | Ending Particle (unified) |
| E | Verbal endings (unified) |
| XPN | Prefix |
| XSN | Noun Suffix |
| XSV | Verb Suffix |
| XSA | Adjective Suffix |
| XR | Root |
| SF | Terminal punctuation (? ! .) |
| SP | Space |
| SS | Quotation marks / parentheses |
| SSO | Opening brackets |
| SSC | Closing brackets |
| SC | Separator (· / :) |
| SE | Ellipsis |
| SY | Other symbol |
| SL | Foreign language |
| SH | Chinese character |
| SN | Number |
| UNKNOWN | Unknown morpheme |
| UNA | Unknown |
| NA | Unknown |
| VSV | Unknown |

Note: Nori uses unified J (joins JKS/JKO/JKB/JKV/JKQ/JC/JX) and unified E (joins EF/EC/ETN/ETM). This is important for compatibility with mecab-ko-core's fine-grained MeCab tags.

### 1.7 Lucene Version Alignment

- ES 8.11–8.19 uses Lucene 9.x (9.5–9.12 range depending on patch)
- ES 9.x (upcoming) will use Lucene 10
- OpenSearch 3.x uses Lucene 10.1.0

The `KoreanTokenizer` API carries an **experimental warning** even in Lucene 10.

---

## 2. Elasticsearch 8.x Plugin API: Stable vs Classic

### 2.1 Classic Plugin API (legacy)

- Plugin class extends `Plugin` and implements `AnalysisPlugin`
- `plugin-descriptor.properties` with mandatory `classname` field
- Pinned to exact ES version — must recompile for every minor/patch
- Compile-time dependency on `elasticsearch:server` (internal APIs)

### 2.2 Stable Plugin API (ES 8.7+, recommended)

The stable API is the correct target for new analysis plugin development.

**Key distinctions:**
- No `classname` in descriptor (stable plugins use `named_components.json` instead)
- No `elasticsearch:server` dependency — only `plugin-api`, `plugin-analysis-api`, `lucene-analysis-common`
- Binary compatible across minor and bugfix versions within same major
- Components registered via `@NamedComponent` annotation

**Four factory interfaces in `plugin-analysis-api`:**
```java
AnalyzerFactory     // creates Lucene Analyzer
TokenizerFactory    // creates Lucene Tokenizer
TokenFilterFactory  // creates Lucene TokenStream (filter)
CharFilterFactory   // creates Lucene CharFilter
```

**Plugin ZIP structure:**
```
my-plugin.zip
├── my-plugin.jar
├── stable-plugin-descriptor.properties
└── named_components.json
```

**stable-plugin-descriptor.properties fields:**

| Field | Required | Notes |
|---|---|---|
| `name` | yes | Plugin identifier |
| `version` | yes | Plugin version |
| `description` | yes | Short description |
| `java.version` | yes | Java version compiled against |
| `elasticsearch.version` | yes | ES version compiled against |
| `classname` | NO | Must be absent for stable plugins |
| `modulename` | optional | For JPMS module isolation |
| `extended.plugins` | optional | SPI extension points |
| `has.native.controller` | optional | For native sidecar processes |
| `licensed` | optional | License agreement flag |

**named_components.json format:**
```json
{
  "org.elasticsearch.plugin.analysis.api.TokenizerFactory": {
    "mecab_ko_tokenizer": "org.example.MecabKoTokenizerFactory"
  },
  "org.elasticsearch.plugin.analysis.api.TokenFilterFactory": {
    "mecab_ko_part_of_speech": "org.example.MecabKoPartOfSpeechFilterFactory",
    "mecab_ko_readingform": "org.example.MecabKoReadingFormFilterFactory"
  },
  "org.elasticsearch.plugin.analysis.api.AnalyzerFactory": {
    "mecab_ko": "org.example.MecabKoAnalyzerFactory"
  }
}
```

**build.gradle skeleton:**
```groovy
plugins {
    id 'elasticsearch.stable-esplugin'
    id 'elasticsearch.yaml-rest-test'
}

ext {
    pluginApiVersion = '8.11.0'    // from build-tools-internal/version.properties
    luceneVersion = '9.8.0'        // matching Lucene version
}

esplugin {
    name 'analysis-mecab-ko'
    description 'MeCab-Ko Korean analysis plugin'
}

dependencies {
    compileOnly "org.elasticsearch.plugin:plugin-api:${pluginApiVersion}"
    compileOnly "org.elasticsearch.plugin:plugin-analysis-api:${pluginApiVersion}"
    compileOnly "org.apache.lucene:lucene-analysis-common:${luceneVersion}"
}
```

---

## 3. OpenSearch 3.x Plugin API

### 3.1 Major Breaking Changes from 2.x

| Area | Change |
|---|---|
| Lucene | Upgraded from 9.x to **Lucene 10.1.0** |
| JDK | Minimum is now **Java 21** (was 17) |
| Security Manager | **Java Security Manager removed** — replaced by Java agent + systemd hardening |
| API | Deprecated terms removed from Java API |
| Packages | Refactored to eliminate top-level split packages (JPMS support) |

All custom plugins must be recompiled for OS 3.x compatibility.

### 3.2 Korean Analyzer in OpenSearch

OpenSearch ships `analysis-nori` as an **optional installable plugin** (not built-in):
- `opensearch-analysis-nori-plugin` 3.3.0 available for OS 3.3.0
- Available via Amazon OpenSearch Service as of Oct 2023 (plus Sudachi, Pinyin, STConvert)
- Package tracking on Arch Linux confirms: version 3.2.0 build date March 9, 2025

The plugin exposes identical API to ES Nori (same Lucene classes underneath).

### 3.3 OpenSearch Security Model for JNI (3.x)

The Java Security Manager is removed in OS 3.0. The replacement approach:
1. **systemd hardening**: OS-level system call restrictions, filesystem isolation, capability dropping
2. **Custom Java agent**: Bytecode instrumentation to intercept file/socket operations per-plugin

The `plugin-security.policy` file format is **retained** for the Java agent configuration (same syntax as JSM policy files) — this is intentional to minimize migration effort.

**For JNI specifically:** The systemd layer provides coarser-grained controls. The Java agent model may not intercept `System.load()` the same way JSM did. The k-NN plugin (reference implementation for JNI in OS) uses:
- `System.loadLibrary()` wrapped in `AccessController.doPrivileged()`
- Lazy loading (libraries loaded only on first use)
- `lib/` directory inside plugin ZIP for native `.so` files

### 3.4 OpenSearch Plugin Build (3.x Gradle)

```groovy
// build.gradle pattern
apply plugin: 'opensearch.esplugin'

opensearchplugin {
    name 'analysis-mecab-ko'
    description 'MeCab-Ko Korean analysis plugin'
    classname 'org.example.MecabKoPlugin'
}

// Version is parameterized
def opensearch_version = System.getProperty("opensearch.version", "3.0.0")

dependencies {
    compileOnly "org.opensearch:opensearch:${opensearch_version}"
}
```

The `-Dopensearch.version=` flag allows building against any target version without changing code.

**Note:** OpenSearch does NOT have a "stable plugin API" equivalent to ES 8.7+. OS plugins remain version-pinned (classic model only). This is a significant development burden vs ES.

---

## 4. JNI-Based Analysis Plugins: Patterns and Challenges

### 4.1 Precedent: OpenSearch k-NN Plugin

The k-NN plugin is the reference implementation for JNI in OpenSearch/ES:
- Three native libraries: `libopensearchknn_nmslib`, `libopensearchknn_faiss`, `libopensearchknn_common`
- Lazy loading: unused engines are never loaded
- Pattern: `AccessController.doPrivileged(() -> System.loadLibrary("opensearchknn_faiss"))`
- Native libs packaged in `lib/` inside the plugin ZIP
- `plugin-security.policy` grants `RuntimePermission "loadLibrary.opensearchknn_faiss"`

### 4.2 Historical Precedent: mecab-ko-lucene-analyzer (Seunjeon)

This was the previous JNI approach for Korean:
- Used mecab-java (JNI wrapper around libmecab C library)
- Required `libMeCab.so` installed system-wide (`/usr/local/lib`)
- ES 2.0's Security Manager blocked `System.loadLibrary()` — plugin broke
- Workaround required explicit `plugin-security.policy` with `loadLibrary.MeCab` permission
- Never updated past ES 6.1.1; forks (seunjeon-elasticsearch-7) handle ES 7 only
- **Conclusion: dead project; no OS 2.x/3.x or ES 8.x version exists**

### 4.3 Known JNI Challenges in ES/OS Plugins

1. **Security Manager / Java agent**: `System.loadLibrary()` requires explicit policy grant
2. **ClassLoader isolation**: Plugin classloaders are separate from core; native libs must be accessible from plugin classloader's namespace
3. **Platform-specific `.so` packaging**: Must include `.so`/`.dll`/`.dylib` per platform in plugin ZIP
4. **Missing lib/ in Maven artifacts**: k-NN plugin Maven artifact [historically missing `lib/`](https://github.com/opensearch-project/k-NN/issues/2643) — must use ZIP distribution
5. **`UnsatisfiedLinkError`**: Triggered if lib not in `java.library.path` or if wrong architecture
6. **Security entropy**: ES 8.x stable plugin API does not define a mechanism for native library loading — this may require falling back to classic plugin approach for JNI

### 4.4 Alternative: JNA (Java Native Access)

JNA avoids some classloader issues by loading libs through a different mechanism. The ES k-NN plugin itself does not use JNA; it uses raw JNI. ES core uses JNA for OS-level calls (mlockall, etc.).

---

## 5. Nori Drop-in Replacement: Requirements Analysis

### 5.1 Index Settings API Surface

A true drop-in must accept the same index settings JSON that Nori accepts:

```json
{
  "analysis": {
    "analyzer": {
      "my_analyzer": {
        "type": "mecab_ko",
        "decompound_mode": "mixed",
        "user_dictionary": "userdict_ko.txt",
        "stoptags": ["J", "E"]
      }
    },
    "tokenizer": {
      "my_tokenizer": {
        "type": "mecab_ko_tokenizer",
        "decompound_mode": "discard",
        "user_dictionary_rules": ["세종", "세종시 세종 시"],
        "discard_punctuation": true
      }
    },
    "filter": {
      "my_pos_filter": {
        "type": "mecab_ko_part_of_speech",
        "stoptags": ["J", "E", "SF"]
      }
    }
  }
}
```

### 5.2 Token Attribute Compatibility

Must set identical attributes for upstream consumers (scoring, highlighting):

| Attribute | Nori behavior | mecab-ko-elasticsearch current status |
|---|---|---|
| `CharTermAttribute` | Surface | Implemented (surface field) |
| `OffsetAttribute` | Char offsets | Implemented |
| `PositionIncrementAttribute` | 1 normally, 0 for decompound siblings | Implemented |
| `PositionLengthAttribute` | Span of compound | Implemented |
| `PartOfSpeechAttribute` | Korean POS | Implemented (pos_tag field) |
| `ReadingAttribute` | Pronunciation | Implemented (reading field) |
| `TypeAttribute` | `<HANGUL>` etc | Implemented (word_type) |

The mecab-ko-elasticsearch crate already models all required attributes.

### 5.3 POS Tag Mapping Gap

MeCab-Ko uses fine-grained Sejong tags (e.g., `JKS`, `JKO`, `JKB` for case particles). Nori collapses these to `J` and `E`. The `nori_compat` module in mecab-ko-core already implements this mapping — this is the critical bridge.

### 5.4 Decompound Mode Semantics

The `mixed` mode requires producing both compound and constituent tokens at the same position:
- Compound: position_increment=1, position_length=N (span)
- Constituent i: position_increment=0 (if i>0), position_length=1

This is what enables phrase queries and proximity scoring to work correctly across decompounded compounds.

### 5.5 What is NOT required for functional replacement

- Identical internal dictionary (mecab-ko-dic vs system nori dict) — users accept different tokenization
- Identical performance (but must be competitive — target: within 2x of Nori, current benchmark shows 25-33% faster)
- `nori_number` filter (experimental, rarely used)

---

## 6. Competitor Analysis

### 6.1 Nori (Official, Lucene-native)

- **Status**: Active, maintained by Elastic + Lucene community
- **ES support**: All ES 6.x+ and OS 1.x+ (as optional plugin in OS)
- **Architecture**: Pure Java, Lucene module, no JNI
- **Dictionary**: mecab-ko-dic compiled into binary FST (bundled)
- **Strengths**: Zero extra dependencies, stable, Elastic SLA
- **Weaknesses**: Fixed dictionary, slow custom dict reload, no hot-reload

### 6.2 Seunjeon (은전한닢)

- **Status**: Dead. Last ES version: 6.1.1 (2018). Forks cover ES 7 only.
- **Architecture**: Scala + JNI wrapping libmecab
- **OS support**: None (no OS version exists)
- **Why abandoned**: JNI complexity, Scala build toolchain friction, ES Security Manager breakage at 2.0

### 6.3 mecab-ko-lucene-analyzer (jaepil fork)

- **Status**: Archived/inactive. Mirrors the Bitbucket eunjeon project.
- **Architecture**: Java + JNI (mecab-java-0.996)
- **ES support**: Up to ES 2.x (broken by Security Manager in ES 2.0+)
- **Note**: This is the direct predecessor to what mecab-ko-elasticsearch aims to replace

### 6.4 elasticsearch-analysis-openkoreantext (OKT)

- **Status**: Abandoned. Last release: ES 6.1.1 (December 2017).
- **Architecture**: Pure Java (wraps open-korean-text/Twitter Korean tokenizer, Scala)
- **Components**: openkoreantext-normalizer (char filter), openkoreantext-tokenizer, stemmer/dedup/phrase filters
- **Strengths**: No JNI, handles colloquial language well
- **Weaknesses**: No morphological accuracy for technical/news text, no updates in 7+ years

### 6.5 KOMORAN (komoran-tokenizer for ES)

- **Status**: Low activity. komoran-tokenizer for ES last updated March 2024 per search results.
- **Architecture**: Pure Java (KOMORAN is Java-native, no JNI)
- **POS accuracy**: Good for general Korean, less specialized than mecab-ko-dic
- **ES support**: Unknown version ceiling; likely ES 7.x at best

### 6.6 Amazon OpenSearch Service — Bundled Analyzers (Oct 2023)

AWS added these as first-party optional plugins:
- Nori (Korean) — available since earlier
- Sudachi (Japanese) — new Oct 2023
- Pinyin (Chinese)
- STConvert (Chinese)

No new third-party Korean analyzers were added.

---

## 7. Strategic Recommendations for mecab-ko Plugin Implementation

### 7.1 Target ES 8.x Stable Plugin API First

Use `elasticsearch.stable-esplugin` Gradle plugin. This avoids version pinning and gives binary compatibility across ES 8.x minor versions. Register:
- `AnalyzerFactory` → `mecab_ko`
- `TokenizerFactory` → `mecab_ko_tokenizer`
- `TokenFilterFactory` → `mecab_ko_part_of_speech`
- `TokenFilterFactory` → `mecab_ko_readingform`

### 7.2 JNI Loading Strategy

The Stable Plugin API does NOT define a native library loading mechanism. Options:
1. **Hybrid approach**: Use stable plugin API for Java registration, but fall back to classic plugin descriptor to get `plugin-security.policy` support for `System.loadLibrary()`
2. **Java agent approach (OS 3.x)**: Retain `plugin-security.policy` — the Java agent model preserves the same policy syntax
3. **Reference implementation**: Study opensearch-project/k-NN `JNIService.java` for the `AccessController.doPrivileged` + `System.loadLibrary` pattern

Native library must be in `lib/` subdirectory of the plugin ZIP (not root). The `java.library.path` must include this path — typically handled by the plugin loader if native controller is declared.

### 7.3 Separate ES and OS Plugin Builds

Given that:
- ES 8.x has Stable Plugin API (Lucene 9.x, JDK 17+)
- OS 3.x is Classic Plugin API only (Lucene 10.x, JDK 21+)
- Lucene 10 has breaking API changes vs Lucene 9

Recommend a shared `core-java` module with the Lucene-agnostic logic, and separate `elasticsearch-plugin` and `opensearch-plugin` modules that adapt to the respective Lucene versions.

### 7.4 Nori Drop-in: Use Same Component Names

To be a true drop-in, use the same ES component type names:
- `"type": "nori_tokenizer"` — NOT `"type": "mecab_ko_tokenizer"`

This requires the plugin to register its factories under the nori names. This is achievable with `@NamedComponent("nori_tokenizer")` but creates a namespace conflict if both plugins are installed. Recommend offering both names: default to `nori_*` names with a `mecab_ko_*` alias option.

### 7.5 Priority Order

1. ES 8.x stable plugin — highest adoption, best forward compatibility
2. OS 2.x classic plugin — large installed base (AWS MSK uses OS 2.x)
3. OS 3.x classic plugin — growing, requires Lucene 10 adaptation
4. Publish to Elastic plugin registry / OS artifact hub

---

## Sources

- [Nori analyzer (ES 8.17)](https://www.elastic.co/guide/en/elasticsearch/plugins/current/analysis-nori-analyzer.html)
- [nori_tokenizer settings](https://www.elastic.co/docs/reference/elasticsearch/plugins/analysis-nori-tokenizer)
- [nori_part_of_speech filter (master)](https://www.elastic.co/guide/en/elasticsearch/plugins/master/analysis-nori-speech.html)
- [Creating stable plugins (ES 8.x)](https://www.elastic.co/docs/extend/elasticsearch/creating-stable-plugins)
- [Stable plugin descriptor format](https://www.elastic.co/docs/extend/elasticsearch/plugin-descriptor-file-stable)
- [Classic plugin descriptor format](https://www.elastic.co/docs/extend/elasticsearch/plugin-descriptor-file-classic)
- [Example text analysis plugin (ES)](https://www.elastic.co/docs/extend/elasticsearch/example-text-analysis-plugin)
- [POS.Tag Lucene 9.12.1](https://lucene.apache.org/core/9_12_1/analysis/nori/org/apache/lucene/analysis/ko/POS.Tag.html)
- [KoreanTokenizer Lucene 10.0.0](https://lucene.apache.org/core/10_0_0/analysis/nori/org/apache/lucene/analysis/ko/KoreanTokenizer.html)
- [OpenSearch 3.0 announcement](https://opensearch.org/blog/opensearch-3-0-what-to-expect/)
- [OpenSearch JSM replacement blog](https://opensearch.org/blog/finding-a-replacement-for-jsm-in-opensearch-3-0/)
- [OpenSearch breaking changes](https://docs.opensearch.org/latest/breaking-changes/)
- [OpenSearch plugin template (Java)](https://github.com/opensearch-project/opensearch-plugin-template-java)
- [OpenSearch k-NN JNI library docs](https://docs.opensearch.org/docs/2.0/search-plugins/knn/jni-libraries/)
- [elasticsearch-analysis-openkoreantext](https://github.com/open-korean-text/elasticsearch-analysis-openkoreantext)
- [seunjeon-elasticsearch-7 (fork)](https://github.com/likejazz/seunjeon-elasticsearch-7)
- [AWS: OpenSearch 4 new language analyzers (Oct 2023)](https://aws.amazon.com/about-aws/whats-new/2023/10/amazon-opensearch-service-adds-support-for-four-new-language-analyzers/)
- [Arch Linux opensearch-analysis-nori-plugin 3.3.0](https://archlinux.org/packages/extra/x86_64/opensearch-analysis-nori-plugin/)
- [Stable Plugin API GitHub issue #88980](https://github.com/elastic/elasticsearch/issues/88980)

---

## Learning Points

1. The ES 8.7+ Stable Plugin API breaks the version-pinning problem — binary compatible across minor versions within a major. This is the single most important architectural choice for ES plugin longevity.
2. OS 3.x has no stable plugin equivalent — plugins remain version-pinned, making ES and OS require separate build artifacts even with shared Java logic.
3. JNI in ES/OS plugins requires `plugin-security.policy` + `AccessController.doPrivileged` pattern; the OS 3.x Java-agent model preserves the same policy file syntax. The k-NN plugin is the canonical reference implementation.
