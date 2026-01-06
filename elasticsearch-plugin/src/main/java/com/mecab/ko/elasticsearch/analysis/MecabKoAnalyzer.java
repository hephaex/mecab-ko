package com.mecab.ko.elasticsearch.analysis;

import org.apache.lucene.analysis.Analyzer;
import org.apache.lucene.analysis.TokenStream;
import org.apache.lucene.analysis.Tokenizer;

import java.io.IOException;
import java.nio.file.Path;

/**
 * MeCab-Ko Korean Analyzer for Lucene.
 *
 * Provides morphological analysis with configurable decompound modes
 * and POS-based filtering.
 */
public class MecabKoAnalyzer extends Analyzer {

    private final DecompoundMode decompoundMode;
    private final Path userDictionaryPath;
    private final String[] stoptags;
    private final boolean outputUnknownUnigrams;

    /**
     * Create analyzer with configuration.
     *
     * @param decompoundMode compound noun handling mode
     * @param userDictionaryPath path to user dictionary (nullable)
     * @param stoptags POS tags to filter out
     * @param outputUnknownUnigrams whether to output unknown words as unigrams
     */
    public MecabKoAnalyzer(DecompoundMode decompoundMode,
                           Path userDictionaryPath,
                           String[] stoptags,
                           boolean outputUnknownUnigrams) throws IOException {
        this.decompoundMode = decompoundMode;
        this.userDictionaryPath = userDictionaryPath;
        this.stoptags = stoptags != null ? stoptags : new String[0];
        this.outputUnknownUnigrams = outputUnknownUnigrams;
    }

    @Override
    protected TokenStreamComponents createComponents(String fieldName) {
        // Create tokenizer
        Tokenizer tokenizer = new MecabKoTokenizer(
            decompoundMode,
            userDictionaryPath,
            outputUnknownUnigrams
        );

        // Apply filters
        TokenStream stream = tokenizer;

        // Apply POS filter if stoptags are configured
        if (stoptags.length > 0) {
            stream = new MecabKoPartOfSpeechStopFilter(stream, stoptags);
        }

        return new TokenStreamComponents(tokenizer, stream);
    }

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
     * @return user dictionary path (nullable)
     */
    public Path getUserDictionaryPath() {
        return userDictionaryPath;
    }

    /**
     * Get stoptags.
     *
     * @return array of stoptags
     */
    public String[] getStoptags() {
        return stoptags;
    }

    /**
     * Check if unknown unigrams should be output.
     *
     * @return true if enabled
     */
    public boolean isOutputUnknownUnigrams() {
        return outputUnknownUnigrams;
    }
}
