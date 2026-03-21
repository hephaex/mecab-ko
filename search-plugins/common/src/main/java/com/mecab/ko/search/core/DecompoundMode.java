package com.mecab.ko.search.core;

/**
 * Decompound mode for compound nouns.
 *
 * <p>Compatible with Lucene Nori's decompound modes.
 *
 * <ul>
 *   <li>NONE - Keep compound nouns as-is</li>
 *   <li>DISCARD - Output only decomposed morphemes</li>
 *   <li>MIXED - Output both compound and decomposed forms</li>
 * </ul>
 */
public enum DecompoundMode {
    /**
     * No decompounding - keep original compound nouns.
     *
     * <p>Example: "형태소분석기" -> ["형태소분석기/NNG"]
     */
    NONE,

    /**
     * Discard original - output only decomposed morphemes.
     *
     * <p>Example: "형태소분석기" -> ["형태소/NNG", "분석/NNG", "기/NNG"]
     */
    DISCARD,

    /**
     * Mixed mode - output both original and decomposed forms.
     *
     * <p>Example: "형태소분석기" -> ["형태소분석기/NNG", "형태소/NNG", "분석/NNG", "기/NNG"]
     */
    MIXED;

    /**
     * Parse decompound mode from string.
     *
     * @param mode mode string (case-insensitive)
     * @return decompound mode
     * @throws IllegalArgumentException if mode is invalid
     */
    public static DecompoundMode fromString(String mode) {
        if (mode == null || mode.isEmpty()) {
            return NONE;
        }

        switch (mode.toLowerCase()) {
            case "none":
                return NONE;
            case "discard":
                return DISCARD;
            case "mixed":
                return MIXED;
            default:
                throw new IllegalArgumentException("Invalid decompound mode: " + mode);
        }
    }

    /**
     * Convert to config string.
     *
     * @return lowercase mode string
     */
    public String toConfigString() {
        return name().toLowerCase();
    }
}
