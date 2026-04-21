package com.mecab.ko.elasticsearch.analysis;

import com.mecab.ko.search.core.DecompoundMode;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.apache.lucene.analysis.Tokenizer;
import org.elasticsearch.common.settings.Settings;
import org.elasticsearch.env.Environment;
import org.elasticsearch.index.IndexSettings;
import org.elasticsearch.index.analysis.AbstractTokenizerFactory;

import java.nio.file.Path;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

/**
 * Factory for MeCab-Ko tokenizer in Elasticsearch.
 *
 * <p>Creates tokenizer instances with configuration from index settings.
 *
 * <p>Supported settings:
 * <ul>
 *   <li>decompound_mode: none|discard|mixed (default: none)</li>
 *   <li>user_dictionary: path to user dictionary file (optional)</li>
 *   <li>stoptags: array of POS tags to filter (optional)</li>
 *   <li>output_unknown_unigrams: boolean (default: false)</li>
 * </ul>
 *
 * <p>Example configuration:
 * <pre>{@code
 * {
 *   "tokenizer": {
 *     "my_tokenizer": {
 *       "type": "mecab_ko_tokenizer",
 *       "decompound_mode": "mixed",
 *       "user_dictionary": "userdict_ko.txt",
 *       "output_unknown_unigrams": true
 *     }
 *   }
 * }
 * }</pre>
 */
public class MecabKoTokenizerFactory extends AbstractTokenizerFactory {

    private static final Logger logger = LogManager.getLogger(MecabKoTokenizerFactory.class);

    private final DecompoundMode decompoundMode;
    private final Path userDictionaryPath;
    private final Set<String> stoptags;
    private final boolean outputUnknownUnigrams;

    /**
     * Create tokenizer factory.
     *
     * @param indexSettings index settings
     * @param env environment
     * @param name tokenizer name
     * @param settings tokenizer settings
     */
    public MecabKoTokenizerFactory(IndexSettings indexSettings,
                                    Environment env,
                                    String name,
                                    Settings settings) {
        super(indexSettings, settings, name);

        // Parse decompound mode
        String decompoundModeStr = settings.get("decompound_mode", "none");
        this.decompoundMode = DecompoundMode.fromString(decompoundModeStr);

        // Parse user dictionary path
        String userDictStr = settings.get("user_dictionary");
        if (userDictStr != null && !userDictStr.isEmpty()) {
            Path dictPath = env.configFile().resolve(userDictStr);
            if (dictPath.toFile().exists()) {
                this.userDictionaryPath = dictPath;
            } else {
                logger.warn("User dictionary not found: {}", dictPath);
                this.userDictionaryPath = null;
            }
        } else {
            this.userDictionaryPath = null;
        }

        // Parse stoptags
        List<String> stoptagsList = settings.getAsList("stoptags");
        if (!stoptagsList.isEmpty()) {
            this.stoptags = new HashSet<>(stoptagsList);
        } else {
            this.stoptags = null;
        }

        // Parse output_unknown_unigrams
        this.outputUnknownUnigrams = settings.getAsBoolean("output_unknown_unigrams", false);

        logger.info("Created MeCab-Ko tokenizer factory '{}' with mode={}, userDict={}, stoptags={}, unknownUnigrams={}",
                    name, decompoundMode,
                    userDictionaryPath != null ? userDictionaryPath.toString() : "none",
                    stoptags != null ? stoptags : "none",
                    outputUnknownUnigrams);
    }

    @Override
    public Tokenizer create() {
        return new MecabKoTokenizer(
            decompoundMode,
            userDictionaryPath,
            stoptags,
            outputUnknownUnigrams
        );
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
     * @return set of stoptags or null
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
