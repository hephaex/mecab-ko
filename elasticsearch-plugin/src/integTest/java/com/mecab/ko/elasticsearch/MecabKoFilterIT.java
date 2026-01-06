package com.mecab.ko.elasticsearch;

import org.elasticsearch.action.admin.indices.analyze.AnalyzeAction;
import org.elasticsearch.action.admin.indices.create.CreateIndexRequestBuilder;
import org.elasticsearch.common.settings.Settings;
import org.elasticsearch.plugins.Plugin;
import org.elasticsearch.test.ESIntegTestCase;
import com.mecab.ko.elasticsearch.plugin.MecabKoPlugin;

import java.util.Collection;
import java.util.Collections;
import java.util.List;
import java.util.stream.Collectors;

import static org.hamcrest.Matchers.*;

/**
 * Integration tests for MeCab-Ko token filters.
 *
 * Tests Part-of-Speech filtering and other token filter functionality.
 */
@ESIntegTestCase.ClusterScope(scope = ESIntegTestCase.Scope.SUITE)
public class MecabKoFilterIT extends ESIntegTestCase {

    @Override
    protected Collection<Class<? extends Plugin>> nodePlugins() {
        return Collections.singletonList(MecabKoPlugin.class);
    }

    public void testPartOfSpeechStopFilterBasic() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.my_tokenizer.type", "mecab_ko_tokenizer")
            .put("index.analysis.filter.my_pos_filter.type", "mecab_ko_part_of_speech")
            .putList("index.analysis.filter.my_pos_filter.stoptags", "J", "E")
            .put("index.analysis.analyzer.my_analyzer.type", "custom")
            .put("index.analysis.analyzer.my_analyzer.tokenizer", "my_tokenizer")
            .putList("index.analysis.analyzer.my_analyzer.filter", "my_pos_filter")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "한국어를 분석한다")
            .setAnalyzer("my_analyzer")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();

        logger.info("POS filter produced {} tokens", tokens.size());
        for (AnalyzeAction.AnalyzeToken token : tokens) {
            String type = token.getType();
            logger.info("  Token: '{}' ({})", token.getTerm(), type);

            // Verify J (josa/particles) and E (eomi/verb endings) are filtered
            assertFalse("Should not contain J (josa) tokens",
                type != null && type.startsWith("J"));
            assertFalse("Should not contain E (eomi) tokens",
                type != null && type.startsWith("E"));
        }
    }

    public void testPartOfSpeechFilterNounsOnly() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.my_tokenizer.type", "mecab_ko_tokenizer")
            .put("index.analysis.filter.my_pos_filter.type", "mecab_ko_part_of_speech")
            // Filter everything except nouns (NNG, NNP, NNB, NR, NP)
            .putList("index.analysis.filter.my_pos_filter.stoptags",
                "E", "J", "M", "V", "X", "S", "SF", "SP", "SS", "SE", "SO", "SW")
            .put("index.analysis.analyzer.my_analyzer.type", "custom")
            .put("index.analysis.analyzer.my_analyzer.tokenizer", "my_tokenizer")
            .putList("index.analysis.analyzer.my_analyzer.filter", "my_pos_filter")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "한국어 형태소 분석기를 사용합니다")
            .setAnalyzer("my_analyzer")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();

        logger.info("Nouns-only filter produced {} tokens", tokens.size());
        for (AnalyzeAction.AnalyzeToken token : tokens) {
            String type = token.getType();
            logger.info("  Token: '{}' ({})", token.getTerm(), type);

            // All tokens should be nouns (N prefix)
            assertTrue("All tokens should be nouns (N prefix), got: " + type,
                type == null || type.startsWith("N") || type.equals("UNKNOWN"));
        }
    }

    public void testPartOfSpeechFilterVerbsAndAdjectives() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.my_tokenizer.type", "mecab_ko_tokenizer")
            .put("index.analysis.filter.my_pos_filter.type", "mecab_ko_part_of_speech")
            // Filter everything except verbs (V) and adjectives (M)
            .putList("index.analysis.filter.my_pos_filter.stoptags",
                "N", "E", "J", "X", "S", "SF", "SP", "SS", "SE", "SO", "SW")
            .put("index.analysis.analyzer.my_analyzer.type", "custom")
            .put("index.analysis.analyzer.my_analyzer.tokenizer", "my_tokenizer")
            .putList("index.analysis.analyzer.my_analyzer.filter", "my_pos_filter")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "빠르게 달리는 아름다운 말")
            .setAnalyzer("my_analyzer")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();

        logger.info("Verbs/Adjectives filter produced {} tokens", tokens.size());
        for (AnalyzeAction.AnalyzeToken token : tokens) {
            String type = token.getType();
            logger.info("  Token: '{}' ({})", token.getTerm(), type);

            // Should only contain V (verbs) or M (adjectives)
            assertTrue("Should only contain V or M tags, got: " + type,
                type == null || type.startsWith("V") || type.startsWith("M") || type.equals("UNKNOWN"));
        }
    }

    public void testPartOfSpeechFilterEmptyStopTags() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.my_tokenizer.type", "mecab_ko_tokenizer")
            .put("index.analysis.filter.my_pos_filter.type", "mecab_ko_part_of_speech")
            .putList("index.analysis.filter.my_pos_filter.stoptags", new String[0])
            .put("index.analysis.analyzer.my_analyzer.type", "custom")
            .put("index.analysis.analyzer.my_analyzer.tokenizer", "my_tokenizer")
            .putList("index.analysis.analyzer.my_analyzer.filter", "my_pos_filter")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "한국어 분석")
            .setAnalyzer("my_analyzer")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();

        // With no stop tags, all tokens should pass through
        assertThat("Should produce tokens", tokens, is(not(empty())));

        logger.info("No-filter produced {} tokens", tokens.size());
        for (AnalyzeAction.AnalyzeToken token : tokens) {
            logger.info("  Token: '{}' ({})", token.getTerm(), token.getType());
        }
    }

    public void testMultipleFiltersChained() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.my_tokenizer.type", "mecab_ko_tokenizer")
            .put("index.analysis.tokenizer.my_tokenizer.decompound_mode", "mixed")
            .put("index.analysis.filter.my_pos_filter.type", "mecab_ko_part_of_speech")
            .putList("index.analysis.filter.my_pos_filter.stoptags", "J", "E")
            .put("index.analysis.filter.lowercase.type", "lowercase")
            .put("index.analysis.analyzer.my_analyzer.type", "custom")
            .put("index.analysis.analyzer.my_analyzer.tokenizer", "my_tokenizer")
            .putList("index.analysis.analyzer.my_analyzer.filter", "my_pos_filter", "lowercase")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "ElasticSearch 검색엔진")
            .setAnalyzer("my_analyzer")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();

        logger.info("Chained filters produced {} tokens", tokens.size());
        for (AnalyzeAction.AnalyzeToken token : tokens) {
            logger.info("  Token: '{}' ({})", token.getTerm(), token.getType());

            // Verify POS filtering
            String type = token.getType();
            assertFalse("Should not contain J tokens", type != null && type.startsWith("J"));
            assertFalse("Should not contain E tokens", type != null && type.startsWith("E"));

            // Verify lowercase is applied to ASCII characters
            String term = token.getTerm();
            if (term.matches("[A-Za-z]+")) {
                assertEquals("ASCII should be lowercased", term.toLowerCase(), term);
            }
        }
    }

    public void testFilterWithDecompoundModes() throws Exception {
        String[] modes = {"none", "discard", "mixed"};

        for (String mode : modes) {
            String indexName = "test_" + mode;

            CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate(indexName);
            builder.setSettings(Settings.builder()
                .put("index.analysis.tokenizer.my_tokenizer.type", "mecab_ko_tokenizer")
                .put("index.analysis.tokenizer.my_tokenizer.decompound_mode", mode)
                .put("index.analysis.filter.my_pos_filter.type", "mecab_ko_part_of_speech")
                .putList("index.analysis.filter.my_pos_filter.stoptags", "J", "E")
                .put("index.analysis.analyzer.my_analyzer.type", "custom")
                .put("index.analysis.analyzer.my_analyzer.tokenizer", "my_tokenizer")
                .putList("index.analysis.analyzer.my_analyzer.filter", "my_pos_filter")
            );
            builder.get();

            ensureGreen(indexName);

            AnalyzeAction.Response response = client().admin().indices()
                .prepareAnalyze(indexName, "형태소분석기를 사용한다")
                .setAnalyzer("my_analyzer")
                .get();

            List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();

            logger.info("Filter with decompound mode '{}' produced {} tokens", mode, tokens.size());
            for (AnalyzeAction.AnalyzeToken token : tokens) {
                logger.info("  Token: '{}' ({})", token.getTerm(), token.getType());
            }
        }
    }

    public void testFilterPreservesOffsets() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.my_tokenizer.type", "mecab_ko_tokenizer")
            .put("index.analysis.filter.my_pos_filter.type", "mecab_ko_part_of_speech")
            .putList("index.analysis.filter.my_pos_filter.stoptags", "J")
            .put("index.analysis.analyzer.my_analyzer.type", "custom")
            .put("index.analysis.analyzer.my_analyzer.tokenizer", "my_tokenizer")
            .putList("index.analysis.analyzer.my_analyzer.filter", "my_pos_filter")
        );
        builder.get();

        ensureGreen("test");

        String text = "한국어를 분석";
        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", text)
            .setAnalyzer("my_analyzer")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();

        for (AnalyzeAction.AnalyzeToken token : tokens) {
            int start = token.getStartOffset();
            int end = token.getEndOffset();

            // Verify offsets are valid
            assertThat("Start offset should be non-negative", start, greaterThanOrEqualTo(0));
            assertThat("End offset should not exceed text length", end, lessThanOrEqualTo(text.length()));
            assertThat("End should be after start", end, greaterThanOrEqualTo(start));

            logger.info("Token '{}': offset {}-{}", token.getTerm(), start, end);
        }
    }

    public void testReadingFormFilter() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.my_tokenizer.type", "mecab_ko_tokenizer")
            .put("index.analysis.filter.my_reading_filter.type", "mecab_ko_reading_form")
            .put("index.analysis.analyzer.my_analyzer.type", "custom")
            .put("index.analysis.analyzer.my_analyzer.tokenizer", "my_tokenizer")
            .putList("index.analysis.analyzer.my_analyzer.filter", "my_reading_filter")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "한국어 형태소 분석")
            .setAnalyzer("my_analyzer")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();

        logger.info("Reading form filter produced {} tokens", tokens.size());
        for (AnalyzeAction.AnalyzeToken token : tokens) {
            logger.info("  Token: '{}' ({})", token.getTerm(), token.getType());
        }

        // Reading form filter should produce tokens (exact behavior depends on implementation)
        assertThat("Should produce tokens", tokens, is(not(empty())));
    }

    public void testFilterWithSpecialCharacters() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.my_tokenizer.type", "mecab_ko_tokenizer")
            .put("index.analysis.filter.my_pos_filter.type", "mecab_ko_part_of_speech")
            .putList("index.analysis.filter.my_pos_filter.stoptags", "S", "SF", "SP", "SS", "SE", "SO", "SW")
            .put("index.analysis.analyzer.my_analyzer.type", "custom")
            .put("index.analysis.analyzer.my_analyzer.tokenizer", "my_tokenizer")
            .putList("index.analysis.analyzer.my_analyzer.filter", "my_pos_filter")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "안녕하세요! 한국어, 분석? (테스트)")
            .setAnalyzer("my_analyzer")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();

        logger.info("Filter with special chars produced {} tokens", tokens.size());
        for (AnalyzeAction.AnalyzeToken token : tokens) {
            String type = token.getType();
            logger.info("  Token: '{}' ({})", token.getTerm(), type);

            // Should filter out symbols (S*)
            assertFalse("Should not contain S (symbol) tokens",
                type != null && type.startsWith("S"));
        }
    }

    public void testFilterPerformance() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.my_tokenizer.type", "mecab_ko_tokenizer")
            .put("index.analysis.filter.my_pos_filter.type", "mecab_ko_part_of_speech")
            .putList("index.analysis.filter.my_pos_filter.stoptags", "J", "E")
            .put("index.analysis.analyzer.my_analyzer.type", "custom")
            .put("index.analysis.analyzer.my_analyzer.tokenizer", "my_tokenizer")
            .putList("index.analysis.analyzer.my_analyzer.filter", "my_pos_filter")
        );
        builder.get();

        ensureGreen("test");

        // Create a moderately long text
        StringBuilder text = new StringBuilder();
        for (int i = 0; i < 50; i++) {
            text.append("한국어 형태소 분석기를 사용하여 텍스트를 분석합니다. ");
        }

        long startTime = System.currentTimeMillis();

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", text.toString())
            .setAnalyzer("my_analyzer")
            .get();

        long endTime = System.currentTimeMillis();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();

        logger.info("Filter performance test: {} chars, {} tokens, {} ms",
            text.length(), tokens.size(), (endTime - startTime));

        assertThat("Should produce tokens", tokens, is(not(empty())));
    }
}
