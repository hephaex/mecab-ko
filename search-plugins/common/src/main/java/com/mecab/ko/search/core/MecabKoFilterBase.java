package com.mecab.ko.search.core;

import java.util.Collections;
import java.util.HashSet;
import java.util.Set;

/**
 * Base class for MeCab-Ko token filters.
 *
 * <p>Provides common filtering logic for POS-based filtering.
 */
public abstract class MecabKoFilterBase {

    /**
     * Default stoptags (Josa and Eomi).
     */
    public static final Set<String> DEFAULT_STOPTAGS;

    static {
        Set<String> tags = new HashSet<>();
        tags.add("J");   // Josa (조사)
        tags.add("E");   // Eomi (어미)
        DEFAULT_STOPTAGS = Collections.unmodifiableSet(tags);
    }

    protected final Set<String> stopTags;

    /**
     * Create filter base with default stoptags.
     */
    protected MecabKoFilterBase() {
        this(DEFAULT_STOPTAGS);
    }

    /**
     * Create filter base with custom stoptags.
     *
     * @param stopTags set of POS tags to filter
     */
    protected MecabKoFilterBase(Set<String> stopTags) {
        this.stopTags = stopTags != null
            ? Collections.unmodifiableSet(new HashSet<>(stopTags))
            : Collections.emptySet();
    }

    /**
     * Create filter base from array of stoptags.
     *
     * @param stopTags array of POS tags to filter
     */
    protected MecabKoFilterBase(String[] stopTags) {
        if (stopTags != null) {
            Set<String> tags = new HashSet<>();
            for (String tag : stopTags) {
                if (tag != null) {
                    tags.add(tag.toUpperCase());
                }
            }
            this.stopTags = Collections.unmodifiableSet(tags);
        } else {
            this.stopTags = Collections.emptySet();
        }
    }

    /**
     * Check if a POS tag should be filtered.
     *
     * @param posTag POS tag to check
     * @return true if the tag should be filtered out
     */
    protected boolean shouldFilter(String posTag) {
        if (posTag == null || posTag.isEmpty()) {
            return false; // Keep tokens without POS tags
        }

        String upperPosTag = posTag.toUpperCase();
        for (String stopTag : stopTags) {
            if (upperPosTag.startsWith(stopTag)) {
                return true; // Filter out this token
            }
        }

        return false; // Keep this token
    }

    /**
     * Get configured stop tags.
     *
     * @return set of stop tags
     */
    public Set<String> getStopTags() {
        return stopTags;
    }

    /**
     * Common Korean POS tags reference.
     *
     * <p>Major POS categories:
     * <ul>
     *   <li>N - Noun (체언)</li>
     *   <li>V - Verb (용언)</li>
     *   <li>M - Modifier (수식언)</li>
     *   <li>I - Interjection (감탄사)</li>
     *   <li>J - Josa (조사)</li>
     *   <li>E - Eomi (어미)</li>
     *   <li>X - Affix (접사)</li>
     *   <li>S - Symbol/Punctuation (기호)</li>
     * </ul>
     *
     * <p>Common stoptag combinations:
     * <ul>
     *   <li>["J", "E"] - Filter Josa and Eomi (most common)</li>
     *   <li>["J", "E", "S"] - Also filter symbols</li>
     *   <li>["J", "E", "X"] - Also filter affixes</li>
     * </ul>
     */
    public static class POSTags {
        // Noun (체언)
        public static final String NNG = "NNG";  // 일반 명사
        public static final String NNP = "NNP";  // 고유 명사
        public static final String NNB = "NNB";  // 의존 명사
        public static final String NR = "NR";    // 수사
        public static final String NP = "NP";    // 대명사

        // Verb (용언)
        public static final String VV = "VV";    // 동사
        public static final String VA = "VA";    // 형용사
        public static final String VX = "VX";    // 보조 용언
        public static final String VCP = "VCP";  // 긍정 지정사
        public static final String VCN = "VCN";  // 부정 지정사

        // Modifier (수식언)
        public static final String MM = "MM";    // 관형사
        public static final String MAG = "MAG";  // 일반 부사
        public static final String MAJ = "MAJ";  // 접속 부사

        // Josa (조사)
        public static final String JKS = "JKS";  // 주격 조사
        public static final String JKC = "JKC";  // 보격 조사
        public static final String JKG = "JKG";  // 관형격 조사
        public static final String JKO = "JKO";  // 목적격 조사
        public static final String JKB = "JKB";  // 부사격 조사
        public static final String JKV = "JKV";  // 호격 조사
        public static final String JKQ = "JKQ";  // 인용격 조사
        public static final String JX = "JX";    // 보조사
        public static final String JC = "JC";    // 접속 조사

        // Eomi (어미)
        public static final String EP = "EP";    // 선어말 어미
        public static final String EF = "EF";    // 종결 어미
        public static final String EC = "EC";    // 연결 어미
        public static final String ETN = "ETN";  // 명사형 전성 어미
        public static final String ETM = "ETM";  // 관형형 전성 어미

        // Symbol (기호)
        public static final String SF = "SF";    // 마침표, 물음표, 느낌표
        public static final String SP = "SP";    // 쉼표, 가운뎃점, 콜론, 빗금
        public static final String SS = "SS";    // 따옴표, 괄호표, 줄표
        public static final String SE = "SE";    // 줄임표
        public static final String SO = "SO";    // 붙임표 (물결, 숨김, 빠짐)
        public static final String SW = "SW";    // 기타 기호

        private POSTags() {} // Prevent instantiation
    }
}
