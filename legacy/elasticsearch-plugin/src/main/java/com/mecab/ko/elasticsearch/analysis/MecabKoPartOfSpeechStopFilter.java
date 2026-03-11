package com.mecab.ko.elasticsearch.analysis;

import org.apache.lucene.analysis.FilteringTokenFilter;
import org.apache.lucene.analysis.TokenStream;
import org.apache.lucene.analysis.tokenattributes.TypeAttribute;

import java.io.IOException;
import java.util.HashSet;
import java.util.Set;

/**
 * Token filter that removes tokens based on POS tags.
 *
 * <p>Filters out tokens whose POS tag starts with any of the configured stop tags.
 * Common stoptags:
 * <ul>
 *   <li>J - Josa (조사)</li>
 *   <li>E - Eomi (어미)</li>
 *   <li>SF - Final punctuation (마침표/물음표/느낌표)</li>
 *   <li>SP - Separator punctuation (쉼표/가운뎃점)</li>
 *   <li>SSC - Closing bracket (닫는 괄호)</li>
 *   <li>SSO - Opening bracket (여는 괄호)</li>
 *   <li>SC - Separator (구분자)</li>
 * </ul>
 */
public class MecabKoPartOfSpeechStopFilter extends FilteringTokenFilter {

    private final Set<String> stopTags;
    private final TypeAttribute typeAttribute = addAttribute(TypeAttribute.class);

    /**
     * Create POS stop filter.
     *
     * @param input input token stream
     * @param stopTags array of POS tags to filter
     */
    public MecabKoPartOfSpeechStopFilter(TokenStream input, String[] stopTags) {
        super(input);
        this.stopTags = new HashSet<>();
        if (stopTags != null) {
            for (String tag : stopTags) {
                this.stopTags.add(tag.toUpperCase());
            }
        }
    }

    @Override
    protected boolean accept() throws IOException {
        String posTag = typeAttribute.type();

        if (posTag == null || posTag.isEmpty()) {
            return true; // Keep tokens without POS tags
        }

        // Check if POS tag starts with any stop tag
        for (String stopTag : stopTags) {
            if (posTag.startsWith(stopTag)) {
                return false; // Filter out this token
            }
        }

        return true; // Keep this token
    }

    /**
     * Get configured stop tags.
     *
     * @return set of stop tags
     */
    public Set<String> getStopTags() {
        return new HashSet<>(stopTags);
    }
}
