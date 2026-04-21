package com.mecab.ko.elasticsearch.analysis;

import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.apache.lucene.analysis.TokenFilter;
import org.apache.lucene.analysis.TokenStream;
import org.elasticsearch.common.settings.Settings;
import org.elasticsearch.env.Environment;
import org.elasticsearch.index.IndexSettings;
import org.elasticsearch.index.analysis.AbstractTokenFilterFactory;

import java.util.Arrays;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

/**
 * Factory for MeCab-Ko token filters in Elasticsearch.
 *
 * <p>Supports two filter types:
 * <ul>
 *   <li>PART_OF_SPEECH - Filter tokens by POS tags</li>
 *   <li>READING_FORM - Convert tokens to reading form (한글 발음)</li>
 * </ul>
 *
 * <p>Example configuration for POS filter:
 * <pre>{@code
 * {
 *   "filter": {
 *     "my_pos_filter": {
 *       "type": "mecab_ko_part_of_speech",
 *       "stoptags": ["J", "E", "SF"]
 *     }
 *   }
 * }
 * }</pre>
 *
 * <p>Example configuration for reading form filter:
 * <pre>{@code
 * {
 *   "filter": {
 *     "my_reading_filter": {
 *       "type": "mecab_ko_reading_form"
 *     }
 *   }
 * }
 * }</pre>
 */
public class MecabKoTokenFilterFactory extends AbstractTokenFilterFactory {

    private static final Logger logger = LogManager.getLogger(MecabKoTokenFilterFactory.class);

    private static final String[] DEFAULT_STOPTAGS = {"J", "E"};

    /**
     * Filter type enumeration.
     */
    public enum FilterType {
        PART_OF_SPEECH,
        READING_FORM
    }

    private final FilterType filterType;
    private final Set<String> stoptags;

    /**
     * Create token filter factory.
     *
     * @param indexSettings index settings
     * @param env environment
     * @param name filter name
     * @param settings filter settings
     * @param filterType type of filter to create
     */
    public MecabKoTokenFilterFactory(IndexSettings indexSettings,
                                      Environment env,
                                      String name,
                                      Settings settings,
                                      FilterType filterType) {
        super(name, settings);
        this.filterType = filterType;

        if (filterType == FilterType.PART_OF_SPEECH) {
            // Parse stoptags for POS filter
            List<String> stoptagsList = settings.getAsList("stoptags", Arrays.asList(DEFAULT_STOPTAGS));
            this.stoptags = new HashSet<>(stoptagsList);
            logger.info("Created MeCab-Ko POS filter '{}' with stoptags={}",
                        name, stoptags);
        } else {
            this.stoptags = null;
            logger.info("Created MeCab-Ko reading form filter '{}'", name);
        }
    }

    @Override
    public TokenFilter create(TokenStream tokenStream) {
        switch (filterType) {
            case PART_OF_SPEECH:
                return new MecabKoPartOfSpeechStopFilter(tokenStream, stoptags);
            case READING_FORM:
                return new MecabKoReadingFormFilter(tokenStream);
            default:
                throw new IllegalStateException("Unknown filter type: " + filterType);
        }
    }

    /**
     * Get filter type.
     *
     * @return filter type
     */
    public FilterType getFilterType() {
        return filterType;
    }

    /**
     * Get stoptags (only for POS filter).
     *
     * @return set of stoptags or null
     */
    public Set<String> getStoptags() {
        return stoptags;
    }
}
