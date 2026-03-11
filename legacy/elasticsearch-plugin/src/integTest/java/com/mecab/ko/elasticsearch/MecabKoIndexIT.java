package com.mecab.ko.elasticsearch;

import org.elasticsearch.action.admin.indices.create.CreateIndexRequestBuilder;
import org.elasticsearch.action.index.IndexResponse;
import org.elasticsearch.action.search.SearchResponse;
import org.elasticsearch.common.settings.Settings;
import org.elasticsearch.index.query.QueryBuilders;
import org.elasticsearch.plugins.Plugin;
import org.elasticsearch.rest.RestStatus;
import org.elasticsearch.search.SearchHit;
import org.elasticsearch.test.ESIntegTestCase;
import org.elasticsearch.xcontent.XContentType;
import com.mecab.ko.elasticsearch.plugin.MecabKoPlugin;

import java.util.Collection;
import java.util.Collections;

import static org.hamcrest.Matchers.*;

/**
 * Integration tests for MeCab-Ko indexing and search.
 *
 * Tests full end-to-end indexing and retrieval scenarios.
 */
@ESIntegTestCase.ClusterScope(scope = ESIntegTestCase.Scope.TEST)
public class MecabKoIndexIT extends ESIntegTestCase {

    @Override
    protected Collection<Class<? extends Plugin>> nodePlugins() {
        return Collections.singletonList(MecabKoPlugin.class);
    }

    public void testBasicIndexingAndSearch() throws Exception {
        // Create index with MeCab-Ko analyzer
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
            .put("index.analysis.analyzer.mecab_ko.decompound_mode", "mixed")
        );
        builder.setMapping(
            "properties", Collections.singletonMap("content",
                Collections.singletonMap("type", "text")
                    .toString()
                    .replace("=", ":")
                    .replace("{", "{\"")
                    .replace("}", "\"}")
                    .replace(", ", "\", \"")
            )
        );

        String mapping = "{\n" +
            "  \"properties\": {\n" +
            "    \"content\": {\n" +
            "      \"type\": \"text\",\n" +
            "      \"analyzer\": \"mecab_ko\"\n" +
            "    }\n" +
            "  }\n" +
            "}";
        builder.setMapping(mapping);
        builder.get();

        ensureGreen("test");

        // Index documents
        String[] documents = {
            "한국어 형태소 분석기입니다",
            "Elasticsearch 한글 검색 플러그인",
            "자연어 처리를 위한 도구"
        };

        for (int i = 0; i < documents.length; i++) {
            IndexResponse response = client().prepareIndex("test")
                .setId(String.valueOf(i))
                .setSource("content", documents[i])
                .get();

            assertEquals("Document should be indexed", RestStatus.CREATED, response.status());
        }

        // Refresh to make documents searchable
        client().admin().indices().prepareRefresh("test").get();

        // Search for documents
        SearchResponse searchResponse = client().prepareSearch("test")
            .setQuery(QueryBuilders.matchQuery("content", "형태소"))
            .get();

        assertThat("Should find documents", searchResponse.getHits().getTotalHits().value, greaterThan(0L));

        logger.info("Search for '형태소' found {} hits", searchResponse.getHits().getTotalHits().value);
        for (SearchHit hit : searchResponse.getHits()) {
            logger.info("  Hit: {}", hit.getSourceAsMap().get("content"));
        }
    }

    public void testDecompoundedSearch() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
            .put("index.analysis.analyzer.mecab_ko.decompound_mode", "mixed")
        );

        String mapping = "{\n" +
            "  \"properties\": {\n" +
            "    \"content\": {\n" +
            "      \"type\": \"text\",\n" +
            "      \"analyzer\": \"mecab_ko\"\n" +
            "    }\n" +
            "  }\n" +
            "}";
        builder.setMapping(mapping);
        builder.get();

        ensureGreen("test");

        // Index document with compound noun
        client().prepareIndex("test")
            .setId("1")
            .setSource("content", "형태소분석기")
            .get();

        client().admin().indices().prepareRefresh("test").get();

        // Search with compound form
        SearchResponse response1 = client().prepareSearch("test")
            .setQuery(QueryBuilders.matchQuery("content", "형태소분석기"))
            .get();
        assertThat("Should find with compound form", response1.getHits().getTotalHits().value, equalTo(1L));

        // Search with decomposed form (should also match in mixed mode)
        SearchResponse response2 = client().prepareSearch("test")
            .setQuery(QueryBuilders.matchQuery("content", "형태소"))
            .get();
        assertThat("Should find with decomposed form", response2.getHits().getTotalHits().value, equalTo(1L));

        SearchResponse response3 = client().prepareSearch("test")
            .setQuery(QueryBuilders.matchQuery("content", "분석"))
            .get();
        assertThat("Should find with decomposed form", response3.getHits().getTotalHits().value, equalTo(1L));
    }

    public void testMultiFieldMapping() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko_mixed.type", "mecab_ko")
            .put("index.analysis.analyzer.mecab_ko_mixed.decompound_mode", "mixed")
            .put("index.analysis.analyzer.mecab_ko_none.type", "mecab_ko")
            .put("index.analysis.analyzer.mecab_ko_none.decompound_mode", "none")
        );

        String mapping = "{\n" +
            "  \"properties\": {\n" +
            "    \"content\": {\n" +
            "      \"type\": \"text\",\n" +
            "      \"analyzer\": \"mecab_ko_mixed\",\n" +
            "      \"fields\": {\n" +
            "        \"exact\": {\n" +
            "          \"type\": \"text\",\n" +
            "          \"analyzer\": \"mecab_ko_none\"\n" +
            "        }\n" +
            "      }\n" +
            "    }\n" +
            "  }\n" +
            "}";
        builder.setMapping(mapping);
        builder.get();

        ensureGreen("test");

        // Index document
        client().prepareIndex("test")
            .setId("1")
            .setSource("content", "형태소분석기")
            .get();

        client().admin().indices().prepareRefresh("test").get();

        // Search on main field (mixed mode)
        SearchResponse response1 = client().prepareSearch("test")
            .setQuery(QueryBuilders.matchQuery("content", "형태소"))
            .get();
        assertThat("Should find in mixed mode", response1.getHits().getTotalHits().value, equalTo(1L));

        // Search on exact field (none mode - should not decompose)
        SearchResponse response2 = client().prepareSearch("test")
            .setQuery(QueryBuilders.matchQuery("content.exact", "형태소분석기"))
            .get();
        assertThat("Should find exact match", response2.getHits().getTotalHits().value, equalTo(1L));
    }

    public void testBulkIndexing() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
            .put("index.number_of_shards", 1)
            .put("index.number_of_replicas", 0)
        );

        String mapping = "{\n" +
            "  \"properties\": {\n" +
            "    \"content\": {\n" +
            "      \"type\": \"text\",\n" +
            "      \"analyzer\": \"mecab_ko\"\n" +
            "    }\n" +
            "  }\n" +
            "}";
        builder.setMapping(mapping);
        builder.get();

        ensureGreen("test");

        // Index multiple documents
        int numDocs = 100;
        for (int i = 0; i < numDocs; i++) {
            client().prepareIndex("test")
                .setId(String.valueOf(i))
                .setSource("content", "문서 번호 " + i + " 한국어 테스트")
                .get();
        }

        client().admin().indices().prepareRefresh("test").get();

        // Verify all documents are indexed
        SearchResponse response = client().prepareSearch("test")
            .setQuery(QueryBuilders.matchAllQuery())
            .setSize(0)
            .get();

        assertEquals("Should index all documents", numDocs, response.getHits().getTotalHits().value);
        logger.info("Bulk indexed {} documents", numDocs);
    }

    public void testMixedLanguageIndexing() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
        );

        String mapping = "{\n" +
            "  \"properties\": {\n" +
            "    \"content\": {\n" +
            "      \"type\": \"text\",\n" +
            "      \"analyzer\": \"mecab_ko\"\n" +
            "    }\n" +
            "  }\n" +
            "}";
        builder.setMapping(mapping);
        builder.get();

        ensureGreen("test");

        // Index documents with mixed Korean/English
        String[] documents = {
            "Elasticsearch는 검색엔진입니다",
            "Java와 Python 프로그래밍",
            "REST API를 통한 데이터 접근",
            "MeCab-Ko 한글 분석기"
        };

        for (int i = 0; i < documents.length; i++) {
            client().prepareIndex("test")
                .setId(String.valueOf(i))
                .setSource("content", documents[i])
                .get();
        }

        client().admin().indices().prepareRefresh("test").get();

        // Search for Korean term
        SearchResponse response1 = client().prepareSearch("test")
            .setQuery(QueryBuilders.matchQuery("content", "검색"))
            .get();
        assertThat("Should find Korean term", response1.getHits().getTotalHits().value, greaterThan(0L));

        // Search for English term
        SearchResponse response2 = client().prepareSearch("test")
            .setQuery(QueryBuilders.matchQuery("content", "API"))
            .get();
        assertThat("Should find English term", response2.getHits().getTotalHits().value, greaterThan(0L));
    }

    public void testPhraseQuery() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
            .put("index.analysis.analyzer.mecab_ko.decompound_mode", "none")
        );

        String mapping = "{\n" +
            "  \"properties\": {\n" +
            "    \"content\": {\n" +
            "      \"type\": \"text\",\n" +
            "      \"analyzer\": \"mecab_ko\"\n" +
            "    }\n" +
            "  }\n" +
            "}";
        builder.setMapping(mapping);
        builder.get();

        ensureGreen("test");

        // Index documents
        client().prepareIndex("test").setId("1")
            .setSource("content", "한국어 형태소 분석기입니다")
            .get();
        client().prepareIndex("test").setId("2")
            .setSource("content", "형태소 분석기는 유용합니다")
            .get();
        client().prepareIndex("test").setId("3")
            .setSource("content", "한국어 처리 도구")
            .get();

        client().admin().indices().prepareRefresh("test").get();

        // Phrase query
        SearchResponse response = client().prepareSearch("test")
            .setQuery(QueryBuilders.matchPhraseQuery("content", "형태소 분석"))
            .get();

        logger.info("Phrase query for '형태소 분석' found {} hits", response.getHits().getTotalHits().value);
        for (SearchHit hit : response.getHits()) {
            logger.info("  Hit: {}", hit.getSourceAsMap().get("content"));
        }
    }

    public void testBooleanQuery() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
        );

        String mapping = "{\n" +
            "  \"properties\": {\n" +
            "    \"content\": {\n" +
            "      \"type\": \"text\",\n" +
            "      \"analyzer\": \"mecab_ko\"\n" +
            "    }\n" +
            "  }\n" +
            "}";
        builder.setMapping(mapping);
        builder.get();

        ensureGreen("test");

        // Index documents
        client().prepareIndex("test").setId("1")
            .setSource("content", "한국어 형태소 분석기")
            .get();
        client().prepareIndex("test").setId("2")
            .setSource("content", "영어 형태소 분석기")
            .get();
        client().prepareIndex("test").setId("3")
            .setSource("content", "한국어 번역기")
            .get();

        client().admin().indices().prepareRefresh("test").get();

        // Boolean query: must have "한국어" and "분석기"
        SearchResponse response = client().prepareSearch("test")
            .setQuery(QueryBuilders.boolQuery()
                .must(QueryBuilders.matchQuery("content", "한국어"))
                .must(QueryBuilders.matchQuery("content", "분석기"))
            )
            .get();

        assertEquals("Should find exactly one document", 1L, response.getHits().getTotalHits().value);
        assertEquals("Should find document 1", "1", response.getHits().getAt(0).getId());
    }

    public void testFuzzyQuery() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
        );

        String mapping = "{\n" +
            "  \"properties\": {\n" +
            "    \"content\": {\n" +
            "      \"type\": \"text\",\n" +
            "      \"analyzer\": \"mecab_ko\"\n" +
            "    }\n" +
            "  }\n" +
            "}";
        builder.setMapping(mapping);
        builder.get();

        ensureGreen("test");

        // Index documents
        client().prepareIndex("test").setId("1")
            .setSource("content", "형태소 분석")
            .get();

        client().admin().indices().prepareRefresh("test").get();

        // Fuzzy query (handles typos)
        SearchResponse response = client().prepareSearch("test")
            .setQuery(QueryBuilders.fuzzyQuery("content", "형태소"))
            .get();

        assertThat("Fuzzy query should find results", response.getHits().getTotalHits().value, greaterThan(0L));
    }

    public void testHighlighting() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
        );

        String mapping = "{\n" +
            "  \"properties\": {\n" +
            "    \"content\": {\n" +
            "      \"type\": \"text\",\n" +
            "      \"analyzer\": \"mecab_ko\"\n" +
            "    }\n" +
            "  }\n" +
            "}";
        builder.setMapping(mapping);
        builder.get();

        ensureGreen("test");

        // Index document
        client().prepareIndex("test").setId("1")
            .setSource("content", "한국어 형태소 분석기는 자연어 처리의 핵심 도구입니다")
            .get();

        client().admin().indices().prepareRefresh("test").get();

        // Search with highlighting
        SearchResponse response = client().prepareSearch("test")
            .setQuery(QueryBuilders.matchQuery("content", "형태소"))
            .highlighter(
                new org.elasticsearch.search.fetch.subphase.highlight.HighlightBuilder()
                    .field("content")
            )
            .get();

        assertThat("Should find document", response.getHits().getTotalHits().value, equalTo(1L));

        SearchHit hit = response.getHits().getAt(0);
        if (hit.getHighlightFields().get("content") != null) {
            logger.info("Highlighted: {}",
                hit.getHighlightFields().get("content").fragments()[0].string());
        }
    }

    public void testAggregations() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
        );

        String mapping = "{\n" +
            "  \"properties\": {\n" +
            "    \"content\": {\n" +
            "      \"type\": \"text\",\n" +
            "      \"analyzer\": \"mecab_ko\",\n" +
            "      \"fielddata\": true\n" +
            "    },\n" +
            "    \"category\": {\n" +
            "      \"type\": \"keyword\"\n" +
            "    }\n" +
            "  }\n" +
            "}";
        builder.setMapping(mapping);
        builder.get();

        ensureGreen("test");

        // Index documents with categories
        client().prepareIndex("test").setId("1")
            .setSource("content", "형태소 분석", "category", "NLP")
            .get();
        client().prepareIndex("test").setId("2")
            .setSource("content", "검색 엔진", "category", "Search")
            .get();
        client().prepareIndex("test").setId("3")
            .setSource("content", "자연어 처리", "category", "NLP")
            .get();

        client().admin().indices().prepareRefresh("test").get();

        // Aggregation by category
        SearchResponse response = client().prepareSearch("test")
            .setSize(0)
            .addAggregation(
                org.elasticsearch.search.aggregations.AggregationBuilders
                    .terms("categories")
                    .field("category")
            )
            .get();

        org.elasticsearch.search.aggregations.bucket.terms.Terms agg =
            response.getAggregations().get("categories");

        logger.info("Aggregation results:");
        for (org.elasticsearch.search.aggregations.bucket.terms.Terms.Bucket bucket : agg.getBuckets()) {
            logger.info("  Category: {}, count: {}", bucket.getKey(), bucket.getDocCount());
        }

        assertThat("Should have aggregation buckets", agg.getBuckets().size(), greaterThan(0));
    }

    public void testUpdateDocument() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
        );

        String mapping = "{\n" +
            "  \"properties\": {\n" +
            "    \"content\": {\n" +
            "      \"type\": \"text\",\n" +
            "      \"analyzer\": \"mecab_ko\"\n" +
            "    }\n" +
            "  }\n" +
            "}";
        builder.setMapping(mapping);
        builder.get();

        ensureGreen("test");

        // Index initial document
        client().prepareIndex("test").setId("1")
            .setSource("content", "초기 내용")
            .get();

        client().admin().indices().prepareRefresh("test").get();

        // Update document
        client().prepareUpdate("test", "1")
            .setDoc("content", "업데이트된 내용입니다")
            .get();

        client().admin().indices().prepareRefresh("test").get();

        // Search for updated content
        SearchResponse response = client().prepareSearch("test")
            .setQuery(QueryBuilders.matchQuery("content", "업데이트"))
            .get();

        assertEquals("Should find updated document", 1L, response.getHits().getTotalHits().value);
    }

    public void testDeleteDocument() throws Exception {
        CreateIndexRequestBuilder builder = client().admin().indices().prepareCreate("test");
        builder.setSettings(Settings.builder()
            .put("index.analysis.analyzer.mecab_ko.type", "mecab_ko")
        );

        String mapping = "{\n" +
            "  \"properties\": {\n" +
            "    \"content\": {\n" +
            "      \"type\": \"text\",\n" +
            "      \"analyzer\": \"mecab_ko\"\n" +
            "    }\n" +
            "  }\n" +
            "}";
        builder.setMapping(mapping);
        builder.get();

        ensureGreen("test");

        // Index document
        client().prepareIndex("test").setId("1")
            .setSource("content", "삭제될 문서")
            .get();

        client().admin().indices().prepareRefresh("test").get();

        // Verify document exists
        SearchResponse response1 = client().prepareSearch("test")
            .setQuery(QueryBuilders.matchAllQuery())
            .get();
        assertEquals("Should find document", 1L, response1.getHits().getTotalHits().value);

        // Delete document
        client().prepareDelete("test", "1").get();
        client().admin().indices().prepareRefresh("test").get();

        // Verify document is deleted
        SearchResponse response2 = client().prepareSearch("test")
            .setQuery(QueryBuilders.matchAllQuery())
            .get();
        assertEquals("Should not find document", 0L, response2.getHits().getTotalHits().value);
    }
}
