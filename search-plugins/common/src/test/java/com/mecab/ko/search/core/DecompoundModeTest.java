package com.mecab.ko.search.core;

import org.junit.Test;
import static org.junit.Assert.*;

/**
 * Tests for DecompoundMode enum.
 */
public class DecompoundModeTest {

    @Test
    public void testFromString_none() {
        assertEquals(DecompoundMode.NONE, DecompoundMode.fromString("none"));
        assertEquals(DecompoundMode.NONE, DecompoundMode.fromString("NONE"));
        assertEquals(DecompoundMode.NONE, DecompoundMode.fromString("None"));
    }

    @Test
    public void testFromString_discard() {
        assertEquals(DecompoundMode.DISCARD, DecompoundMode.fromString("discard"));
        assertEquals(DecompoundMode.DISCARD, DecompoundMode.fromString("DISCARD"));
        assertEquals(DecompoundMode.DISCARD, DecompoundMode.fromString("Discard"));
    }

    @Test
    public void testFromString_mixed() {
        assertEquals(DecompoundMode.MIXED, DecompoundMode.fromString("mixed"));
        assertEquals(DecompoundMode.MIXED, DecompoundMode.fromString("MIXED"));
        assertEquals(DecompoundMode.MIXED, DecompoundMode.fromString("Mixed"));
    }

    @Test
    public void testFromString_null() {
        assertEquals(DecompoundMode.NONE, DecompoundMode.fromString(null));
    }

    @Test
    public void testFromString_empty() {
        assertEquals(DecompoundMode.NONE, DecompoundMode.fromString(""));
    }

    @Test(expected = IllegalArgumentException.class)
    public void testFromString_invalid() {
        DecompoundMode.fromString("invalid");
    }

    @Test
    public void testToConfigString() {
        assertEquals("none", DecompoundMode.NONE.toConfigString());
        assertEquals("discard", DecompoundMode.DISCARD.toConfigString());
        assertEquals("mixed", DecompoundMode.MIXED.toConfigString());
    }
}
