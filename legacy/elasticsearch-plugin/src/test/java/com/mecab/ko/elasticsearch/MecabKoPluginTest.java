package com.mecab.ko.elasticsearch;

import com.mecab.ko.elasticsearch.plugin.MecabKoPlugin;
import org.elasticsearch.index.analysis.AnalyzerProvider;
import org.elasticsearch.index.analysis.TokenFilterFactory;
import org.elasticsearch.index.analysis.TokenizerFactory;
import org.elasticsearch.indices.analysis.AnalysisModule;
import org.junit.Before;
import org.junit.Test;

import java.util.Map;

import static org.junit.Assert.*;

/**
 * Unit tests for MecabKoPlugin.
 */
public class MecabKoPluginTest {

    private MecabKoPlugin plugin;

    @Before
    public void setUp() {
        // Note: This will try to load native library
        // Make sure native library is available in test environment
        try {
            plugin = new MecabKoPlugin();
        } catch (RuntimeException e) {
            // Native library not available in test environment
            System.err.println("Warning: Native library not loaded: " + e.getMessage());
            plugin = null;
        }
    }

    @Test
    public void testGetAnalyzers() {
        if (plugin == null) {
            System.out.println("Skipping test - native library not available");
            return;
        }

        Map<String, AnalysisModule.AnalysisProvider<AnalyzerProvider<?>>> analyzers =
            plugin.getAnalyzers();

        assertNotNull("Analyzers map should not be null", analyzers);
        assertTrue("Should register mecab_ko analyzer",
            analyzers.containsKey("mecab_ko"));
        assertTrue("Should register nori analyzer (alias)",
            analyzers.containsKey("nori"));
    }

    @Test
    public void testGetTokenizers() {
        if (plugin == null) {
            System.out.println("Skipping test - native library not available");
            return;
        }

        Map<String, AnalysisModule.AnalysisProvider<TokenizerFactory>> tokenizers =
            plugin.getTokenizers();

        assertNotNull("Tokenizers map should not be null", tokenizers);
        assertTrue("Should register mecab_ko_tokenizer",
            tokenizers.containsKey("mecab_ko_tokenizer"));
        assertTrue("Should register nori_tokenizer (alias)",
            tokenizers.containsKey("nori_tokenizer"));
    }

    @Test
    public void testGetTokenFilters() {
        if (plugin == null) {
            System.out.println("Skipping test - native library not available");
            return;
        }

        Map<String, AnalysisModule.AnalysisProvider<TokenFilterFactory>> filters =
            plugin.getTokenFilters();

        assertNotNull("Token filters map should not be null", filters);
        assertTrue("Should register mecab_ko_part_of_speech",
            filters.containsKey("mecab_ko_part_of_speech"));
        assertTrue("Should register mecab_ko_reading_form",
            filters.containsKey("mecab_ko_reading_form"));
        assertTrue("Should register nori_part_of_speech (alias)",
            filters.containsKey("nori_part_of_speech"));
        assertTrue("Should register nori_reading_form (alias)",
            filters.containsKey("nori_reading_form"));
    }
}
