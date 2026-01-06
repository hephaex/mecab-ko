package com.mecab.ko.elasticsearch.analysis;

import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.apache.lucene.analysis.Analyzer;
import org.elasticsearch.common.settings.Settings;
import org.elasticsearch.env.Environment;
import org.elasticsearch.index.IndexSettings;
import org.elasticsearch.index.analysis.AbstractIndexAnalyzerProvider;

import java.io.IOException;
import java.nio.file.Path;

/**
 * Provider for MeCab-Ko analyzer.
 *
 * Creates analyzer instances with configuration from index settings.
 *
 * <p>Supported settings:
 * <ul>
 *   <li>decompound_mode: none|discard|mixed (default: none)</li>
 *   <li>user_dictionary: path to user dictionary file (optional)</li>
 *   <li>stoptags: array of POS tags to filter (default: ["J", "E"])</li>
 *   <li>output_unknown_unigrams: boolean (default: false)</li>
 * </ul>
 */
public class MecabKoAnalyzerProvider extends AbstractIndexAnalyzerProvider<Analyzer> {

    private static final Logger logger = LogManager.getLogger(MecabKoAnalyzerProvider.class);

    private final MecabKoAnalyzer analyzer;

    /**
     * Create analyzer provider.
     *
     * @param indexSettings index settings
     * @param env environment
     * @param name analyzer name
     * @param settings analyzer settings
     */
    public MecabKoAnalyzerProvider(IndexSettings indexSettings,
                                    Environment env,
                                    String name,
                                    Settings settings) {
        super(indexSettings, name, settings);

        // Parse decompound mode
        String decompoundModeStr = settings.get("decompound_mode", "none");
        DecompoundMode decompoundMode = DecompoundMode.fromString(decompoundModeStr);

        // Parse user dictionary path
        Path userDictPath = null;
        String userDictStr = settings.get("user_dictionary");
        if (userDictStr != null && !userDictStr.isEmpty()) {
            userDictPath = env.configFile().resolve(userDictStr);
            if (!userDictPath.toFile().exists()) {
                logger.warn("User dictionary not found: {}", userDictPath);
                userDictPath = null;
            }
        }

        // Parse stoptags
        String[] stoptags = settings.getAsArray("stoptags", new String[]{"J", "E"});

        // Parse output_unknown_unigrams
        boolean outputUnknownUnigrams = settings.getAsBoolean("output_unknown_unigrams", false);

        logger.info("Creating MeCab-Ko analyzer '{}' with mode={}, stoptags={}, userDict={}",
                    name, decompoundMode, java.util.Arrays.toString(stoptags),
                    userDictPath != null ? userDictPath.toString() : "none");

        try {
            this.analyzer = new MecabKoAnalyzer(
                decompoundMode,
                userDictPath,
                stoptags,
                outputUnknownUnigrams
            );
        } catch (IOException e) {
            throw new RuntimeException("Failed to create MeCab-Ko analyzer", e);
        }
    }

    @Override
    public Analyzer get() {
        return analyzer;
    }
}
