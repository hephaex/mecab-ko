package com.mecab.ko.elasticsearch.analysis;

import com.mecab.ko.search.core.DecompoundMode;
import org.apache.lucene.analysis.Analyzer;
import org.apache.lucene.analysis.TokenStream;
import org.apache.lucene.analysis.Tokenizer;

import java.nio.file.Path;
import java.util.Set;

/**
 * MeCab-Ko Korean Analyzer for Elasticsearch/Lucene 9.x.
 *
 * <p>Provides morphological analysis with configurable decompound modes
 * and POS-based filtering.
 */
public class MecabKoAnalyzer extends Analyzer {

    private final DecompoundMode decompoundMode;
    private final Path userDictionaryPath;
    private final Set<String> stoptags;
    private final boolean outputUnknownUnigrams;

    /**
     * Create analyzer with default settings.
     */
    public MecabKoAnalyzer() {
        this(DecompoundMode.NONE, null, null, false);
    }

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
                           Set<String> stoptags,
                           boolean outputUnknownUnigrams) {
        this.decompoundMode = decompoundMode != null ? decompoundMode : DecompoundMode.NONE;
        this.userDictionaryPath = userDictionaryPath;
        this.stoptags = stoptags;
        this.outputUnknownUnigrams = outputUnknownUnigrams;
    }

    @Override
    protected TokenStreamComponents createComponents(String fieldName) {
        // Create tokenizer
        Tokenizer tokenizer = new MecabKoTokenizer(
            decompoundMode,
            userDictionaryPath,
            null, // stoptags handled by filter
            outputUnknownUnigrams
        );

        // Apply filters
        TokenStream stream = tokenizer;

        // Apply POS filter if stoptags are configured
        if (stoptags != null && !stoptags.isEmpty()) {
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
     * @return user dictionary path or null
     */
    public Path getUserDictionaryPath() {
        return userDictionaryPath;
    }

    /**
     * Get stoptags.
     *
     * @return set of stoptags
     */
    public Set<String> getStoptags() {
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
