package com.mecab.ko.search.jni;

import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.Locale;

/**
 * Native library loader for MeCab-Ko JNI bindings.
 *
 * <p>Loads platform-specific native libraries from plugin resources.
 * Supports multiple platforms and architectures:
 * <ul>
 *   <li>linux-x64 (x86_64-unknown-linux-gnu)</li>
 *   <li>linux-arm64 (aarch64-unknown-linux-gnu)</li>
 *   <li>darwin-x64 (x86_64-apple-darwin)</li>
 *   <li>darwin-arm64 (aarch64-apple-darwin)</li>
 *   <li>windows-x64 (x86_64-pc-windows-msvc)</li>
 * </ul>
 */
public final class NativeLibraryLoader {

    private static final Logger logger;
    private static volatile boolean loaded = false;
    private static final Object LOCK = new Object();
    private static String loadedVersion = null;

    static {
        Logger tempLogger;
        try {
            tempLogger = LogManager.getLogger(NativeLibraryLoader.class);
        } catch (NoClassDefFoundError e) {
            // Log4j not available, use null logger
            tempLogger = null;
        }
        logger = tempLogger;
    }

    private NativeLibraryLoader() {
        // Prevent instantiation
    }

    /**
     * Load the native library.
     * Thread-safe singleton loading.
     *
     * @throws UnsatisfiedLinkError if library cannot be loaded
     */
    public static void load() {
        if (loaded) {
            return;
        }

        synchronized (LOCK) {
            if (loaded) {
                return;
            }

            try {
                loadFromResources();
                loaded = true;
            } catch (UnsatisfiedLinkError e) {
                // Try system library path as fallback
                try {
                    loadFromSystem();
                    loaded = true;
                } catch (UnsatisfiedLinkError e2) {
                    logError("Failed to load native library from resources and system", e2);
                    throw e2;
                }
            }
        }
    }

    /**
     * Load library from embedded resources.
     */
    private static void loadFromResources() {
        String platform = detectPlatform();
        String libraryName = getLibraryName();
        String resourcePath = "/native/" + platform + "/" + libraryName;

        logInfo("Loading native library: {} from {}", libraryName, resourcePath);

        try {
            // Extract library to temp directory
            Path tempDir = Files.createTempDirectory("mecab-ko-native-");
            Path tempLibrary = tempDir.resolve(libraryName);

            try (InputStream is = NativeLibraryLoader.class.getResourceAsStream(resourcePath)) {
                if (is == null) {
                    throw new UnsatisfiedLinkError(
                        "Native library not found in resources: " + resourcePath
                    );
                }

                Files.copy(is, tempLibrary, StandardCopyOption.REPLACE_EXISTING);

                // Make executable on Unix systems
                if (!isWindows()) {
                    tempLibrary.toFile().setExecutable(true);
                }
            }

            // Load library
            System.load(tempLibrary.toAbsolutePath().toString());

            // Schedule cleanup on JVM exit
            Path finalTempDir = tempDir;
            Path finalTempLibrary = tempLibrary;
            Runtime.getRuntime().addShutdownHook(new Thread(() -> {
                try {
                    Files.deleteIfExists(finalTempLibrary);
                    Files.deleteIfExists(finalTempDir);
                } catch (IOException e) {
                    // Ignore cleanup errors
                }
            }));

            logInfo("Successfully loaded native library: {}", libraryName);

        } catch (IOException e) {
            throw new UnsatisfiedLinkError("Failed to extract native library: " + e.getMessage());
        }
    }

    /**
     * Load library from system library path.
     */
    private static void loadFromSystem() {
        String libraryName = System.mapLibraryName("mecab_ko_elasticsearch");
        logInfo("Attempting to load from system: {}", libraryName);
        System.loadLibrary("mecab_ko_elasticsearch");
        logInfo("Successfully loaded from system library path");
    }

    /**
     * Detect platform string.
     *
     * @return platform string (e.g., "linux-x64", "darwin-arm64")
     */
    private static String detectPlatform() {
        String os = normalizeOS();
        String arch = normalizeArch();
        return os + "-" + arch;
    }

    /**
     * Normalize OS name.
     */
    private static String normalizeOS() {
        String os = System.getProperty("os.name").toLowerCase(Locale.ROOT);

        if (os.contains("linux")) {
            return "linux";
        } else if (os.contains("mac") || os.contains("darwin")) {
            return "darwin";
        } else if (os.contains("win")) {
            return "windows";
        } else {
            throw new UnsatisfiedLinkError("Unsupported operating system: " + os);
        }
    }

    /**
     * Normalize architecture name.
     */
    private static String normalizeArch() {
        String arch = System.getProperty("os.arch").toLowerCase(Locale.ROOT);

        if (arch.equals("x86_64") || arch.equals("amd64")) {
            return "x64";
        } else if (arch.equals("aarch64") || arch.equals("arm64")) {
            return "arm64";
        } else {
            throw new UnsatisfiedLinkError("Unsupported architecture: " + arch);
        }
    }

    /**
     * Get platform-specific library name.
     *
     * @return library filename
     */
    private static String getLibraryName() {
        String os = normalizeOS();

        switch (os) {
            case "windows":
                return "mecab_ko_elasticsearch.dll";
            case "darwin":
                return "libmecab_ko_elasticsearch.dylib";
            case "linux":
            default:
                return "libmecab_ko_elasticsearch.so";
        }
    }

    /**
     * Check if running on Windows.
     */
    private static boolean isWindows() {
        return System.getProperty("os.name").toLowerCase(Locale.ROOT).contains("win");
    }

    /**
     * Check if native library is loaded.
     *
     * @return true if loaded
     */
    public static boolean isLoaded() {
        return loaded;
    }

    /**
     * Get loaded version.
     *
     * @return version string or null if not loaded
     */
    public static String getLoadedVersion() {
        return loadedVersion;
    }

    /**
     * Get current platform string.
     *
     * @return platform string
     */
    public static String getPlatform() {
        return detectPlatform();
    }

    // Logging helpers that handle missing Log4j

    private static void logInfo(String message, Object... args) {
        if (logger != null) {
            logger.info(message, args);
        }
    }

    private static void logError(String message, Throwable t) {
        if (logger != null) {
            logger.error(message, t);
        }
    }
}
