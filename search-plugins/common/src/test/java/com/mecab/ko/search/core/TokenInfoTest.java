package com.mecab.ko.search.core;

import org.junit.Test;
import static org.junit.Assert.*;

/**
 * Tests for TokenInfo class.
 */
public class TokenInfoTest {

    @Test
    public void testBasicConstructor() {
        TokenInfo token = new TokenInfo("한국어", "NNG", 0, 3);

        assertEquals("한국어", token.getSurface());
        assertEquals("NNG", token.getPosTag());
        assertEquals(0, token.getStartOffset());
        assertEquals(3, token.getEndOffset());
        assertNull(token.getReading());
        assertEquals(1, token.getPositionIncrement());
        assertFalse(token.hasReading());
    }

    @Test
    public void testFullConstructor() {
        TokenInfo token = new TokenInfo("韓国語", "NNG", 0, 3, "한국어", 1);

        assertEquals("韓国語", token.getSurface());
        assertEquals("NNG", token.getPosTag());
        assertEquals(0, token.getStartOffset());
        assertEquals(3, token.getEndOffset());
        assertEquals("한국어", token.getReading());
        assertEquals(1, token.getPositionIncrement());
        assertTrue(token.hasReading());
    }

    @Test
    public void testBuilder() {
        TokenInfo token = new TokenInfo.Builder()
            .surface("테스트")
            .posTag("NNG")
            .startOffset(0)
            .endOffset(3)
            .reading("테스트")
            .positionIncrement(2)
            .build();

        assertEquals("테스트", token.getSurface());
        assertEquals("NNG", token.getPosTag());
        assertEquals(0, token.getStartOffset());
        assertEquals(3, token.getEndOffset());
        assertEquals("테스트", token.getReading());
        assertEquals(2, token.getPositionIncrement());
    }

    @Test
    public void testEquals() {
        TokenInfo token1 = new TokenInfo("한국어", "NNG", 0, 3);
        TokenInfo token2 = new TokenInfo("한국어", "NNG", 0, 3);
        TokenInfo token3 = new TokenInfo("영어", "NNG", 0, 2);

        assertEquals(token1, token2);
        assertNotEquals(token1, token3);
    }

    @Test
    public void testHashCode() {
        TokenInfo token1 = new TokenInfo("한국어", "NNG", 0, 3);
        TokenInfo token2 = new TokenInfo("한국어", "NNG", 0, 3);

        assertEquals(token1.hashCode(), token2.hashCode());
    }

    @Test
    public void testToString() {
        TokenInfo token = new TokenInfo("한국어", "NNG", 0, 3);
        String str = token.toString();

        assertTrue(str.contains("한국어"));
        assertTrue(str.contains("NNG"));
        assertTrue(str.contains("0-3"));
    }

    @Test(expected = NullPointerException.class)
    public void testNullSurface() {
        new TokenInfo(null, "NNG", 0, 3);
    }

    @Test(expected = NullPointerException.class)
    public void testNullPosTag() {
        new TokenInfo("한국어", null, 0, 3);
    }
}
