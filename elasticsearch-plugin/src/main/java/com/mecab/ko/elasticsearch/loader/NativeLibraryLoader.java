package com.mecab.ko.elasticsearch.loader;

import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;

/**
 * Native library loader for MeCab-Ko JNI bindings.
 *
 * Loads platform-specific native libraries from plugin resources.
 */
public class NativeLibraryLoader {

    private static final Logger logger = LogManager.getLogger(NativeLibraryLoader.class);
    private static boolean loaded = false;
    private static final Object lock = new Object();

    /**
     * Load the native library.
     * Thread-safe singleton loading.
     *
     * @throws UnsatisfiedLinkError if library cannot be loaded
     */
    public static void load() {
        synchronized (lock) {
            if (loaded) {
                return;
            }

            try {
                String libraryName = getLibraryName();
                String libraryPath = "/native/" + libraryName;

                logger.info("Loading native library: {}", libraryName);

                // Extract library to temp directory
                Path tempDir = Files.createTempDirectory("mecab-ko-native-");
                Path tempLibrary = tempDir.resolve(libraryName);

                try (InputStream is = NativeLibraryLoader.class.getResourceAsStream(libraryPath)) {
                    if (is == null) {
                        throw new UnsatisfiedLinkError(
                            "Native library not found in resources: " + libraryPath
                        );
                    }

                    Files.copy(is, tempLibrary, StandardCopyOption.REPLACE_EXISTING);

                    // Make executable on Unix systems
                    if (!System.getProperty("os.name").toLowerCase().contains("win")) {
                        tempLibrary.toFile().setExecutable(true);
                    }
                }

                // Load library
                System.load(tempLibrary.toAbsolutePath().toString());

                // Schedule cleanup on JVM exit
                Runtime.getRuntime().addShutdownHook(new Thread(() -> {
                    try {
                        Files.deleteIfExists(tempLibrary);
                        Files.deleteIfExists(tempDir);
                    } catch (IOException e) {
                        logger.warn("Failed to cleanup temporary library", e);
                    }
                }));

                loaded = true;
                logger.info("Successfully loaded native library: {}", libraryName);

            } catch (IOException e) {
                throw new UnsatisfiedLinkError("Failed to extract native library: " + e.getMessage());
            }
        }
    }

    /**
     * Get platform-specific library name.
     *
     * @return library filename
     */
    private static String getLibraryName() {
        String os = System.getProperty("os.name").toLowerCase();
        String arch = System.getProperty("os.arch").toLowerCase();

        // Normalize architecture names
        String normalizedArch = normalizeArch(arch);

        if (os.contains("win")) {
            return "mecab_ko_elasticsearch.dll";
        } else if (os.contains("mac") || os.contains("darwin")) {
            return "libmecab_ko_elasticsearch.dylib";
        } else if (os.contains("linux")) {
            return "libmecab_ko_elasticsearch.so";
        } else {
            throw new UnsatisfiedLinkError("Unsupported operating system: " + os);
        }
    }

    /**
     * Normalize architecture names.
     *
     * @param arch raw architecture string
     * @return normalized architecture
     */
    private static String normalizeArch(String arch) {
        if (arch.equals("x86_64") || arch.equals("amd64")) {
            return "x86_64";
        } else if (arch.equals("aarch64") || arch.equals("arm64")) {
            return "aarch64";
        } else {
            return arch;
        }
    }

    /**
     * Check if native library is loaded.
     *
     * @return true if loaded
     */
    public static boolean isLoaded() {
        synchronized (lock) {
            return loaded;
        }
    }

    /**
     * Get library version from native code.
     *
     * @return version string
     */
    public static native String getVersion();

    /**
     * Test if native library is working.
     *
     * @return true if working
     */
    public static boolean test() {
        try {
            load();
            String version = getVersion();
            logger.info("Native library version: {}", version);
            return version != null && !version.isEmpty();
        } catch (UnsatisfiedLinkError e) {
            logger.error("Native library test failed", e);
            return false;
        }
    }
}
