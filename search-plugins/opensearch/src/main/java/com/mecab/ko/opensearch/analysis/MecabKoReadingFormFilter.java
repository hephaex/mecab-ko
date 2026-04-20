package com.mecab.ko.opensearch.analysis;

import com.mecab.ko.search.core.ReadingFormAttribute;
import org.apache.lucene.analysis.TokenFilter;
import org.apache.lucene.analysis.TokenStream;
import org.apache.lucene.analysis.tokenattributes.CharTermAttribute;
import org.apache.lucene.analysis.tokenattributes.KeywordAttribute;

import java.io.IOException;

/**
 * Token filter that converts Korean text to reading form (pronunciation)
 * for OpenSearch/Lucene 10.x.
 *
 * <p>This filter replaces the surface form of a token with its reading form
 * (pronunciation) when one is available. Reading forms are produced by the
 * native MeCab-Ko analyzer during tokenization and carried through the token
 * stream via {@link ReadingFormAttribute}.
 *
 * <p>Common use cases include converting Hanja (한자) to their Korean
 * pronunciation. Tokens that do not have a reading form, or whose reading
 * form matches the surface form, are left unchanged.
 */
public class MecabKoReadingFormFilter extends TokenFilter {

    private final CharTermAttribute termAttribute = addAttribute(CharTermAttribute.class);
    private final KeywordAttribute keywordAttribute = addAttribute(KeywordAttribute.class);
    private final ReadingFormAttribute readingAttribute = addAttribute(ReadingFormAttribute.class);

    /**
     * Create reading form filter.
     *
     * @param input input token stream
     */
    public MecabKoReadingFormFilter(TokenStream input) {
        super(input);
    }

    @Override
    public boolean incrementToken() throws IOException {
        if (!input.incrementToken()) {
            return false;
        }

        // Skip if marked as keyword
        if (keywordAttribute.isKeyword()) {
            return true;
        }

        // Get reading form from the attribute set by the tokenizer
        String reading = readingAttribute.getReading();

        // Replace with reading form if it differs from the surface form
        if (reading != null && !reading.isEmpty()) {
            String term = termAttribute.toString();
            if (!reading.equals(term)) {
                termAttribute.setEmpty().append(reading);
            }
        }

        return true;
    }
}
