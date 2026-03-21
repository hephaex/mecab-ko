package com.mecab.ko.opensearch.plugin;

import com.mecab.ko.opensearch.analysis.MecabKoAnalyzerProvider;
import com.mecab.ko.opensearch.analysis.MecabKoTokenFilterFactory;
import com.mecab.ko.opensearch.analysis.MecabKoTokenizerFactory;
import com.mecab.ko.search.jni.NativeLibraryLoader;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.opensearch.index.analysis.AnalyzerProvider;
import org.opensearch.index.analysis.TokenFilterFactory;
import org.opensearch.index.analysis.TokenizerFactory;
import org.opensearch.indices.analysis.AnalysisModule;
import org.opensearch.plugins.AnalysisPlugin;
import org.opensearch.plugins.Plugin;

import java.util.HashMap;
import java.util.Map;

/**
 * MeCab-Ko OpenSearch Plugin.
 *
 * <p>Provides Korean morphological analysis capabilities using MeCab-Ko
 * with Nori-compatible interface for OpenSearch 3.x.
 *
 * <p>This plugin registers:
 * <ul>
 *   <li>mecab_ko analyzer - Full analysis chain with tokenizer and filters</li>
 *   <li>mecab_ko_tokenizer - Korean morphological tokenizer</li>
 *   <li>mecab_ko_part_of_speech - POS-based token filter</li>
 *   <li>mecab_ko_reading_form - Reading form token filter</li>
 * </ul>
 *
 * <p>Nori-compatible aliases are also registered:
 * <ul>
 *   <li>nori - alias for mecab_ko analyzer</li>
 *   <li>nori_tokenizer - alias for mecab_ko_tokenizer</li>
 *   <li>nori_part_of_speech - alias for mecab_ko_part_of_speech</li>
 *   <li>nori_reading_form - alias for mecab_ko_reading_form</li>
 * </ul>
 *
 * <p>Configuration example:
 * <pre>{@code
 * {
 *   "settings": {
 *     "analysis": {
 *       "analyzer": {
 *         "my_analyzer": {
 *           "type": "mecab_ko",
 *           "decompound_mode": "mixed",
 *           "stoptags": ["J", "E"]
 *         }
 *       },
 *       "tokenizer": {
 *         "my_tokenizer": {
 *           "type": "mecab_ko_tokenizer",
 *           "decompound_mode": "discard",
 *           "user_dictionary": "userdict_ko.txt"
 *         }
 *       }
 *     }
 *   }
 * }
 * }</pre>
 */
public class MecabKoPlugin extends Plugin implements AnalysisPlugin {

    private static final Logger logger = LogManager.getLogger(MecabKoPlugin.class);

    /**
     * Plugin constructor.
     * Loads native library on plugin initialization.
     */
    public MecabKoPlugin() {
        logger.info("Initializing MeCab-Ko OpenSearch Plugin");
        try {
            NativeLibraryLoader.load();
            logger.info("MeCab-Ko native library loaded successfully (platform: {})",
                        NativeLibraryLoader.getPlatform());
        } catch (UnsatisfiedLinkError e) {
            logger.error("Failed to load MeCab-Ko native library", e);
            throw new RuntimeException("Failed to load MeCab-Ko native library", e);
        }
    }

    /**
     * Register custom analyzers.
     *
     * @return map of analyzer providers
     */
    @Override
    public Map<String, AnalysisModule.AnalysisProvider<AnalyzerProvider<?>>> getAnalyzers() {
        Map<String, AnalysisModule.AnalysisProvider<AnalyzerProvider<?>>> analyzers = new HashMap<>();

        // Register mecab_ko analyzer
        analyzers.put("mecab_ko", MecabKoAnalyzerProvider::new);

        // Nori compatibility alias
        analyzers.put("nori", MecabKoAnalyzerProvider::new);

        logger.debug("Registered analyzers: mecab_ko, nori");
        return analyzers;
    }

    /**
     * Register custom tokenizers.
     *
     * @return map of tokenizer factories
     */
    @Override
    public Map<String, AnalysisModule.AnalysisProvider<TokenizerFactory>> getTokenizers() {
        Map<String, AnalysisModule.AnalysisProvider<TokenizerFactory>> tokenizers = new HashMap<>();

        // Register mecab_ko_tokenizer
        tokenizers.put("mecab_ko_tokenizer", MecabKoTokenizerFactory::new);

        // Nori compatibility alias
        tokenizers.put("nori_tokenizer", MecabKoTokenizerFactory::new);

        logger.debug("Registered tokenizers: mecab_ko_tokenizer, nori_tokenizer");
        return tokenizers;
    }

    /**
     * Register custom token filters.
     *
     * @return map of token filter factories
     */
    @Override
    public Map<String, AnalysisModule.AnalysisProvider<TokenFilterFactory>> getTokenFilters() {
        Map<String, AnalysisModule.AnalysisProvider<TokenFilterFactory>> filters = new HashMap<>();

        // Register POS filter
        filters.put("mecab_ko_part_of_speech",
            (indexSettings, env, name, settings) ->
                new MecabKoTokenFilterFactory(indexSettings, env, name, settings,
                    MecabKoTokenFilterFactory.FilterType.PART_OF_SPEECH));

        // Register reading form filter
        filters.put("mecab_ko_reading_form",
            (indexSettings, env, name, settings) ->
                new MecabKoTokenFilterFactory(indexSettings, env, name, settings,
                    MecabKoTokenFilterFactory.FilterType.READING_FORM));

        // Nori compatibility aliases
        filters.put("nori_part_of_speech",
            (indexSettings, env, name, settings) ->
                new MecabKoTokenFilterFactory(indexSettings, env, name, settings,
                    MecabKoTokenFilterFactory.FilterType.PART_OF_SPEECH));

        filters.put("nori_readingform",
            (indexSettings, env, name, settings) ->
                new MecabKoTokenFilterFactory(indexSettings, env, name, settings,
                    MecabKoTokenFilterFactory.FilterType.READING_FORM));

        logger.debug("Registered token filters: mecab_ko_part_of_speech, mecab_ko_reading_form, " +
                     "nori_part_of_speech, nori_readingform");
        return filters;
    }
}
