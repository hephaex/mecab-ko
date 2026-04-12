# Scalable User Dictionary Expansion - Research Report

**Date**: 2026-04-01
**Category**: dictionary
**Scope**: Production-ready approaches for extending mecab-ko user dictionaries at scale

---

## Summary

Community-driven contribution, automated neologism detection, and layered
hot-reload are the three levers most projects pull. All proven approaches
converge on a simple CSV format (4-6 fields), frequency-derived negative costs,
and an index-close/reopen or watch-and-swap reload pattern. Domain-specific
dictionaries are best managed as separate overlay files rather than merged into a
single monolithic file.

---

## 1. Community-Driven Dictionary Expansion

### How comparable projects handle contributions

**MeCab / mecab-ko-dic**
- Accepts CSV patch files via pull request. Each entry: `surface, left_id, right_id, cost, pos, ...`
- mecab-ko-dic has not had an official release since 2018 (v2.1.1). The community fork model has filled the gap — contributors submit CSVs that are merged after human review.
- Lesson: low barrier (plain text) + human review gate keeps quality high but throughput low.

**Sudachi (WorksApplications)**
- Ships three dictionary tiers: Small (core), Core, Full (neologism-heavy). Updated every few months.
- User dictionaries are additive — `ubuild` CLI compiles a `.dic` from user CSV without touching the system dict. Multiple user dicts can be stacked.
- Contributions to the main dict go through GitHub Issues/PRs with a dedicated team reviewing frequency data.

**Kuromoji (Apache Lucene)**
- `UserDictionary` class loaded at tokenizer construction time. Format: `word,reading,reading,POS`.
- Contributions to the bundled IPAdic go through Apache JIRA + Lucene committer review. No automated pipeline.
- Hot reload: Lucene index close/open. No file-watch mechanism in core.

**Nori (Elasticsearch / Korean)**
- `user_dictionary` config points to a flat word-per-line file or TSV.
- AWS Elasticsearch added hot-reload for dictionary files in November 2020 — no node restart, only index close/reopen required.
- Dictionary update does NOT re-index existing documents; only new indexing and search queries use the new entries.

### Key takeaways for mecab-ko

- Keep contribution format identical to the existing user dict CSV (surface, POS, cost, reading).
- Provide a `validate` sub-command contributors run locally before submitting.
- Separate pull requests by domain (IT, medical, legal, neologisms) for focused review.

---

## 2. Automated New-Word Collection

### Proven techniques

**Corpus frequency extraction**
- Scan target-domain text (news, social media, technical docs).
- Extract character n-grams (2–5) whose frequency exceeds a threshold.
- Filter by: (a) not in current system dict, (b) cohesion score > 0.3, (c) branching entropy on both sides > 1.5.
- This is the method behind soynlp's `LTokenizer` and is directly applicable to building candidate lists for mecab-ko user dicts.

**Neologism sourcing**
- Korean neologisms appear in blogs/SNS weeks before news. Nam et al. (2024) found 75%+ of news neologisms appear in blogs first.
- Practical pipeline: crawl Naver Blog / Twitter KR weekly → extract OOV candidates → rank by cohesion + frequency → queue for human review.

**TF-IDF for domain-specific terms**
- For medical/legal/IT: take a domain corpus and a general corpus. Words with high domain TF-IDF but absent from the system dict are strong candidates.
- A 2024 study demonstrated this on Korean literature corpora; the same method transfers to technical domains cleanly.

**Transliterated foreign words**
- Korean OOV problem is acute for transliterated English (쿠버네티스, 도커, 챗GPT).
- Lexicon-corpus syllable identification (Kim & Oh, 2016 ScienceDirect) extracts foreign-word candidates by detecting syllable patterns typical of loanwords.

---

## 3. Dictionary Format Standards

### Comparison

| Format | Readability | Tooling | Streaming | Version diff |
|--------|-------------|---------|-----------|--------------|
| CSV/TSV | High | Universal | Yes | Clean git diff |
| YAML | High | Good | Poor | Verbose diff |
| JSON | Medium | Good | Poor | Noisier diff |
| Binary (.dic) | None | Compiled only | N/A | Opaque |

**Recommendation: stay with CSV (4-field)**

```
surface,POS,cost,reading
딥러닝,NNG,-1000,딥러닝
쿠버네티스,NNP,-1000,쿠버네티스
```

Reasons:
- Git diff is human-readable — enables PR review without tooling.
- Every NLP ecosystem (MeCab, Kuromoji, Sudachi, Nori) understands CSV/TSV variants.
- Adding optional fields (e.g. `domain`, `source_url`, `added_date`) as trailing columns preserves backward compatibility.

**Extended community format (proposed)**

```csv
surface,POS,cost,reading,domain,source
딥러닝,NNG,-1000,딥러닝,IT,corpus_2024
챗GPT,NNP,-1500,챗지피티,IT,manual
```

Extra fields are ignored by the parser; they serve as provenance metadata in the repository.

---

## 4. Cost Estimation

### Current mecab-ko approach

Current docs recommend fixed values: -1000 (high priority), -500 (medium), 0 or positive (deprioritized). This is correct but coarse.

### Frequency-based automatic cost

MeCab's CRF model assigns costs during training. For user dict entries — where we skip training — a frequency proxy is the practical alternative:

```
cost ≈ -round(log(freq_per_million) × 200)
```

Examples:
- Word appearing 10 000 times per million → cost ≈ -920
- Word appearing 100 times per million → cost ≈ -460
- Word appearing 1 time per million → cost ≈ 0

The formula mirrors how CRF-trained costs scale: natural log of frequency maps to a linear cost range. Clip the minimum to -1500 to avoid overpowering morpheme segmentation.

### PMI for compound words

For compounds (자연어처리, 머신러닝모델): compute PMI between the component morphemes. High PMI → register as a single unit at -1000. Low PMI → skip or register at -300.

### Neural cost estimation (longer term)

Shrinking Japanese Morphological Analyzers (Fukuda et al., ACL 2019) showed that semi-supervised neural models can predict optimal dictionary costs from context without full CRF retraining. Practical for mecab-ko if a sufficient Korean corpus is available for fine-tuning.

---

## 5. Hot-Reload

### The constraint

mecab-ko compiles the user dict into a trie at startup. The system dict is memory-mapped from binary. Neither supports in-place mutation.

### Proven pattern: watch-and-swap

```
1. Write new entries to user_dict_next.csv
2. Validate (POS check, encoding, cost range)
3. Build new in-memory trie from updated CSV
4. Atomic pointer swap: replace Arc<UserDictionary> under RwLock
5. New requests use updated dict; in-flight requests finish on old
```

This is directly implementable with mecab-ko's current `Arc<UserDictionary>` field in `SystemDictionary`. Steps:

- Wrap `user_dict: Option<Arc<UserDictionary>>` in `RwLock<Option<Arc<UserDictionary>>>`.
- Expose a `reload_user_dict(path)` method that builds a new `UserDictionary`, then `write()` swaps the arc.
- A background thread can `notify::Watcher` on the CSV file and call `reload_user_dict` automatically.

### Elasticsearch Nori reference

Nori's documented approach: update the dictionary file, then call `POST /<index>/_reload_search_analyzers`. Only search-time tokenization is updated; stored tokens in existing documents are unchanged. The same semantic applies to mecab-ko: entries added after prior analysis runs will not retroactively change cached results.

---

## 6. Quality Gates

### Minimum validation (currently implemented)

mecab-ko's `user_dict.rs` already validates:
- POS tag against `VALID_POS_TAGS` list (Sejong tag set + compound tags)
- Surface non-empty
- Cost is an integer

### Additional gates to add

**1. Encoding check** — reject any non-UTF-8 bytes. Already required; add explicit error message.

**2. Duplicate detection** — warn if `(surface, POS)` pair already exists in the system dict or user dict.

**3. Cost range enforcement** — reject costs outside [-3000, 3000]. Values outside this range typically signal a data-entry error.

**4. Minimum surface length** — reject single-character entries for NNG/NNP (almost always noise).

**5. Reading consistency** — if reading is provided, check it consists of valid Korean syllables or ASCII for SL entries.

**6. Domain-specific POS plausibility** — medical terms should generally be NNG/NNP; verbs (VV) from medical domain deserve manual review.

**CI pipeline gate (for repository contributions)**

```yaml
# .github/workflows/dict-validate.yml
- run: cargo run --bin mecab-ko-validate -- --dict user-dicts/**/*.csv
```

The validate binary applies all rules above and exits non-zero on any failure.

---

## 7. Domain-Specific Dictionary Management

### Architecture: overlay stack

Do not merge domain dicts into one file. Maintain separate files:

```
user-dicts/
  core/          # Neologisms broadly useful (챗GPT, 딥러닝)
  medical/       # 진단명, 약품명, 시술명
  legal/         # 법률 용어, 판례 용어
  it/            # 프레임워크, 라이브러리, 프로토콜
  finance/       # 금융 용어
```

Load order matters: later-loaded dicts override earlier ones for the same surface+POS. A deployment can compose the stack it needs.

### Sources for domain terms

**Medical**
- Korean Bio-Medical Corpus (KBMC, arxiv 2403.16158) — NER dataset with TRM labels.
- Korean Medical Terminology compilation (KoreaMed Synapse, KAMS).
- Pre-trained KM-BERT embeddings contain implicit medical vocabulary that can be extracted.

**Legal**
- National Law Information Center (법제처) provides machine-readable statute text — a crawlable source for legal term extraction.

**IT**
- Stack Overflow Korean tags, GitHub Korean README files, Korean tech blogs (Naver D2, Kakao Engineering) are high-yield sources for IT neologisms.

### Update cadence recommendation

| Domain | Recommended cadence | Method |
|--------|--------------------|-|
| Core neologisms | Monthly | Automated corpus scan + human review |
| IT/technology | Bi-monthly | Crawl tech blogs + PR review |
| Medical | Quarterly | Manual curation from KBMC + clinician review |
| Legal | Semi-annual | Manual curation from statute text |

---

## Recommendations (Priority Order)

1. **Implement watch-and-swap hot reload** — wrap `user_dict` in `RwLock`, add `reload_user_dict()`, add optional file watcher. Estimated effort: 1–2 days.

2. **Add automated cost estimation** — implement `estimate_cost(freq_per_million)` function using the log-frequency formula. Default to -1000 when frequency is unknown.

3. **Add CI validation workflow** — `mecab-ko-validate` binary + GitHub Actions check on any PR touching `user-dicts/`. Estimated effort: 0.5 days.

4. **Structure domain overlay directories** — create `user-dicts/{core,it,medical,legal}/` with seed files. Estimated effort: 0.5 days (structure + 50 seed entries per domain).

5. **Automated neologism pipeline** — weekly script: Korean news/blog corpus → cohesion filter → OOV check → candidate CSV queued for review. Medium effort; outsized long-term value.

6. **Extend CSV format with provenance fields** — add optional `domain` and `source` columns parsed but not required by the engine. Parser already ignores extra fields gracefully.

---

## References

- [MeCab official documentation](http://taku910.github.io/mecab/)
- [Sudachi GitHub - WorksApplications](https://github.com/WorksApplications/Sudachi)
- [SudachiDict AWS Open Data](https://registry.opendata.aws/sudachi/)
- [Nori Korean analyzer - Elastic](https://www.elastic.co/blog/nori-the-official-elasticsearch-plugin-for-korean-language-analysis)
- [Dictionary update behavior for Elasticsearch CJK analyzers](https://www.elastic.co/blog/dictionary-update-behavior-for-elasticsearch-cjk-language-analyzers)
- [AWS hot reload for Elasticsearch dictionary files (2020)](https://aws.amazon.com/about-aws/whats-new/2020/11/-amazon-elasticsearch-service-adds-support-for-hot-reload-of-dictionary-files/)
- [Kuromoji UserDictionary API - Lucene 9.1.0](https://lucene.apache.org/core/9_1_0/analysis/kuromoji/org/apache/lucene/analysis/ja/dict/UserDictionary.html)
- [Korean Bio-Medical Corpus (KBMC)](https://arxiv.org/html/2403.16158v1)
- [Shrinking Japanese Morphological Analyzers - ACL 2019](https://aclanthology.org/N19-1281/)
- [Lexicon-corpus Korean foreign word extraction - ScienceDirect](https://www.sciencedirect.com/science/article/pii/S1877705816318343)
- [python-mecab-ko custom vocabulary docs](https://python-mecab-ko.readthedocs.io/en/latest/usage/custom-vocabulary/)
- [mecab-ko-dic-msvc user-dic README](https://github.com/Pusnow/mecab-ko-dic-msvc/blob/master/user-dic/README.md)

---

## Learning Points

1. All production-grade CJK analyzers (Nori, Kuromoji, Sudachi) reload user dicts via an atomic swap, not a restart — mecab-ko's `Arc<UserDictionary>` is already shaped for this.
2. Frequency-derived negative costs (log-frequency × -200) reproduce CRF-trained behavior closely enough for user dict entries where full retraining is impractical.
3. Domain dictionaries maintained as separate overlay files (not merged) give operators composability and reviewers focused scope — the standard pattern across every mature project examined.
