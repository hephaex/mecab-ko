package com.mecab.ko.search.core;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.mecab.ko.search.jni.NativeAnalyzer;

import java.io.IOException;
import java.io.Reader;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Set;

/**
 * Base class for MeCab-Ko tokenizers.
 *
 * <p>Provides common tokenization logic shared between Elasticsearch and OpenSearch.
 * Platform-specific implementations should extend this class and implement the
 * abstract methods for Lucene attribute handling.
 */
public abstract class MecabKoTokenizerBase {

    private static final ObjectMapper OBJECT_MAPPER = new ObjectMapper();

    protected final DecompoundMode decompoundMode;
    protected final Set<String> stoptags;
    protected final Path userDictionaryPath;
    protected final boolean outputUnknownUnigrams;

    protected long analyzerHandle;
    protected Iterator<TokenInfo> tokenIterator;
    protected String inputText;

    /**
     * Create tokenizer base.
     *
     * @param decompoundMode compound noun handling mode
     * @param userDictionaryPath path to user dictionary (nullable)
     * @param stoptags POS tags to filter (nullable)
     * @param outputUnknownUnigrams whether to output unknown words as unigrams
     */
    protected MecabKoTokenizerBase(DecompoundMode decompoundMode,
                                    Path userDictionaryPath,
                                    Set<String> stoptags,
                                    boolean outputUnknownUnigrams) {
        this.decompoundMode = decompoundMode != null ? decompoundMode : DecompoundMode.NONE;
        this.userDictionaryPath = userDictionaryPath;
        this.stoptags = stoptags;
        this.outputUnknownUnigrams = outputUnknownUnigrams;
    }

    /**
     * Initialize the native analyzer.
     *
     * @throws RuntimeException if initialization fails
     */
    protected void initializeAnalyzer() {
        String configJson = buildConfigJson();
        analyzerHandle = NativeAnalyzer.createAnalyzer(configJson);
        if (analyzerHandle == 0) {
            throw new RuntimeException("Failed to create native analyzer");
        }
    }

    /**
     * Build configuration JSON for native analyzer.
     *
     * @return JSON configuration string
     */
    protected String buildConfigJson() {
        StringBuilder json = new StringBuilder();
        json.append("{");
        json.append("\"decompound_mode\":\"").append(decompoundMode.toConfigString()).append("\"");

        if (userDictionaryPath != null) {
            json.append(",\"user_dictionary_path\":\"")
                .append(escapeJson(userDictionaryPath.toString()))
                .append("\"");
        }

        json.append(",\"stoptags\":[");
        if (stoptags != null && !stoptags.isEmpty()) {
            boolean first = true;
            for (String tag : stoptags) {
                if (!first) json.append(",");
                json.append("\"").append(escapeJson(tag)).append("\"");
                first = false;
            }
        }
        json.append("]");

        json.append(",\"output_unknown_unigrams\":").append(outputUnknownUnigrams);
        json.append("}");

        return json.toString();
    }

    /**
     * Escape string for JSON.
     */
    private String escapeJson(String str) {
        return str.replace("\\", "\\\\")
                  .replace("\"", "\\\"")
                  .replace("\n", "\\n")
                  .replace("\r", "\\r")
                  .replace("\t", "\\t");
    }

    /**
     * Read input text from reader.
     *
     * @param input input reader
     * @return input text
     * @throws IOException if reading fails
     */
    protected String readInputText(Reader input) throws IOException {
        StringBuilder sb = new StringBuilder();
        char[] buffer = new char[8192];
        int numRead;
        while ((numRead = input.read(buffer)) != -1) {
            sb.append(buffer, 0, numRead);
        }
        return sb.toString();
    }

    /**
     * Tokenize input text.
     *
     * @param text text to tokenize
     * @return list of tokens
     * @throws IOException if tokenization fails
     */
    protected List<TokenInfo> tokenize(String text) throws IOException {
        if (analyzerHandle == 0 || text == null || text.isEmpty()) {
            return new ArrayList<>();
        }

        String resultJson = NativeAnalyzer.analyzeText(analyzerHandle, text);
        return parseTokens(resultJson);
    }

    /**
     * Parse JSON token array from native library.
     *
     * @param json JSON string
     * @return list of tokens
     * @throws IOException if parsing fails
     */
    protected List<TokenInfo> parseTokens(String json) throws IOException {
        List<TokenInfo> tokens = new ArrayList<>();

        if (json == null || json.isEmpty()) {
            return tokens;
        }

        try {
            JsonNode root = OBJECT_MAPPER.readTree(json);
            if (root.isArray()) {
                for (JsonNode node : root) {
                    TokenInfo.Builder builder = new TokenInfo.Builder()
                        .surface(getStringValue(node, "surface", ""))
                        .posTag(getStringValue(node, "pos_tag", "UNK"))
                        .startOffset(getIntValue(node, "start_offset", 0))
                        .endOffset(getIntValue(node, "end_offset", 0));

                    // Optional fields
                    if (node.has("reading")) {
                        builder.reading(node.get("reading").asText());
                    }
                    if (node.has("position_increment")) {
                        builder.positionIncrement(node.get("position_increment").asInt(1));
                    }

                    tokens.add(builder.build());
                }
            }
        } catch (JsonProcessingException e) {
            throw new IOException("Failed to parse token JSON: " + e.getMessage(), e);
        }

        return tokens;
    }

    private String getStringValue(JsonNode node, String field, String defaultValue) {
        JsonNode fieldNode = node.get(field);
        return fieldNode != null ? fieldNode.asText(defaultValue) : defaultValue;
    }

    private int getIntValue(JsonNode node, String field, int defaultValue) {
        JsonNode fieldNode = node.get(field);
        return fieldNode != null ? fieldNode.asInt(defaultValue) : defaultValue;
    }

    /**
     * Process next token.
     *
     * @return true if a token was processed, false if no more tokens
     */
    protected boolean processNextToken() {
        if (tokenIterator == null || !tokenIterator.hasNext()) {
            return false;
        }

        TokenInfo token = tokenIterator.next();
        return populateAttributes(token);
    }

    /**
     * Reset the tokenizer for new input.
     *
     * @param input input reader
     * @throws IOException if reset fails
     */
    protected void resetTokenizer(Reader input) throws IOException {
        inputText = readInputText(input);
        List<TokenInfo> tokens = tokenize(inputText);
        tokenIterator = tokens.iterator();
    }

    /**
     * Get final offset for end().
     *
     * @return final offset
     */
    protected int getFinalOffset() {
        return inputText != null ? inputText.length() : 0;
    }

    /**
     * Close the tokenizer and release resources.
     */
    protected void closeTokenizer() {
        if (analyzerHandle != 0) {
            NativeAnalyzer.destroyAnalyzer(analyzerHandle);
            analyzerHandle = 0;
        }
        tokenIterator = null;
        inputText = null;
    }

    /**
     * Populate Lucene attributes from token info.
     * Must be implemented by platform-specific subclasses.
     *
     * @param token token info
     * @return true if token was processed successfully
     */
    protected abstract boolean populateAttributes(TokenInfo token);

    /**
     * Get decompound mode.
     *
     * @return decompound mode
     */
    public DecompoundMode getDecompoundMode() {
        return decompoundMode;
    }

    /**
     * Get user dictionary path.
     *
     * @return user dictionary path or null
     */
    public Path getUserDictionaryPath() {
        return userDictionaryPath;
    }

    /**
     * Get stoptags.
     *
     * @return set of stoptags or null
     */
    public Set<String> getStoptags() {
        return stoptags;
    }

    /**
     * Check if outputting unknown unigrams.
     *
     * @return true if enabled
     */
    public boolean isOutputUnknownUnigrams() {
        return outputUnknownUnigrams;
    }
}
