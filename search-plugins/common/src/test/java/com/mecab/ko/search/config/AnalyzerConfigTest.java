package com.mecab.ko.search.config;

import com.mecab.ko.search.core.DecompoundMode;
import org.junit.Test;
import static org.junit.Assert.*;

import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;
import java.util.HashSet;
import java.util.Set;

/**
 * Tests for AnalyzerConfig class.
 */
public class AnalyzerConfigTest {

    @Test
    public void testDefaultConfig() {
        AnalyzerConfig config = AnalyzerConfig.defaultConfig();

        assertEquals(DecompoundMode.NONE, config.getDecompoundMode());
        assertNull(config.getUserDictionaryPath());
        assertNull(config.getStoptags());
        assertFalse(config.isOutputUnknownUnigrams());
    }

    @Test
    public void testBuilder_basic() {
        AnalyzerConfig config = AnalyzerConfig.builder()
            .decompoundMode(DecompoundMode.MIXED)
            .build();

        assertEquals(DecompoundMode.MIXED, config.getDecompoundMode());
    }

    @Test
    public void testBuilder_decompoundModeString() {
        AnalyzerConfig config = AnalyzerConfig.builder()
            .decompoundMode("discard")
            .build();

        assertEquals(DecompoundMode.DISCARD, config.getDecompoundMode());
    }

    @Test
    public void testBuilder_userDictionary() {
        Path dictPath = Paths.get("/path/to/dict.txt");
        AnalyzerConfig config = AnalyzerConfig.builder()
            .userDictionaryPath(dictPath)
            .build();

        assertEquals(dictPath, config.getUserDictionaryPath());
    }

    @Test
    public void testBuilder_stoptags_set() {
        Set<String> stoptags = new HashSet<>(Arrays.asList("J", "E", "SF"));
        AnalyzerConfig config = AnalyzerConfig.builder()
            .stoptags(stoptags)
            .build();

        assertEquals(stoptags, config.getStoptags());
    }

    @Test
    public void testBuilder_stoptags_varargs() {
        AnalyzerConfig config = AnalyzerConfig.builder()
            .stoptags("J", "E", "SF")
            .build();

        Set<String> expected = new HashSet<>(Arrays.asList("J", "E", "SF"));
        assertEquals(expected, config.getStoptags());
    }

    @Test
    public void testBuilder_defaultStoptags() {
        AnalyzerConfig config = AnalyzerConfig.builder()
            .useDefaultStoptags()
            .build();

        assertTrue(config.getStoptags().contains("J"));
        assertTrue(config.getStoptags().contains("E"));
    }

    @Test
    public void testBuilder_outputUnknownUnigrams() {
        AnalyzerConfig config = AnalyzerConfig.builder()
            .outputUnknownUnigrams(true)
            .build();

        assertTrue(config.isOutputUnknownUnigrams());
    }

    @Test
    public void testBuilder_full() {
        Path dictPath = Paths.get("/path/to/dict.txt");
        Set<String> stoptags = new HashSet<>(Arrays.asList("J", "E"));

        AnalyzerConfig config = AnalyzerConfig.builder()
            .decompoundMode(DecompoundMode.MIXED)
            .userDictionaryPath(dictPath)
            .stoptags(stoptags)
            .outputUnknownUnigrams(true)
            .build();

        assertEquals(DecompoundMode.MIXED, config.getDecompoundMode());
        assertEquals(dictPath, config.getUserDictionaryPath());
        assertEquals(stoptags, config.getStoptags());
        assertTrue(config.isOutputUnknownUnigrams());
    }

    @Test
    public void testToJson() {
        AnalyzerConfig config = AnalyzerConfig.builder()
            .decompoundMode(DecompoundMode.MIXED)
            .stoptags("J", "E")
            .outputUnknownUnigrams(true)
            .build();

        String json = config.toJson();

        assertTrue(json.contains("\"decompound_mode\":\"mixed\""));
        assertTrue(json.contains("\"stoptags\":["));
        assertTrue(json.contains("\"output_unknown_unigrams\":true"));
    }

    @Test
    public void testToJson_withUserDict() {
        Path dictPath = Paths.get("/path/to/dict.txt");
        AnalyzerConfig config = AnalyzerConfig.builder()
            .userDictionaryPath(dictPath)
            .build();

        String json = config.toJson();
        assertTrue(json.contains("\"user_dictionary_path\":\"/path/to/dict.txt\""));
    }

    @Test
    public void testEquals() {
        AnalyzerConfig config1 = AnalyzerConfig.builder()
            .decompoundMode(DecompoundMode.MIXED)
            .stoptags("J", "E")
            .build();

        AnalyzerConfig config2 = AnalyzerConfig.builder()
            .decompoundMode(DecompoundMode.MIXED)
            .stoptags("J", "E")
            .build();

        assertEquals(config1, config2);
    }

    @Test
    public void testHashCode() {
        AnalyzerConfig config1 = AnalyzerConfig.builder()
            .decompoundMode(DecompoundMode.MIXED)
            .build();

        AnalyzerConfig config2 = AnalyzerConfig.builder()
            .decompoundMode(DecompoundMode.MIXED)
            .build();

        assertEquals(config1.hashCode(), config2.hashCode());
    }

    @Test
    public void testToString() {
        AnalyzerConfig config = AnalyzerConfig.builder()
            .decompoundMode(DecompoundMode.MIXED)
            .build();

        String str = config.toString();
        assertTrue(str.contains("MIXED"));
    }
}
