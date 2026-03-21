plugins {
    id("java")
}

val elasticsearchVersion = "8.11.3"
val luceneVersion = "9.8.0"

dependencies {
    // Common module
    implementation(project(":common"))

    // Elasticsearch
    compileOnly("org.elasticsearch:elasticsearch:${elasticsearchVersion}")
    compileOnly("org.elasticsearch.plugin:elasticsearch-plugin-analysis-api:${elasticsearchVersion}")

    // Lucene 9.x (for Elasticsearch 8.x)
    compileOnly("org.apache.lucene:lucene-core:${luceneVersion}")
    compileOnly("org.apache.lucene:lucene-analysis-common:${luceneVersion}")

    // Logging
    compileOnly("org.apache.logging.log4j:log4j-api:2.21.1")

    // Testing
    testImplementation("junit:junit:4.13.2")
    testImplementation("org.hamcrest:hamcrest:2.2")
    testRuntimeOnly("org.apache.logging.log4j:log4j-core:2.21.1")
}

tasks.jar {
    manifest {
        attributes(
            "Implementation-Title" to "MeCab-Ko Elasticsearch Plugin",
            "Implementation-Version" to project.version,
            "Automatic-Module-Name" to "com.mecab.ko.elasticsearch"
        )
    }
}

// Bundle plugin for distribution
tasks.register<Zip>("bundlePlugin") {
    dependsOn("jar")

    from(tasks.named("jar"))
    from(project(":common").tasks.named("jar"))
    from(configurations.runtimeClasspath) {
        into("lib")
    }

    // Include native libraries from all platforms
    from("${rootProject.projectDir}/native") {
        into("native")
    }

    // Include plugin descriptor
    from("src/main/resources") {
        include("plugin-descriptor.properties")
        include("plugin-security.policy")
    }

    archiveBaseName.set("mecab-ko-analyzer")
    archiveVersion.set(project.version.toString())
    destinationDirectory.set(file("${buildDir}/distributions"))
}

// Copy native libraries for testing
tasks.register<Copy>("copyNativeLibsForTest") {
    from("${rootProject.projectDir}/native") {
        include("**/*.so")
        include("**/*.dylib")
        include("**/*.dll")
    }
    into("${buildDir}/test-native")
}

tasks.test {
    dependsOn("copyNativeLibsForTest")
    systemProperty("java.library.path", "${buildDir}/test-native")
}

tasks.named("assemble") {
    dependsOn("bundlePlugin")
}
