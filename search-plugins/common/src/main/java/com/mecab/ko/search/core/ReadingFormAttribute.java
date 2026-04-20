package com.mecab.ko.search.core;

import org.apache.lucene.util.Attribute;

/**
 * Lucene attribute that carries the reading form (pronunciation) for a token.
 *
 * <p>The reading form is set during tokenization by MecabKoTokenizer when the
 * native analyzer returns a reading for a token (e.g., Hanja to Korean pronunciation).
 * Downstream filters such as MecabKoReadingFormFilter consume this attribute
 * to replace the surface form with the reading form.
 */
public interface ReadingFormAttribute extends Attribute {

    /**
     * Get the reading form.
     *
     * @return reading form or null if not available
     */
    String getReading();

    /**
     * Set the reading form.
     *
     * @param reading reading form (nullable)
     */
    void setReading(String reading);
}
