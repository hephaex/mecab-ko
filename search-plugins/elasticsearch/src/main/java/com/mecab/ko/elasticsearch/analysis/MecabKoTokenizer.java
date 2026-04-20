package com.mecab.ko.elasticsearch.analysis;

import com.mecab.ko.search.core.DecompoundMode;
import com.mecab.ko.search.core.MecabKoTokenizerBase;
import com.mecab.ko.search.core.ReadingFormAttribute;
import com.mecab.ko.search.core.TokenInfo;
import org.apache.lucene.analysis.Tokenizer;
import org.apache.lucene.analysis.tokenattributes.CharTermAttribute;
import org.apache.lucene.analysis.tokenattributes.OffsetAttribute;
import org.apache.lucene.analysis.tokenattributes.PositionIncrementAttribute;
import org.apache.lucene.analysis.tokenattributes.TypeAttribute;

import java.io.IOException;
import java.nio.file.Path;
import java.util.Set;

/**
 * MeCab-Ko tokenizer for Elasticsearch/Lucene 9.x.
 *
 * <p>Wraps native MeCab-Ko library via JNI for morphological analysis.
 */
public class MecabKoTokenizer extends Tokenizer {

    private final MecabKoTokenizerDelegate delegate;

    private final CharTermAttribute termAtt = addAttribute(CharTermAttribute.class);
    private final OffsetAttribute offsetAtt = addAttribute(OffsetAttribute.class);
    private final PositionIncrementAttribute posIncrAtt = addAttribute(PositionIncrementAttribute.class);
    private final TypeAttribute typeAtt = addAttribute(TypeAttribute.class);
    private final ReadingFormAttribute readingAtt = addAttribute(ReadingFormAttribute.class);

    /**
     * Create tokenizer with default settings.
     */
    public MecabKoTokenizer() {
        this(DecompoundMode.NONE, null, null, false);
    }

    /**
     * Create tokenizer.
     *
     * @param decompoundMode compound noun handling mode
     * @param userDictionaryPath path to user dictionary (nullable)
     * @param stoptags POS tags to filter (nullable)
     * @param outputUnknownUnigrams whether to output unknown words as unigrams
     */
    public MecabKoTokenizer(DecompoundMode decompoundMode,
                            Path userDictionaryPath,
                            Set<String> stoptags,
                            boolean outputUnknownUnigrams) {
        this.delegate = new MecabKoTokenizerDelegate(
            decompoundMode,
            userDictionaryPath,
            stoptags,
            outputUnknownUnigrams
        );
        delegate.initializeAnalyzer();
    }

    @Override
    public boolean incrementToken() throws IOException {
        clearAttributes();
        return delegate.processNextToken();
    }

    @Override
    public void reset() throws IOException {
        super.reset();
        delegate.resetTokenizer(input);
    }

    @Override
    public void end() throws IOException {
        super.end();
        int finalOffset = delegate.getFinalOffset();
        offsetAtt.setOffset(finalOffset, finalOffset);
    }

    @Override
    public void close() throws IOException {
        super.close();
        delegate.closeTokenizer();
    }

    /**
     * Get decompound mode.
     *
     * @return decompound mode
     */
    public DecompoundMode getDecompoundMode() {
        return delegate.getDecompoundMode();
    }

    /**
     * Get user dictionary path.
     *
     * @return user dictionary path or null
     */
    public Path getUserDictionaryPath() {
        return delegate.getUserDictionaryPath();
    }

    /**
     * Internal delegate that handles tokenization logic.
     */
    private class MecabKoTokenizerDelegate extends MecabKoTokenizerBase {

        MecabKoTokenizerDelegate(DecompoundMode decompoundMode,
                                  Path userDictionaryPath,
                                  Set<String> stoptags,
                                  boolean outputUnknownUnigrams) {
            super(decompoundMode, userDictionaryPath, stoptags, outputUnknownUnigrams);
        }

        @Override
        protected boolean populateAttributes(TokenInfo token) {
            termAtt.setEmpty().append(token.getSurface());
            offsetAtt.setOffset(token.getStartOffset(), token.getEndOffset());
            posIncrAtt.setPositionIncrement(token.getPositionIncrement());
            typeAtt.setType(token.getPosTag());
            readingAtt.setReading(token.getReading());
            return true;
        }
    }
}
