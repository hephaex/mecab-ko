package com.mecab.ko.elasticsearch.analysis;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.lucene.analysis.Tokenizer;
import org.apache.lucene.analysis.tokenattributes.CharTermAttribute;
import org.apache.lucene.analysis.tokenattributes.OffsetAttribute;
import org.apache.lucene.analysis.tokenattributes.PositionIncrementAttribute;
import org.apache.lucene.analysis.tokenattributes.TypeAttribute;

import java.io.IOException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

/**
 * MeCab-Ko tokenizer for Lucene.
 *
 * Wraps native MeCab-Ko library via JNI for morphological analysis.
 */
public class MecabKoTokenizer extends Tokenizer {

    private final CharTermAttribute termAtt = addAttribute(CharTermAttribute.class);
    private final OffsetAttribute offsetAtt = addAttribute(OffsetAttribute.class);
    private final PositionIncrementAttribute posIncrAtt = addAttribute(PositionIncrementAttribute.class);
    private final TypeAttribute typeAtt = addAttribute(TypeAttribute.class);

    private final DecompoundMode decompoundMode;
    private final Path userDictionaryPath;
    private final boolean outputUnknownUnigrams;

    private long analyzerHandle = 0;
    private Iterator<TokenInfo> tokenIterator;
    private String inputText;

    private static final ObjectMapper objectMapper = new ObjectMapper();

    /**
     * Create tokenizer.
     *
     * @param decompoundMode compound noun handling mode
     * @param userDictionaryPath path to user dictionary (nullable)
     * @param outputUnknownUnigrams whether to output unknown words as unigrams
     */
    public MecabKoTokenizer(DecompoundMode decompoundMode,
                            Path userDictionaryPath,
                            boolean outputUnknownUnigrams) {
        this.decompoundMode = decompoundMode;
        this.userDictionaryPath = userDictionaryPath;
        this.outputUnknownUnigrams = outputUnknownUnigrams;
        initializeAnalyzer();
    }

    /**
     * Initialize native analyzer.
     */
    private void initializeAnalyzer() {
        try {
            String configJson = buildConfigJson();
            analyzerHandle = createAnalyzer(configJson);
        } catch (JsonProcessingException e) {
            throw new RuntimeException("Failed to create analyzer config", e);
        }
    }

    /**
     * Build configuration JSON for native analyzer.
     */
    private String buildConfigJson() throws JsonProcessingException {
        StringBuilder json = new StringBuilder();
        json.append("{");
        json.append("\"decompound_mode\":\"").append(decompoundMode.toConfigString()).append("\"");

        if (userDictionaryPath != null) {
            json.append(",\"user_dictionary_path\":\"")
                .append(userDictionaryPath.toString().replace("\\", "\\\\"))
                .append("\"");
        }

        json.append(",\"stoptags\":[]");
        json.append(",\"output_unknown_unigrams\":").append(outputUnknownUnigrams);
        json.append("}");

        return json.toString();
    }

    @Override
    public boolean incrementToken() throws IOException {
        clearAttributes();

        if (tokenIterator == null || !tokenIterator.hasNext()) {
            return false;
        }

        TokenInfo token = tokenIterator.next();

        termAtt.setEmpty().append(token.surface);
        offsetAtt.setOffset(token.startOffset, token.endOffset);
        posIncrAtt.setPositionIncrement(1);
        typeAtt.setType(token.posTag);

        return true;
    }

    @Override
    public void reset() throws IOException {
        super.reset();

        // Read input text
        StringBuilder sb = new StringBuilder();
        char[] buffer = new char[8192];
        int numRead;
        while ((numRead = input.read(buffer)) != -1) {
            sb.append(buffer, 0, numRead);
        }
        inputText = sb.toString();

        // Tokenize via native library
        if (analyzerHandle != 0 && !inputText.isEmpty()) {
            String resultJson = analyzeText(analyzerHandle, inputText);
            tokenIterator = parseTokens(resultJson).iterator();
        } else {
            tokenIterator = new ArrayList<TokenInfo>().iterator();
        }
    }

    @Override
    public void end() throws IOException {
        super.end();
        // Set final offset
        int finalOffset = inputText != null ? inputText.length() : 0;
        offsetAtt.setOffset(finalOffset, finalOffset);
    }

    @Override
    public void close() throws IOException {
        super.close();
        if (analyzerHandle != 0) {
            destroyAnalyzer(analyzerHandle);
            analyzerHandle = 0;
        }
    }

    /**
     * Parse JSON token array from native library.
     */
    private List<TokenInfo> parseTokens(String json) throws IOException {
        List<TokenInfo> tokens = new ArrayList<>();

        try {
            JsonNode root = objectMapper.readTree(json);
            if (root.isArray()) {
                for (JsonNode node : root) {
                    TokenInfo token = new TokenInfo(
                        node.get("surface").asText(),
                        node.get("pos_tag").asText(),
                        node.get("start_offset").asInt(),
                        node.get("end_offset").asInt()
                    );
                    tokens.add(token);
                }
            }
        } catch (JsonProcessingException e) {
            throw new IOException("Failed to parse token JSON", e);
        }

        return tokens;
    }

    /**
     * Token information from native library.
     */
    private static class TokenInfo {
        final String surface;
        final String posTag;
        final int startOffset;
        final int endOffset;

        TokenInfo(String surface, String posTag, int startOffset, int endOffset) {
            this.surface = surface;
            this.posTag = posTag;
            this.startOffset = startOffset;
            this.endOffset = endOffset;
        }
    }

    // Native methods (from JNI bindings)
    private static native long createAnalyzer(String configJson);
    private static native String analyzeText(long handle, String text);
    private static native void destroyAnalyzer(long handle);
}
