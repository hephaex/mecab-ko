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
 * Integration tests for MeCab-Ko tokenizer.
 *
 * Tests tokenization behavior in various scenarios.
 */
@ESIntegTestCase.ClusterScope(scope = ESIntegTestCase.Scope.SUITE)
public class MecabKoTokenizerIT extends ESIntegTestCase {

    @Override
    protected Collection<Class<? extends Plugin>> nodePlugins() {
        return Collections.singletonList(MecabKoPlugin.class);
    }

    public void testBasicTokenization() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.mecab_ko.type", "mecab_ko_tokenizer")
            .put("index.analysis.tokenizer.mecab_ko.decompound_mode", "none")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "한국어 형태소 분석기")
            .setTokenizer("mecab_ko")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
        assertThat(tokens, is(not(empty())));

        logger.info("Basic tokenization produced {} tokens", tokens.size());
        for (AnalyzeAction.AnalyzeToken token : tokens) {
            logger.info("  Token: '{}' (type: {}, position: {})",
                token.getTerm(), token.getType(), token.getPosition());
        }
    }

    public void testDecompoundModeNone() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.mecab_ko.type", "mecab_ko_tokenizer")
            .put("index.analysis.tokenizer.mecab_ko.decompound_mode", "none")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "형태소분석기")
            .setTokenizer("mecab_ko")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
        assertThat(tokens, is(not(empty())));

        // In NONE mode, compound nouns should be kept as-is
        logger.info("NONE mode produced {} tokens", tokens.size());
        for (AnalyzeAction.AnalyzeToken token : tokens) {
            logger.info("  Token: '{}' ({})", token.getTerm(), token.getType());
        }
    }

    public void testDecompoundModeDiscard() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.mecab_ko.type", "mecab_ko_tokenizer")
            .put("index.analysis.tokenizer.mecab_ko.decompound_mode", "discard")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "형태소분석기")
            .setTokenizer("mecab_ko")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
        assertThat(tokens, is(not(empty())));

        // In DISCARD mode, only decomposed morphemes should appear
        logger.info("DISCARD mode produced {} tokens", tokens.size());
        for (AnalyzeAction.AnalyzeToken token : tokens) {
            logger.info("  Token: '{}' ({})", token.getTerm(), token.getType());
        }
    }

    public void testDecompoundModeMixed() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.mecab_ko.type", "mecab_ko_tokenizer")
            .put("index.analysis.tokenizer.mecab_ko.decompound_mode", "mixed")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "형태소분석기")
            .setTokenizer("mecab_ko")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
        assertThat(tokens, is(not(empty())));

        // In MIXED mode, both compound and decomposed forms should appear
        logger.info("MIXED mode produced {} tokens", tokens.size());
        for (AnalyzeAction.AnalyzeToken token : tokens) {
            logger.info("  Token: '{}' ({})", token.getTerm(), token.getType());
        }
    }

    public void testMixedKoreanEnglishText() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.mecab_ko.type", "mecab_ko_tokenizer")
            .put("index.analysis.tokenizer.mecab_ko.decompound_mode", "mixed")
        );
        builder.get();

        ensureGreen("test");

        String[] testTexts = {
            "Elasticsearch 한글 플러그인",
            "MeCab-Ko는 한국어 형태소 분석기입니다",
            "Java와 Python을 사용합니다",
            "REST API 호출"
        };

        for (String text : testTexts) {
            AnalyzeAction.Response response = client().admin().indices()
                .prepareAnalyze("test", text)
                .setTokenizer("mecab_ko")
                .get();

            List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
            assertThat("Should produce tokens for: " + text, tokens, is(not(empty())));

            logger.info("Mixed text '{}' produced {} tokens", text, tokens.size());
            for (AnalyzeAction.AnalyzeToken token : tokens) {
                logger.info("  Token: '{}' ({})", token.getTerm(), token.getType());
            }
        }
    }

    public void testSpecialCharacters() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.mecab_ko.type", "mecab_ko_tokenizer")
            .put("index.analysis.tokenizer.mecab_ko.decompound_mode", "none")
        );
        builder.get();

        ensureGreen("test");

        String[] testTexts = {
            "안녕하세요!",
            "이메일: test@example.com",
            "가격: 10,000원",
            "날짜: 2026-01-06",
            "URL: https://example.com/path",
            "괄호(안의 내용)과 대괄호[내용]",
            "특수문자 #$%^&* 포함"
        };

        for (String text : testTexts) {
            AnalyzeAction.Response response = client().admin().indices()
                .prepareAnalyze("test", text)
                .setTokenizer("mecab_ko")
                .get();

            List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
            assertThat("Should handle special characters in: " + text, tokens, is(not(empty())));

            logger.info("Special characters '{}' produced {} tokens", text, tokens.size());
            for (AnalyzeAction.AnalyzeToken token : tokens) {
                logger.info("  Token: '{}' (type: {}, offset: {}-{})",
                    token.getTerm(), token.getType(),
                    token.getStartOffset(), token.getEndOffset());
            }
        }
    }

    public void testEmptyAndWhitespace() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.mecab_ko.type", "mecab_ko_tokenizer")
        );
        builder.get();

        ensureGreen("test");

        // Empty string
        AnalyzeAction.Response response1 = client().admin().indices()
            .prepareAnalyze("test", "")
            .setTokenizer("mecab_ko")
            .get();
        assertThat("Empty string should produce no tokens",
            response1.getTokens(), is(empty()));

        // Only whitespace
        AnalyzeAction.Response response2 = client().admin().indices()
            .prepareAnalyze("test", "   \t\n  ")
            .setTokenizer("mecab_ko")
            .get();
        assertThat("Whitespace-only should produce no tokens",
            response2.getTokens(), is(empty()));
    }

    public void testLongText() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.mecab_ko.type", "mecab_ko_tokenizer")
            .put("index.analysis.tokenizer.mecab_ko.decompound_mode", "mixed")
        );
        builder.get();

        ensureGreen("test");

        // Create a longer text
        StringBuilder longText = new StringBuilder();
        for (int i = 0; i < 100; i++) {
            longText.append("한국어 형태소 분석기는 자연어 처리의 기본이 되는 도구입니다. ");
        }

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", longText.toString())
            .setTokenizer("mecab_ko")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
        assertThat("Should handle long text", tokens, is(not(empty())));

        logger.info("Long text ({} chars) produced {} tokens",
            longText.length(), tokens.size());
    }

    public void testOffsetAccuracy() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.mecab_ko.type", "mecab_ko_tokenizer")
            .put("index.analysis.tokenizer.mecab_ko.decompound_mode", "none")
        );
        builder.get();

        ensureGreen("test");

        String text = "한글 테스트 문장";
        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", text)
            .setTokenizer("mecab_ko")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();

        for (AnalyzeAction.AnalyzeToken token : tokens) {
            int start = token.getStartOffset();
            int end = token.getEndOffset();

            // Verify offsets are within text bounds
            assertThat("Start offset should be non-negative", start, greaterThanOrEqualTo(0));
            assertThat("End offset should not exceed text length", end, lessThanOrEqualTo(text.length()));
            assertThat("End should be after start", end, greaterThanOrEqualTo(start));

            logger.info("Token '{}': offset {}-{}, substring: '{}'",
                token.getTerm(), start, end, text.substring(start, end));
        }
    }

    public void testOutputUnknownUnigrams() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.mecab_ko_default.type", "mecab_ko_tokenizer")
            .put("index.analysis.tokenizer.mecab_ko_default.output_unknown_unigrams", false)
            .put("index.analysis.tokenizer.mecab_ko_unigrams.type", "mecab_ko_tokenizer")
            .put("index.analysis.tokenizer.mecab_ko_unigrams.output_unknown_unigrams", true)
        );
        builder.get();

        ensureGreen("test");

        // Test with text that may contain unknown words
        String text = "한글test未知語";

        AnalyzeAction.Response response1 = client().admin().indices()
            .prepareAnalyze("test", text)
            .setTokenizer("mecab_ko_default")
            .get();

        AnalyzeAction.Response response2 = client().admin().indices()
            .prepareAnalyze("test", text)
            .setTokenizer("mecab_ko_unigrams")
            .get();

        logger.info("Without unigrams: {} tokens", response1.getTokens().size());
        for (AnalyzeAction.AnalyzeToken token : response1.getTokens()) {
            logger.info("  Token: '{}' ({})", token.getTerm(), token.getType());
        }

        logger.info("With unigrams: {} tokens", response2.getTokens().size());
        for (AnalyzeAction.AnalyzeToken token : response2.getTokens()) {
            logger.info("  Token: '{}' ({})", token.getTerm(), token.getType());
        }
    }

    public void testPositionIncrements() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.mecab_ko.type", "mecab_ko_tokenizer")
            .put("index.analysis.tokenizer.mecab_ko.decompound_mode", "mixed")
        );
        builder.get();

        ensureGreen("test");

        AnalyzeAction.Response response = client().admin().indices()
            .prepareAnalyze("test", "형태소분석")
            .setTokenizer("mecab_ko")
            .get();

        List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();

        int lastPosition = -1;
        for (AnalyzeAction.AnalyzeToken token : tokens) {
            int position = token.getPosition();
            assertThat("Position should advance", position, greaterThanOrEqualTo(lastPosition));
            lastPosition = position;

            logger.info("Token '{}': position {}, type {}",
                token.getTerm(), position, token.getType());
        }
    }

    public void testConcurrentAnalysis() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.tokenizer.mecab_ko.type", "mecab_ko_tokenizer")
            .put("index.analysis.tokenizer.mecab_ko.decompound_mode", "mixed")
        );
        builder.get();

        ensureGreen("test");

        // Perform multiple analyses concurrently
        String[] texts = {
            "첫번째 테스트 문장입니다",
            "두번째 테스트 문장입니다",
            "세번째 테스트 문장입니다",
            "네번째 테스트 문장입니다",
            "다섯번째 테스트 문장입니다"
        };

        for (String text : texts) {
            AnalyzeAction.Response response = client().admin().indices()
                .prepareAnalyze("test", text)
                .setTokenizer("mecab_ko")
                .get();

            List<AnalyzeAction.AnalyzeToken> tokens = response.getTokens();
            assertThat("Should produce tokens for: " + text, tokens, is(not(empty())));
        }
    }
}
