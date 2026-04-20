package com.mecab.ko.search.core;

import org.apache.lucene.util.AttributeImpl;
import org.apache.lucene.util.AttributeReflector;

/**
 * Implementation of {@link ReadingFormAttribute}.
 *
 * <p>Stores the reading form (pronunciation) for a token as determined
 * by the native MeCab-Ko analyzer during tokenization.
 */
public class ReadingFormAttributeImpl extends AttributeImpl implements ReadingFormAttribute {

    private String reading;

    @Override
    public String getReading() {
        return reading;
    }

    @Override
    public void setReading(String reading) {
        this.reading = reading;
    }

    @Override
    public void clear() {
        reading = null;
    }

    @Override
    public void copyTo(AttributeImpl target) {
        ((ReadingFormAttribute) target).setReading(reading);
    }

    @Override
    public void reflectWith(AttributeReflector reflector) {
        reflector.reflect(ReadingFormAttribute.class, "reading", reading);
    }
}
