package com.mecab.ko.search.config;

import com.mecab.ko.search.core.DecompoundMode;

import java.nio.file.Path;
import java.util.Arrays;
import java.util.Collections;
import java.util.HashSet;
import java.util.Objects;
import java.util.Set;

/**
 * Configuration for MeCab-Ko analyzer.
 *
 * <p>Immutable configuration object that can be shared between
 * tokenizer and filter instances.
 */
public final class AnalyzerConfig {

    /**
     * Default stoptags for POS filtering.
     */
    public static final Set<String> DEFAULT_STOPTAGS = Collections.unmodifiableSet(
        new HashSet<>(Arrays.asList("J", "E"))
    );

    private final DecompoundMode decompoundMode;
    private final Path userDictionaryPath;
    private final Set<String> stoptags;
    private final boolean outputUnknownUnigrams;

    private AnalyzerConfig(Builder builder) {
        this.decompoundMode = builder.decompoundMode;
        this.userDictionaryPath = builder.userDictionaryPath;
        this.stoptags = builder.stoptags != null
            ? Collections.unmodifiableSet(new HashSet<>(builder.stoptags))
            : null;
        this.outputUnknownUnigrams = builder.outputUnknownUnigrams;
    }

    /**
     * Get decompound mode.
     *
     * @return decompound mode
     */
    public DecompoundMode getDecompoundMode() {
        return decompoundMode;
    }

    /**
     * Get user dictionary path.
     *
     * @return user dictionary path or null
     */
    public Path getUserDictionaryPath() {
        return userDictionaryPath;
    }

    /**
     * Get stoptags.
     *
     * @return set of stoptags or null
     */
    public Set<String> getStoptags() {
        return stoptags;
    }

    /**
     * Check if outputting unknown unigrams.
     *
     * @return true if enabled
     */
    public boolean isOutputUnknownUnigrams() {
        return outputUnknownUnigrams;
    }

    /**
     * Convert to JSON string for native library.
     *
     * @return JSON configuration string
     */
    public String toJson() {
        StringBuilder json = new StringBuilder();
        json.append("{");
        json.append("\"decompound_mode\":\"").append(decompoundMode.toConfigString()).append("\"");

        if (userDictionaryPath != null) {
            json.append(",\"user_dictionary_path\":\"")
                .append(escapeJson(userDictionaryPath.toString()))
                .append("\"");
        }

        json.append(",\"stoptags\":[");
        if (stoptags != null && !stoptags.isEmpty()) {
            boolean first = true;
            for (String tag : stoptags) {
                if (!first) json.append(",");
                json.append("\"").append(escapeJson(tag)).append("\"");
                first = false;
            }
        }
        json.append("]");

        json.append(",\"output_unknown_unigrams\":").append(outputUnknownUnigrams);
        json.append("}");

        return json.toString();
    }

    private String escapeJson(String str) {
        return str.replace("\\", "\\\\")
                  .replace("\"", "\\\"")
                  .replace("\n", "\\n")
                  .replace("\r", "\\r")
                  .replace("\t", "\\t");
    }

    /**
     * Create default configuration.
     *
     * @return default configuration
     */
    public static AnalyzerConfig defaultConfig() {
        return new Builder().build();
    }

    /**
     * Create new builder.
     *
     * @return new builder
     */
    public static Builder builder() {
        return new Builder();
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        AnalyzerConfig that = (AnalyzerConfig) o;
        return outputUnknownUnigrams == that.outputUnknownUnigrams &&
               decompoundMode == that.decompoundMode &&
               Objects.equals(userDictionaryPath, that.userDictionaryPath) &&
               Objects.equals(stoptags, that.stoptags);
    }

    @Override
    public int hashCode() {
        return Objects.hash(decompoundMode, userDictionaryPath, stoptags, outputUnknownUnigrams);
    }

    @Override
    public String toString() {
        return "AnalyzerConfig{" +
               "decompoundMode=" + decompoundMode +
               ", userDictionaryPath=" + userDictionaryPath +
               ", stoptags=" + stoptags +
               ", outputUnknownUnigrams=" + outputUnknownUnigrams +
               '}';
    }

    /**
     * Builder for AnalyzerConfig.
     */
    public static class Builder {
        private DecompoundMode decompoundMode = DecompoundMode.NONE;
        private Path userDictionaryPath;
        private Set<String> stoptags;
        private boolean outputUnknownUnigrams = false;

        /**
         * Set decompound mode.
         *
         * @param decompoundMode decompound mode
         * @return this builder
         */
        public Builder decompoundMode(DecompoundMode decompoundMode) {
            this.decompoundMode = decompoundMode != null ? decompoundMode : DecompoundMode.NONE;
            return this;
        }

        /**
         * Set decompound mode from string.
         *
         * @param mode mode string
         * @return this builder
         */
        public Builder decompoundMode(String mode) {
            this.decompoundMode = DecompoundMode.fromString(mode);
            return this;
        }

        /**
         * Set user dictionary path.
         *
         * @param path path to user dictionary
         * @return this builder
         */
        public Builder userDictionaryPath(Path path) {
            this.userDictionaryPath = path;
            return this;
        }

        /**
         * Set stoptags.
         *
         * @param stoptags set of POS tags to filter
         * @return this builder
         */
        public Builder stoptags(Set<String> stoptags) {
            this.stoptags = stoptags;
            return this;
        }

        /**
         * Set stoptags from array.
         *
         * @param stoptags array of POS tags to filter
         * @return this builder
         */
        public Builder stoptags(String... stoptags) {
            if (stoptags != null) {
                this.stoptags = new HashSet<>(Arrays.asList(stoptags));
            }
            return this;
        }

        /**
         * Use default stoptags.
         *
         * @return this builder
         */
        public Builder useDefaultStoptags() {
            this.stoptags = new HashSet<>(DEFAULT_STOPTAGS);
            return this;
        }

        /**
         * Set output unknown unigrams.
         *
         * @param outputUnknownUnigrams whether to output unknown words as unigrams
         * @return this builder
         */
        public Builder outputUnknownUnigrams(boolean outputUnknownUnigrams) {
            this.outputUnknownUnigrams = outputUnknownUnigrams;
            return this;
        }

        /**
         * Build configuration.
         *
         * @return analyzer configuration
         */
        public AnalyzerConfig build() {
            return new AnalyzerConfig(this);
        }
    }
}
