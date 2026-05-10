# Dataset Expansion Research for mecab-ko Evaluation

> Researched 2026-05-11. Evaluation baseline: 99.9% sentence accuracy on 1,100 curated sentences — too easy. Goal: find diverse, noisy, real-world Korean text with Sejong-compatible POS gold annotations.

---

## Executive Summary

The two strongest candidates are **NIKL 모두의말뭉치 형태분석 (Modu Morphological Analysis Corpus)** and **KLUE DP (Dependency Parsing subset)**. Modu provides 371,571 training examples using Sejong-compatible POS tags (NNG, NNP, VV, VA, JKS, etc.), is academically credentialed, and represents diverse domains including news, web, and spoken text — but requires manual registration at corpus.korean.go.kr and has license restrictions that prevent direct redistribution in the repository. KLUE DP is on HuggingFace, has 12,000 sentences (10K train / 2K val) with Sejong-style POS tags already combined with morpheme boundaries (`NNG+JKO` format), CC BY-SA 4.0 licensed, and can be auto-downloaded in CI — making it the highest-value immediately actionable option. The killer caveat: neither source includes intentionally noisy text (typos, SNS slang, code-mixing). A complementary silver-labeling pipeline using C++ mecab-ko on unlabeled news/blog data would fill that gap.

---

## 1. 세종 코퍼스 (21st Century Sejong Project)

- **License:** Non-commercial, no redistribution. The license explicitly prohibits modification and redistribution even after error correction. Access is gated by the National Institute of Korean Language (국립국어원).
- **Access:** Request form required. Korean nationals can receive DVD or download credentials. Non-Korean researchers can make a written request but face higher friction. No public download URL. Informal mirrors exist on GitHub (e.g., `coolengineer/sejong-corpus`, `lovit/sejong_corpus`) — but redistributing these likely violates the license.
- **Format:** Proprietary XML-like format. Community tools exist to parse and convert (e.g., `lovit/sejong_corpus` Python package). Morphologically annotated text with 46 POS labels.
- **Size:** ~200 spoken + 279 written samples; approximately 11 million eojeols (syllable-block units) tagged with POS; ~175,000 sentences estimated for the morphologically annotated written portion.
- **Compatibility:** This is the origin of the Sejong POS tag scheme used by mecab-ko-dic. Tags are fully compatible: NNG, NNP, VV, VA, JKS, JKO, JKB, EC, EP, EF, etc. (46 tags total). mecab-ko-dic was trained directly from this corpus.
- **Verdict: CAUTION.** Legally, the no-redistribution clause rules out inclusion in the repo or auto-download in CI. Community mirrors exist but their use is gray-area. For internal benchmarking only, the corpus is ideal — if the team has access, it should be used. Do not include in public CI pipelines.

---

## 2. 모두의말뭉치 — NIKL Morphological Analysis Corpus (국립국어원)

- **License:** Academic use only, institutional authentication required. The Korpora Python package explicitly omits auto-download for this corpus due to license restrictions. Commercial use is not permitted. Redistribution is not allowed.
- **Access:** Manual registration at `https://corpus.korean.go.kr` (now redirects to `https://kli.korean.go.kr`). Login required; the download portal requires authentication. The site is in Korean only. Once downloaded, files are placed in `~/Korpora/NIKL_MP/` for the Korpora loader.
- **Format:** JSON-based with morphological segmentation and POS tags per eojeol. Korpora loader maps it to Python objects. Can be converted to TSV format compatible with mecab-ko's `text\t surface1/POS1 surface2/POS2 ...` format.
- **Size:** 371,571 examples (train split). Text spans multiple domains: news (문어), web text (웹), spoken language transcriptions (구어) — approximately 3 million eojeols (200만 written + 100만 spoken). This is the largest available Korean morphological gold corpus.
- **Compatibility:** Uses Sejong-compatible POS tags: NNG, NNP, NNB, VV, VA, JKS, JKO, JKB, JC, EP, EC, EF, XSN, SL, SN, SW, etc. Directly compatible with mecab-ko-dic scheme. The spoken sub-corpus includes informal register.
- **Verdict: GO (for internal benchmarking) / CAUTION (for CI).** Cannot be auto-downloaded in CI. Best path: developer downloads manually, runs a one-time conversion script, commits only the converted TSV to a `data/eval/` subdirectory (check if the license permits committing derivative formats — likely OK for the TSV if it's used purely internally and not redistirbuted). If this is a public repo, a `make download-eval-data` script with a license acknowledgement gate is the cleanest approach.

---

## 3. KAIST Corpora

### 3a. KAIST Morpho-Syntactically Annotated Corpus

- **License:** Academic use only, no redistribution.
- **Access:** Contact KAIST Semantic Web Research Center (`http://semanticweb.kaist.ac.kr/home/index.php/KAIST_Corpus`). The site is old and may require direct email inquiry. No public download.
- **Format:** Older proprietary format from 1999. Requires conversion tooling.
- **Size:** 70 million words. The largest Korean POS corpus by word count.
- **Compatibility:** Uses KAIST-specific POS tags, distinct from Sejong tags. Mapping exists but is not 1:1. Would require a non-trivial tag converter.
- **Verdict: NO-GO.** Access friction (email-based), old format, non-Sejong POS tags, no redistribution. Not suitable for automated eval pipelines.

### 3b. UD Korean-Kaist (Universal Dependencies)

- **License:** CC BY-SA 4.0. Fully open, redistributable.
- **Access:** GitHub `https://github.com/UniversalDependencies/UD_Korean-Kaist`. Part of UD 2.18 release. Auto-downloadable.
- **Format:** CoNLL-U (10-column tab-separated). Contains UPOS (17 universal tags: NOUN, VERB, ADJ, etc.) plus XPOS (language-specific). Morphological features in column 6.
- **Size:** 27,363 sentences / 350,090 tokens.
- **Compatibility:** Uses Universal POS (UPOS) tags, NOT Sejong tags. Conversion to Sejong is possible but lossy — UPOS NOUN maps to both NNG and NNP; VERB maps to VV and VA; particles and endings are merged under ADP/PART. A mechanical converter would produce noisy pseudo-Sejong labels.
- **Verdict: CAUTION.** Large and open, but requires a Sejong converter. The converted labels will not be gold-quality for mecab-ko evaluation without manual review. Useful as a corpus diversity source for silver labeling.

### 3c. UD Korean-GSD (Google)

- **License:** CC BY-SA 4.0.
- **Access:** GitHub `https://github.com/UniversalDependencies/UD_Korean-GSD`.
- **Format:** CoNLL-U.
- **Size:** 6,339 sentences / 80,322 tokens.
- **Compatibility:** Same UPOS issue as UD_Korean-Kaist.
- **Verdict: CAUTION.** Smaller than Kaist-UD but same structure and license. Not directly usable without Sejong tag converter.

---

## 4. HuggingFace Korean Datasets

### 4a. KLUE DP (Dependency Parsing)

- **License:** CC BY-SA 4.0. Freely downloadable, redistributable.
- **Access:** `https://huggingface.co/datasets/klue/klue`, config `dp`. Auto-downloadable via `datasets` library in Python or Rust `hf-hub` crate. No registration required.
- **Format:** JSON with fields `sentence`, `word_form` (list), `lemma` (list), `pos` (list), `head` (list), `deprel` (list). POS field contains Sejong-style combined tags: `"NNG"`, `"NNG+JKO"`, `"VV+EC"`, `"NNP+JKG"`, `"NNG+XSN+JKS"`, etc.
- **Size:** 12,000 sentence-level examples. Train: 10,000 / Val: 2,000.
- **Compatibility:** KLUE-DP uses mecab-ko POS tagging internally for annotation. POS tags are Sejong-style at the morpheme level, combined with `+` for eojeol-level representation. A splitter on `+` yields individual morpheme POS pairs directly compatible with mecab-ko-dic output format. Example: `"NNG+JKO"` → `[NNG, JKO]`.
- **Verdict: GO.** Best immediately actionable source. CC BY-SA 4.0, auto-downloadable, Sejong-compatible tags, 12K sentences of real editorial Korean text. The `+` splitting produces per-morpheme annotations matching mecab-ko output. Can be added to CI evaluation pipeline today.
- **Note on domain coverage:** Primarily Korean news/editorial text. Less noisy than SNS. Sentences tend to be well-formed, limiting exposure to informal register errors.

### 4b. KLUE NER (Named Entity Recognition)

- **License:** CC BY-SA 4.0.
- **Access:** `https://huggingface.co/datasets/klue/klue`, config `ner`.
- **Format:** Token-level NER tags but NO per-token POS tags. Sentence text + BIO NER labels only.
- **Size:** 24,000 sentences.
- **Compatibility:** No morphological gold. Cannot be directly used for morphological accuracy evaluation. Could be used to test NNP detection quality if we generate our own POS predictions and compare against NER boundaries.
- **Verdict: NO-GO** for morphological eval. Medium value for NNP boundary testing only.

### 4c. OpenKorPOS

- **License:** "All uses" (commercially available), redistribution allowed. Referenced as CC BY 4.0 in ACL Anthology publication context. License file in repo (`https://github.com/openkorpos/openkorpos`).
- **Access:** Must be built from source. Requires Python 3.9+, Click, Ninja. Run `python openkorpos.py ningen base && ninja`. Output is JSON Lines format. Source text is Korean Wikipedia.
- **Format:** JSON Lines with morpheme-level POS annotations. Uses Sejong-derived tags but the exact tagset needs verification from the paper (LREC 2022, Moon et al.). Corpus built by voting across multiple Korean morphological analyzers.
- **Size:** ~55 million words (Wikipedia). Sentence count not publicly specified; estimated 3–5 million sentences from Wikipedia scale.
- **Compatibility:** Likely Sejong-compatible (built using mecab-ko among other analyzers). The voting approach means tags are silver-quality (automatically generated), not gold-quality. This is a feature: it represents analyzer consensus, useful for testing edge cases where analyzers disagree.
- **Verdict: GO (with caveats).** Open license, buildable, large scale, Sejong-adjacent tags. Quality is silver (consensus annotation, not human-verified gold). Best used for bulk coverage testing rather than precision accuracy claims. Build pipeline required — not a simple download.

### 4d. KoBEST (Korean Balanced Evaluation of Significant Tasks)

- **License:** CC BY-SA 4.0.
- **Access:** `https://huggingface.co/datasets/skt/kobest_v1`.
- **Format:** Sentence classification tasks (BoolQ, COPA, HellaSwag, SentiNeg, WIC). No morphological gold annotations.
- **Verdict: NO-GO** for morphological evaluation.

---

## 5. Noisy / Real-World Korean Data

### 5a. Korean SNS / Social Media

No publicly available Korean SNS corpus with Sejong-compatible POS gold annotations was found. The landscape:

- **Korean Twitter (학술 연구용):** Twitter/X Academic API allows historical data access for research but provides raw text only, no POS annotations. Any annotations would need to be generated (silver-label pipeline).
- **KOR-SNS:** Referenced in some papers but no publicly accessible download found as of 2026. Appears to be an internal research corpus.
- **Naver Reviews (NSMC, NaverShopping):** Sentiment corpora (e.g., `e9t/nsmc` on GitHub, 200K sentences) exist with sentiment labels, not POS gold. Raw text could be silver-labeled.
- **Conclusion:** No off-the-shelf noisy Korean POS corpus exists publicly. Must be created via silver-labeling.

### 5b. Silver Labeling with C++ mecab-ko

The most practical path to noisy-text coverage:

1. Take any raw Korean text (news, blog, SNS, Wikipedia).
2. Run through the original C++ `mecab-ko` (the reference implementation).
3. Treat output as "silver gold" — correct for well-formed text, potentially wrong for edge cases.
4. Use divergence between Rust mecab-ko and C++ mecab-ko output as error signal (instead of comparing against human gold).

**Validity:** The OpenKorPOS paper (LREC 2022, Moon et al.) validates this voting/consensus approach. In the Korean NLP community, using existing high-accuracy analyzers to bootstrap annotation is accepted for large-scale evaluation, with the caveat that analyzer-specific errors are shared.

**Recommended sources for silver labeling:**
- Korean Wikipedia dump (freely downloadable, `dumps.wikimedia.org/kowiki/`)
- Common Crawl Korean subset (CC-100 Korean, ~54GB, freely available)
- AI Hub web corpus (1 billion tokens, news — requires Korean national registration)

### 5c. Learner Korean / L2 Data

- **KSL (Korean as Second Language) Learner Corpus:** Referenced in ACL 2023 (BEA workshop). Contains learner-produced Korean with errors. POS annotations may exist. License unclear.
- **Verdict:** Low priority for mecab-ko evaluation; L2 errors are orthographically distinct from the informal native-speaker errors we want to test.

---

## Comparison Table

| Source | License | Sentences | Format | Sejong Compat | CI Auto-DL | Effort | Verdict |
|--------|---------|-----------|--------|--------------|------------|--------|---------|
| 세종 코퍼스 | No redistrib | ~175K | Proprietary XML | Native (origin) | No | High (request) | CAUTION |
| NIKL Modu 형태분석 | Academic only | ~371K | JSON | Native | No | Medium (register) | GO (internal) |
| KAIST Morpho | Academic only | ~70M words | Proprietary | KAIST-specific | No | Very High | NO-GO |
| UD Korean-Kaist | CC BY-SA 4.0 | 27,363 | CoNLL-U | Convert needed | Yes | Medium | CAUTION |
| UD Korean-GSD | CC BY-SA 4.0 | 6,339 | CoNLL-U | Convert needed | Yes | Medium | CAUTION |
| KLUE DP | CC BY-SA 4.0 | 12,000 | JSON | Near-native (+split) | Yes | Low | GO |
| KLUE NER | CC BY-SA 4.0 | 24,000 | JSON | None (NER only) | Yes | N/A | NO-GO (morph) |
| OpenKorPOS | Open / CC BY 4.0 | ~3-5M est. | JSON Lines | Likely compatible | Build-from-src | High (build) | GO (silver) |
| AI Hub web corpus | Restricted (KR nationals) | ~65K files | JSON | Named entity only | No | Very High | NO-GO |
| Silver (kowiki + C++ mecab) | CC BY-SA 4.0 (text) | Unlimited | TSV (generated) | Native | Yes (pipeline) | Medium | GO |

---

## Recommended Path Forward

### Option A — Quick Win: KLUE DP (Sprint 124, 1 sprint effort)

**What:** Add KLUE DP as a second evaluation target, downloadable via HuggingFace `datasets` Python library or direct HTTP. Write a converter in Rust that splits `NNG+JKO` → `[NNG, JKO]` at eojeol boundaries and produces our TSV format.

**Effort estimate:** 3–5 days.
- Day 1: Write Python converter script `tools/convert_klue_dp.py` to fetch and convert KLUE DP to `data/eval/klue_dp_val.tsv` (2,000 sentences).
- Day 2: Verify tag compatibility (check all KLUE POS tags against mecab-ko-dic scheme; expected ~98% direct match).
- Day 3: Integrate into `cargo test` eval harness as a second eval dataset.
- Day 4: Run evaluation, analyze errors, document results.
- Day 5: Add CI step for KLUE DP download + eval.

**Verdict:** This is the single best Sprint 124 recommendation. KLUE DP is CC BY-SA 4.0, auto-downloadable, Sejong-compatible, 12K sentences, and immediately usable. The `+` delimiter for morpheme concatenation is easy to split. Expected to surface new error categories (NNP boundary errors, verbal endings, etc.) not present in our curated 1,100-sentence set.

**Caveat:** 12,000 sentences is 10x our current dataset but primarily editorial Korean. Does not cover informal register.

---

### Option B — Medium Win: NIKL Modu Manual Download (Sprint 125–126, 2 sprint effort)

**What:** Developer registers at `https://kli.korean.go.kr`, downloads the NIKL_MP corpus, and runs a local conversion script to produce `data/eval/nikl_mp_sample.tsv` (sample of 5,000–10,000 sentences). The full dataset stays off-repo due to license; only the evaluation results are committed.

**Effort estimate:** 1 week total, mostly blocked on registration approval time (can be 1–5 business days).
- Day 1: Register at kli.korean.go.kr, initiate download request.
- Day 3–5 (after approval): Download corpus, write `tools/convert_nikl_mp.py`.
- Day 6–7: Convert, curate 5,000-sentence sample covering diverse domains.
- Day 8: Run evaluation, document domain breakdown (news vs. web vs. spoken).

**Verdict:** Best for comprehensive coverage. 371K examples, multi-domain, Sejong-native. The spoken subcorpus (100만 eojeol) is the closest to informal Korean available in a gold corpus. Primary obstacle is licensing — cannot auto-download in CI, cannot commit corpus to repo.

**Workflow for CI:** Commit a `SHA256` checksum file alongside the eval script. CI checks for corpus presence; if absent, prints instructions and skips (rather than failing). Optional: gate the eval behind a `NIKL_MP_AVAILABLE=1` env variable.

---

### Option C — Long-Term: Silver Label Pipeline on Korean Wikipedia (Sprint 126–128, 3–4 sprint effort)

**What:** Build an automated pipeline that:
1. Downloads Korean Wikipedia dump (`https://dumps.wikimedia.org/kowiki/latest/kowiki-latest-articles.xml.bz2`).
2. Extracts plain text (using WikiExtractor or similar).
3. Runs C++ `mecab-ko` (the reference implementation) to generate silver POS labels.
4. Samples 50,000 sentences across topic categories (science, sports, entertainment, political, historical).
5. Saves as `data/eval/kowiki_silver_50k.tsv`.

**Effort estimate:** 2–3 weeks.
- Week 1: Build extraction + annotation pipeline (Docker image with C++ mecab-ko).
- Week 2: Sample, deduplicate, domain-balance 50,000 sentences.
- Week 3: Run Rust mecab-ko vs. C++ mecab-ko diff analysis. Document divergence patterns.

**Verdict:** Ideal for large-scale regression testing and finding edge cases. The key insight is we measure Rust-vs-C++ fidelity, not Rust-vs-gold — which is actually the correct evaluation goal for a reimplementation. 50K sentences from Wikipedia will expose compound noun boundary errors, low-frequency vocabulary failures, and POS ambiguity in ways our 1,100-sentence set cannot.

**License note:** Korean Wikipedia text is CC BY-SA 4.0. C++ mecab-ko annotations are our own derived work. No license issues for committing the generated TSV.

---

## Open Questions for User Decision

1. **Redistribution policy:** Is this a public repo? If so, can we commit KLUE DP converted TSV directly (CC BY-SA 4.0 attribution required)? Or should the eval data live in a separate private data repo?

2. **NIKL registration:** Is the team willing to go through the NIKL/모두의말뭉치 registration process? Requires institutional affiliation (university or company). If registering as individuals, approval may be slower.

3. **C++ mecab-ko availability:** Is the original C++ mecab-ko binary available in the CI environment for silver labeling? The Docker image for `eunjeon/mecab-ko` exists but is older. Alternatively, `mecab-ko-dic` + `mecab` compiled with Korean support would suffice.

4. **Informal/noisy text priority:** How important is SNS/informal text coverage in the near term? If NNP accuracy in news text is the priority, KLUE DP covers it. If social media robustness is the priority, a custom silver-label pipeline on Naver blog dumps or NSMC raw text is needed.

5. **Evaluation metric design:** When using KLUE DP (`NNG+JKO` format), should we evaluate at eojeol level (whole eojeol POS correct) or morpheme level (each morpheme within eojeol correct)? The morpheme-level metric is more informative but requires adjusting the current evaluation harness.

6. **Tag edge cases in KLUE DP:** KLUE uses `NNG+XSN+JKS` compound tags (3+ morphemes per eojeol). Current mecab-ko eval format uses space-separated `surface/POS` pairs. A conversion decision is needed: split compounds per morpheme (more granular) vs. treat as single eojeol unit (preserves KLUE ground truth).

---

## References

- KLUE Benchmark Paper (NeurIPS 2021 Datasets & Benchmarks): https://datasets-benchmarks-proceedings.neurips.cc/paper/2021/hash/98dce83da57b0395e163467c9dae521b-Abstract-round2.html
- KLUE HuggingFace Dataset: https://huggingface.co/datasets/klue/klue
- OpenKorPOS (LREC 2022, Moon et al.): https://aclanthology.org/2022.lrec-1.531/
- OpenKorPOS GitHub: https://github.com/openkorpos/openkorpos
- Open Korean Corpora (ko-nlp): https://github.com/ko-nlp/Open-korean-corpora
- NIKL 모두의말뭉치 Korpora docs: https://ko-nlp.github.io/Korpora/ko-docs/corpuslist/modu_mp.html
- 모두의말뭉치 portal: https://kli.korean.go.kr
- UD Korean-Kaist (CC BY-SA 4.0): https://github.com/UniversalDependencies/UD_Korean-Kaist
- UD Korean-GSD (CC BY-SA 4.0): https://github.com/UniversalDependencies/UD_Korean-GSD
- Sejong Corpus overview (GitHub): https://github.com/coolengineer/sejong-corpus
- KAIST Corpus: http://semanticweb.kaist.ac.kr/home/index.php/KAIST_Corpus
- Korpora Korean corpus repository: https://github.com/ko-nlp/Korpora
- AwesomeKorean_Data: https://github.com/songys/AwesomeKorean_Data
- Korean Wikipedia dumps: https://dumps.wikimedia.org/kowiki/

---

## Learning Points

1. **The Sejong POS tag fragmentation problem:** Every Korean morphological corpus uses a slightly different incarnation of Sejong tags. The "native" version in the raw Sejong corpus uses 46 tags; mecab-ko-dic uses a subset; KLUE DP uses Sejong tags but combines them per eojeol with `+`. Never assume two "Sejong-compatible" datasets are format-identical — always inspect 20–30 examples before committing to a converter design.

2. **No freely redistributable Korean gold morphological corpus exists at scale.** NIKL Modu is the best gold corpus but requires manual registration and cannot be auto-downloaded. KLUE DP (12K sentences, CC BY-SA 4.0) is the only freely auto-downloadable Sejong-compatible gold dataset. Everything larger requires either gated access or silver labeling.

3. **For a reimplementation, Rust-vs-C++ divergence is a valid evaluation signal.** Rather than chasing an unobtainable gold standard, testing Rust mecab-ko against C++ mecab-ko output on large, diverse corpora (Wikipedia, news) exposes real bugs without requiring human annotation. This is the approach to use for Option C.

---

*Researched: 2026-05-11*
*Sprint context: Sprint 124 planning — accuracy evaluation expansion*
