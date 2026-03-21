rootProject.name = "mecab-ko-search-plugins"

include("common")
include("elasticsearch")
include("opensearch")

// Set project paths
project(":common").projectDir = file("common")
project(":elasticsearch").projectDir = file("elasticsearch")
project(":opensearch").projectDir = file("opensearch")

// Dependency resolution management
dependencyResolutionManagement {
    versionCatalogs {
        create("libs") {
            version("lucene9", "9.8.0")
            version("lucene10", "10.0.0")
            version("elasticsearch", "8.11.3")
            version("opensearch", "3.0.0")
            version("log4j", "2.21.1")
            version("jackson", "2.15.3")
            version("jna", "5.13.0")
            version("junit", "4.13.2")
            version("junit5", "5.10.0")
            version("hamcrest", "2.2")

            library("jackson-core", "com.fasterxml.jackson.core", "jackson-core").versionRef("jackson")
            library("jackson-databind", "com.fasterxml.jackson.core", "jackson-databind").versionRef("jackson")
            library("jna", "net.java.dev.jna", "jna").versionRef("jna")
            library("log4j-api", "org.apache.logging.log4j", "log4j-api").versionRef("log4j")
            library("log4j-core", "org.apache.logging.log4j", "log4j-core").versionRef("log4j")
            library("junit", "junit", "junit").versionRef("junit")
            library("hamcrest", "org.hamcrest", "hamcrest").versionRef("hamcrest")
        }
    }
}
