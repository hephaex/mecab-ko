package com.mecab.ko.search.jni;

/**
 * JNI wrapper for native MeCab-Ko analyzer.
 *
 * <p>Provides access to the Rust-based mecab-ko-elasticsearch library.
 */
public final class NativeAnalyzer {

    static {
        NativeLibraryLoader.load();
    }

    private NativeAnalyzer() {
        // Prevent instantiation
    }

    /**
     * Create a new analyzer instance.
     *
     * @param configJson JSON configuration string containing:
     *                   - decompound_mode: "none"|"discard"|"mixed"
     *                   - user_dictionary_path: path to user dictionary (optional)
     *                   - stoptags: array of POS tags to filter (optional)
     *                   - output_unknown_unigrams: boolean (optional)
     * @return analyzer handle (opaque pointer), or 0 on failure
     */
    public static native long createAnalyzer(String configJson);

    /**
     * Analyze text and return JSON array of tokens.
     *
     * @param handle analyzer handle
     * @param text text to analyze
     * @return JSON array of tokens, each containing:
     *         - surface: surface form
     *         - pos_tag: POS tag
     *         - start_offset: start character offset
     *         - end_offset: end character offset
     *         - reading: reading form (optional)
     */
    public static native String analyzeText(long handle, String text);

    /**
     * Destroy analyzer instance and free resources.
     *
     * @param handle analyzer handle
     */
    public static native void destroyAnalyzer(long handle);

    /**
     * Get native library version.
     *
     * @return version string
     */
    public static native String getVersion();

    /**
     * Check if native library is available.
     *
     * @return true if native library is loaded and functional
     */
    public static boolean isAvailable() {
        try {
            String version = getVersion();
            return version != null && !version.isEmpty();
        } catch (UnsatisfiedLinkError e) {
            return false;
        }
    }

    /**
     * Get dictionary path configured in native library.
     *
     * @return dictionary path or null if not configured
     */
    public static native String getDictionaryPath();

    /**
     * Set dictionary path in native library.
     *
     * @param path path to dictionary directory
     * @return true on success
     */
    public static native boolean setDictionaryPath(String path);
}
