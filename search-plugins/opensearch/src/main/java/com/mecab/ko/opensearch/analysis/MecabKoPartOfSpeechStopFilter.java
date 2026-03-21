package com.mecab.ko.opensearch.analysis;

import com.mecab.ko.search.core.MecabKoFilterBase;
import org.apache.lucene.analysis.FilteringTokenFilter;
import org.apache.lucene.analysis.TokenStream;
import org.apache.lucene.analysis.tokenattributes.TypeAttribute;

import java.io.IOException;
import java.util.Set;

/**
 * Token filter that removes tokens based on POS tags for OpenSearch/Lucene 10.x.
 *
 * <p>Filters out tokens whose POS tag starts with any of the configured stop tags.
 *
 * <p>Common stoptags:
 * <ul>
 *   <li>J - Josa (조사)</li>
 *   <li>E - Eomi (어미)</li>
 *   <li>SF - Final punctuation (마침표/물음표/느낌표)</li>
 *   <li>SP - Separator punctuation (쉼표/가운뎃점)</li>
 *   <li>SS - Quotes and brackets (따옴표/괄호표)</li>
 *   <li>SE - Ellipsis (줄임표)</li>
 *   <li>SO - Dash (붙임표)</li>
 *   <li>SW - Other symbols (기타 기호)</li>
 * </ul>
 */
public class MecabKoPartOfSpeechStopFilter extends FilteringTokenFilter {

    private final MecabKoFilterBase filterBase;
    private final TypeAttribute typeAttribute = addAttribute(TypeAttribute.class);

    /**
     * Create POS stop filter with default stoptags (J, E).
     *
     * @param input input token stream
     */
    public MecabKoPartOfSpeechStopFilter(TokenStream input) {
        this(input, MecabKoFilterBase.DEFAULT_STOPTAGS);
    }

    /**
     * Create POS stop filter with custom stoptags.
     *
     * @param input input token stream
     * @param stopTags set of POS tags to filter
     */
    public MecabKoPartOfSpeechStopFilter(TokenStream input, Set<String> stopTags) {
        super(input);
        this.filterBase = new MecabKoFilterBase(stopTags) {};
    }

    /**
     * Create POS stop filter from array of stoptags.
     *
     * @param input input token stream
     * @param stopTags array of POS tags to filter
     */
    public MecabKoPartOfSpeechStopFilter(TokenStream input, String[] stopTags) {
        super(input);
        this.filterBase = new MecabKoFilterBase(stopTags) {};
    }

    @Override
    protected boolean accept() throws IOException {
        String posTag = typeAttribute.type();
        return !filterBase.shouldFilter(posTag);
    }

    /**
     * Get configured stop tags.
     *
     * @return set of stop tags
     */
    public Set<String> getStopTags() {
        return filterBase.getStopTags();
    }
}
