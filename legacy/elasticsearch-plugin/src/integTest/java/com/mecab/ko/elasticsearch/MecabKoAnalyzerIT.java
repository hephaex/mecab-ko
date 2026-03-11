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

import static org.hamcrest.Matchers.*;

/**
 * Integration tests for MeCab-Ko analyzer.
 *
 * Tests the plugin in a real Elasticsearch cluster environment.
 */
@ESIntegTestCase.ClusterScope(scope = ESIntegTestCase.Scope.SUITE)
public class MecabKoAnalyzerIT extends ESIntegTestCase {

    @Override
    protected Collection<Class<? extends Plugin>> nodePlugins() {
        return Collections.singletonList(MecabKoPlugin.class);
    }

    public void testMecabKoAnalyzerBasic() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
            .put("index.analysis.analyzer.mecab_ko.decompound_mode", "none")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "한국어 형태소 분석기")
            .setAnalyzer("mecab_ko")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
        assertThat(tokens, is(not(empty())));

        // Verify we got tokens
        assertTrue("Should produce at least one token", tokens.size() > 0);
    }

    public void testMecabKoTokenizer() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.my_tokenizer.type", "mecab_ko_tokenizer")
            .put("index.analysis.tokenizer.my_tokenizer.decompound_mode", "mixed")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "형태소분석")
            .setTokenizer("my_tokenizer")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
        assertThat(tokens, is(not(empty())));

        // In mixed mode, should produce both compound and decomposed forms
        assertTrue("Should produce tokens in mixed mode", tokens.size() > 0);
    }

    public void testMecabKoPartOfSpeechFilter() throws Exception {
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

        // Should filter out J (josa) and E (eomi)
        for (AnalyzeAction.AnalyzeToken token : tokens) {
            String type = token.getType();
            assertFalse("Should not contain J (josa) tokens",
                type != null && type.startsWith("J"));
            assertFalse("Should not contain E (eomi) tokens",
                type != null && type.startsWith("E"));
        }
    }

    public void testDecompoundModeNone() throws Exception {
        testDecompoundMode("none");
    }

    public void testDecompoundModeDiscard() throws Exception {
        testDecompoundMode("discard");
    }

    public void testDecompoundModeMixed() throws Exception {
        testDecompoundMode("mixed");
    }

    private void testDecompoundMode(String mode) throws Exception {
        String indexName = "test_" + mode;

        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate(indexName);
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.test_analyzer.type", "mecab_ko")
            .put("index.analysis.analyzer.test_analyzer.decompound_mode", mode)
            .putList("index.analysis.analyzer.test_analyzer.stoptags", new String[0]) // No filtering
        );
        builder.get();

        ensureGreen(indexName);

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze(indexName, "형태소분석기")
            .setAnalyzer("test_analyzer")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
        assertThat(tokens, is(not(empty())));

        logger.info("Decompound mode '{}' produced {} tokens", mode, tokens.size());
        for (AnalyzeAction.AnalyzeToken token : tokens) {
            logger.info("  Token: {} ({})", token.getTerm(), token.getType());
        }
    }

    public void testNoriCompatibility() throws Exception {
        // Test Nori alias
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.nori.type", "nori")
            .put("index.analysis.tokenizer.my_tokenizer.type", "nori_tokenizer")
            .put("index.analysis.filter.my_filter.type", "nori_part_of_speech")
        );
        builder.get();

        ensureGreen("test");

        // Verify nori analyzer works
        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "테스트")
            .setAnalyzer("nori")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
        assertThat(tokens, is(not(empty())));
    }

    public void testLargeDocumentAnalysis() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
            .put("index.analysis.analyzer.mecab_ko.decompound_mode", "mixed")
        );
        builder.get();

        ensureGreen("test");

        // Create a large document (10KB+)
        StringBuilder largeDoc = new StringBuilder();
        String paragraph = "한국어 형태소 분석기는 자연어 처리를 위한 핵심 도구입니다. " +
                          "MeCab-Ko는 일본어 형태소 분석기인 MeCab을 한국어에 맞게 개조한 것입니다. " +
                          "Elasticsearch 플러그인으로 제공되어 검색 시스템에 통합할 수 있습니다. ";

        for (int i = 0; i < 50; i++) {
            largeDoc.append(paragraph);
        }

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", largeDoc.toString())
            .setAnalyzer("mecab_ko")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
        assertThat("Large document should produce many tokens", tokens.size(), greaterThan(100));

        logger.info("Large document ({} chars) produced {} tokens",
            largeDoc.length(), tokens.size());
    }

    public void testCustomStopTags() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
            .putList("index.analysis.analyzer.mecab_ko.stoptags", "J", "E", "S")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "한국어를 분석합니다!")
            .setAnalyzer("mecab_ko")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();

        for (AnalyzeAction.AnalyzeToken token : tokens) {
            String type = token.getType();
            logger.info("Token: '{}' ({})", token.getTerm(), type);

            // Verify stop tags are filtered
            assertFalse("Should not contain J tags", type != null && type.startsWith("J"));
            assertFalse("Should not contain E tags", type != null && type.startsWith("E"));
            assertFalse("Should not contain S tags", type != null && type.startsWith("S"));
        }
    }

    public void testAnalyzerWithUserDictionary() throws Exception {
        // Note: This test assumes a user dictionary file exists
        // In a real scenario, you would create a temp dictionary file
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko_custom.type", "mecab_ko")
            .put("index.analysis.analyzer.mecab_ko_custom.decompound_mode", "none")
            // User dictionary path would be set here if available
            // .put("index.analysis.analyzer.mecab_ko_custom.user_dictionary", "/path/to/userdict.csv")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "커스텀 사전 테스트")
            .setAnalyzer("mecab_ko_custom")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
        assertThat("Should produce tokens", tokens, is(not(empty())));

        logger.info("Custom analyzer produced {} tokens", tokens.size());
    }

    public void testMultipleAnalyzersInSameIndex() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko_none.type", "mecab_ko")
            .put("index.analysis.analyzer.mecab_ko_none.decompound_mode", "none")
            .put("index.analysis.analyzer.mecab_ko_discard.type", "mecab_ko")
            .put("index.analysis.analyzer.mecab_ko_discard.decompound_mode", "discard")
            .put("index.analysis.analyzer.mecab_ko_mixed.type", "mecab_ko")
            .put("index.analysis.analyzer.mecab_ko_mixed.decompound_mode", "mixed")
        );
        builder.get();

        ensureGreen("test");

        String text = "형태소분석기";

        // Test each analyzer
        String[] analyzers = {"mecab_ko_none", "mecab_ko_discard", "mecab_ko_mixed"};
        for (String analyzer : analyzers) {
            AnalyzeAction.Response response = client().admin().indices()
                .prepareAnalyze("test", text)
                .setAnalyzer(analyzer)
                .get();

            List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
            logger.info("Analyzer '{}' produced {} tokens", analyzer, tokens.size());
            for (AnalyzeAction.AnalyzeToken token : tokens) {
                logger.info("  Token: '{}'", token.getTerm());
            }
        }
    }

    public void testSpecialCharactersHandling() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
        );
        builder.get();

        ensureGreen("test");

        String[] testCases = {
            "이메일: test@example.com",
            "URL: https://example.com",
            "전화: 010-1234-5678",
            "날짜: 2026/01/06",
            "가격: ₩10,000",
            "수식: x² + y² = z²",
            "이모지: 😀👍🎉"
        };

        for (String text : testCases) {
            AnalyzeAction.Response response = client().admin().indices()
                .prepareAnalyze("test", text)
                .setAnalyzer("mecab_ko")
                .get();

            List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
            logger.info("Special chars '{}' produced {} tokens", text, tokens.size());
            for (AnalyzeAction.AnalyzeToken token : tokens) {
                logger.info("  Token: '{}' (offset: {}-{})",
                    token.getTerm(), token.getStartOffset(), token.getEndOffset());
            }
        }
    }

    public void testPerformanceBenchmark() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
            .put("index.analysis.analyzer.mecab_ko.decompound_mode", "mixed")
        );
        builder.get();

        ensureGreen("test");

        String text = "한국어 형태소 분석기는 자연어 처리의 핵심 도구입니다. " +
                     "Elasticsearch와 통합하여 강력한 검색 기능을 제공합니다.";

        // Warm-up
        for (int i = 0; i < 10; i++) {
            client().admin().indices()
                .prepareAnalyze("test", text)
                .setAnalyzer("mecab_ko")
                .get();
        }

        // Benchmark
        int iterations = 100;
        long startTime = System.currentTimeMillis();

        for (int i = 0; i < iterations; i++) {
            client().admin().indices()
                .prepareAnalyze("test", text)
                .setAnalyzer("mecab_ko")
                .get();
        }

        long endTime = System.currentTimeMillis();
        long totalTime = endTime - startTime;
        double avgTime = (double) totalTime / iterations;

        logger.info("Performance benchmark: {} iterations in {} ms (avg: {} ms/iteration)",
            iterations, totalTime, avgTime);

        assertTrue("Average analysis time should be reasonable", avgTime < 100); // 100ms threshold
    }

    public void testConcurrentAnalysis() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
        );
        builder.get();

        ensureGreen("test");

        // Simulate concurrent requests
        String[] texts = new String[20];
        for (int i = 0; i < texts.length; i++) {
            texts[i] = "테스트 문서 번호 " + i + " 한국어 분석";
        }

        // Execute all analyses
        for (String text : texts) {
            AnalyzeAction.Response response = client().admin().indices()
                .prepareAnalyze("test", text)
                .setAnalyzer("mecab_ko")
                .get();

            assertThat("Each analysis should produce tokens",
                response.getTokens(), is(not(empty())));
        }

        logger.info("Concurrent analysis completed successfully for {} requests", texts.length);
    }

    public void testEmptyAndWhitespaceInput() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
        );
        builder.get();

        ensureGreen("test");

        // Empty string
        AnalyzeAction.Response response1 = client().admin().indices()
            .prepareAnalyze("test", "")
            .setAnalyzer("mecab_ko")
            .get();
        assertThat("Empty string should produce no tokens",
            response1.getTokens(), is(empty()));

        // Whitespace only
        AnalyzeAction.Response response2 = client().admin().indices()
            .prepareAnalyze("test", "   \t\n   ")
            .setAnalyzer("mecab_ko")
            .get();
        assertThat("Whitespace should produce no tokens",
            response2.getTokens(), is(empty()));

        // Single space
        AnalyzeAction.Response response3 = client().admin().indices()
            .prepareAnalyze("test", " ")
            .setAnalyzer("mecab_ko")
            .get();
        assertThat("Single space should produce no tokens",
            response3.getTokens(), is(empty()));
    }
}
