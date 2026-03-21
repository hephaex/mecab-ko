package com.mecab.ko.search.core;

import org.junit.Test;
import static org.junit.Assert.*;

import java.util.Arrays;
import java.util.HashSet;
import java.util.Set;

/**
 * Tests for MecabKoFilterBase class.
 */
public class MecabKoFilterBaseTest {

    @Test
    public void testDefaultStoptags() {
        Set<String> defaultTags = MecabKoFilterBase.DEFAULT_STOPTAGS;

        assertTrue(defaultTags.contains("J"));
        assertTrue(defaultTags.contains("E"));
        assertEquals(2, defaultTags.size());
    }

    @Test
    public void testShouldFilter_josa() {
        MecabKoFilterBase filter = new MecabKoFilterBase() {};

        // Josa tags should be filtered
        assertTrue(filter.shouldFilter("J"));
        assertTrue(filter.shouldFilter("JKS"));
        assertTrue(filter.shouldFilter("JKC"));
        assertTrue(filter.shouldFilter("JKG"));
        assertTrue(filter.shouldFilter("JKO"));
        assertTrue(filter.shouldFilter("JKB"));
        assertTrue(filter.shouldFilter("JX"));
    }

    @Test
    public void testShouldFilter_eomi() {
        MecabKoFilterBase filter = new MecabKoFilterBase() {};

        // Eomi tags should be filtered
        assertTrue(filter.shouldFilter("E"));
        assertTrue(filter.shouldFilter("EP"));
        assertTrue(filter.shouldFilter("EF"));
        assertTrue(filter.shouldFilter("EC"));
        assertTrue(filter.shouldFilter("ETN"));
        assertTrue(filter.shouldFilter("ETM"));
    }

    @Test
    public void testShouldFilter_nouns() {
        MecabKoFilterBase filter = new MecabKoFilterBase() {};

        // Nouns should not be filtered
        assertFalse(filter.shouldFilter("NNG"));
        assertFalse(filter.shouldFilter("NNP"));
        assertFalse(filter.shouldFilter("NNB"));
        assertFalse(filter.shouldFilter("NR"));
        assertFalse(filter.shouldFilter("NP"));
    }

    @Test
    public void testShouldFilter_verbs() {
        MecabKoFilterBase filter = new MecabKoFilterBase() {};

        // Verbs should not be filtered
        assertFalse(filter.shouldFilter("VV"));
        assertFalse(filter.shouldFilter("VA"));
        assertFalse(filter.shouldFilter("VX"));
    }

    @Test
    public void testShouldFilter_null() {
        MecabKoFilterBase filter = new MecabKoFilterBase() {};
        assertFalse(filter.shouldFilter(null));
    }

    @Test
    public void testShouldFilter_empty() {
        MecabKoFilterBase filter = new MecabKoFilterBase() {};
        assertFalse(filter.shouldFilter(""));
    }

    @Test
    public void testCustomStoptags_set() {
        Set<String> customTags = new HashSet<>(Arrays.asList("N", "V"));
        MecabKoFilterBase filter = new MecabKoFilterBase(customTags) {};

        // Custom tags should be filtered
        assertTrue(filter.shouldFilter("NNG"));
        assertTrue(filter.shouldFilter("VV"));

        // Default tags should not be filtered
        assertFalse(filter.shouldFilter("JKS"));
        assertFalse(filter.shouldFilter("EF"));
    }

    @Test
    public void testCustomStoptags_array() {
        MecabKoFilterBase filter = new MecabKoFilterBase(new String[]{"SF", "SP"}) {};

        // Custom tags should be filtered
        assertTrue(filter.shouldFilter("SF"));
        assertTrue(filter.shouldFilter("SP"));

        // Default tags should not be filtered
        assertFalse(filter.shouldFilter("J"));
        assertFalse(filter.shouldFilter("E"));
    }

    @Test
    public void testCaseInsensitivity() {
        MecabKoFilterBase filter = new MecabKoFilterBase() {};

        // Should handle different cases
        assertTrue(filter.shouldFilter("j"));
        assertTrue(filter.shouldFilter("J"));
        assertTrue(filter.shouldFilter("jks"));
        assertTrue(filter.shouldFilter("JKS"));
    }

    @Test
    public void testGetStopTags() {
        Set<String> customTags = new HashSet<>(Arrays.asList("X", "Y"));
        MecabKoFilterBase filter = new MecabKoFilterBase(customTags) {};

        Set<String> stopTags = filter.getStopTags();
        assertTrue(stopTags.contains("X"));
        assertTrue(stopTags.contains("Y"));
        assertEquals(2, stopTags.size());
    }
}
