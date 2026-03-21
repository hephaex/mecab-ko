package com.mecab.ko.search.core;

import java.util.Objects;

/**
 * Token information from the native MeCab-Ko analyzer.
 *
 * <p>Contains the surface form, POS tag, offsets, and optional reading form.
 */
public final class TokenInfo {

    private final String surface;
    private final String posTag;
    private final int startOffset;
    private final int endOffset;
    private final String reading;
    private final int positionIncrement;

    /**
     * Create token info.
     *
     * @param surface surface form
     * @param posTag POS tag
     * @param startOffset start offset in original text
     * @param endOffset end offset in original text
     */
    public TokenInfo(String surface, String posTag, int startOffset, int endOffset) {
        this(surface, posTag, startOffset, endOffset, null, 1);
    }

    /**
     * Create token info with all fields.
     *
     * @param surface surface form
     * @param posTag POS tag
     * @param startOffset start offset in original text
     * @param endOffset end offset in original text
     * @param reading reading form (nullable)
     * @param positionIncrement position increment
     */
    public TokenInfo(String surface, String posTag, int startOffset, int endOffset,
                     String reading, int positionIncrement) {
        this.surface = Objects.requireNonNull(surface, "surface cannot be null");
        this.posTag = Objects.requireNonNull(posTag, "posTag cannot be null");
        this.startOffset = startOffset;
        this.endOffset = endOffset;
        this.reading = reading;
        this.positionIncrement = positionIncrement;
    }

    /**
     * Get surface form.
     *
     * @return surface form
     */
    public String getSurface() {
        return surface;
    }

    /**
     * Get POS tag.
     *
     * @return POS tag
     */
    public String getPosTag() {
        return posTag;
    }

    /**
     * Get start offset.
     *
     * @return start offset
     */
    public int getStartOffset() {
        return startOffset;
    }

    /**
     * Get end offset.
     *
     * @return end offset
     */
    public int getEndOffset() {
        return endOffset;
    }

    /**
     * Get reading form.
     *
     * @return reading form or null
     */
    public String getReading() {
        return reading;
    }

    /**
     * Get position increment.
     *
     * @return position increment
     */
    public int getPositionIncrement() {
        return positionIncrement;
    }

    /**
     * Check if this token has a reading form.
     *
     * @return true if reading form exists
     */
    public boolean hasReading() {
        return reading != null && !reading.isEmpty();
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        TokenInfo tokenInfo = (TokenInfo) o;
        return startOffset == tokenInfo.startOffset &&
               endOffset == tokenInfo.endOffset &&
               positionIncrement == tokenInfo.positionIncrement &&
               Objects.equals(surface, tokenInfo.surface) &&
               Objects.equals(posTag, tokenInfo.posTag) &&
               Objects.equals(reading, tokenInfo.reading);
    }

    @Override
    public int hashCode() {
        return Objects.hash(surface, posTag, startOffset, endOffset, reading, positionIncrement);
    }

    @Override
    public String toString() {
        StringBuilder sb = new StringBuilder();
        sb.append("TokenInfo{surface='").append(surface).append('\'');
        sb.append(", posTag='").append(posTag).append('\'');
        sb.append(", offset=").append(startOffset).append("-").append(endOffset);
        if (reading != null) {
            sb.append(", reading='").append(reading).append('\'');
        }
        if (positionIncrement != 1) {
            sb.append(", posInc=").append(positionIncrement);
        }
        sb.append('}');
        return sb.toString();
    }

    /**
     * Builder for TokenInfo.
     */
    public static class Builder {
        private String surface;
        private String posTag;
        private int startOffset;
        private int endOffset;
        private String reading;
        private int positionIncrement = 1;

        public Builder surface(String surface) {
            this.surface = surface;
            return this;
        }

        public Builder posTag(String posTag) {
            this.posTag = posTag;
            return this;
        }

        public Builder startOffset(int startOffset) {
            this.startOffset = startOffset;
            return this;
        }

        public Builder endOffset(int endOffset) {
            this.endOffset = endOffset;
            return this;
        }

        public Builder reading(String reading) {
            this.reading = reading;
            return this;
        }

        public Builder positionIncrement(int positionIncrement) {
            this.positionIncrement = positionIncrement;
            return this;
        }

        public TokenInfo build() {
            return new TokenInfo(surface, posTag, startOffset, endOffset, reading, positionIncrement);
        }
    }
}
