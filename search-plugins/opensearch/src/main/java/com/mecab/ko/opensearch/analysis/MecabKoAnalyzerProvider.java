package com.mecab.ko.opensearch.analysis;

import com.mecab.ko.search.core.DecompoundMode;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.apache.lucene.analysis.Analyzer;
import org.opensearch.common.settings.Settings;
import org.opensearch.env.Environment;
import org.opensearch.index.IndexSettings;
import org.opensearch.index.analysis.AbstractIndexAnalyzerProvider;

import java.nio.file.Path;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

/**
 * Provider for MeCab-Ko analyzer in OpenSearch.
 *
 * <p>Creates analyzer instances with configuration from index settings.
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

    private static final String[] DEFAULT_STOPTAGS = {"J", "E"};

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
            userDictPath = env.configDir().resolve(userDictStr);
            if (!userDictPath.toFile().exists()) {
                logger.warn("User dictionary not found: {}", userDictPath);
                userDictPath = null;
            }
        }

        // Parse stoptags (OpenSearch 3.x: getAsList replaces getAsArray)
        List<String> stoptagsList = settings.getAsList("stoptags", List.of(DEFAULT_STOPTAGS));
        Set<String> stoptags = new HashSet<>(stoptagsList);

        // Parse output_unknown_unigrams
        boolean outputUnknownUnigrams = Boolean.TRUE.equals(
            settings.getAsBoolean("output_unknown_unigrams", false));

        logger.info("Creating MeCab-Ko analyzer '{}' with mode={}, stoptags={}, userDict={}",
                    name, decompoundMode, stoptags,
                    userDictPath != null ? userDictPath.toString() : "none");

        this.analyzer = new MecabKoAnalyzer(
            decompoundMode,
            userDictPath,
            stoptags,
            outputUnknownUnigrams
        );
    }

    @Override
    public Analyzer get() {
        return analyzer;
    }
}
