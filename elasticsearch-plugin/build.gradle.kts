import org.elasticsearch.gradle.plugin.PluginBuildPlugin

plugins {
    id("java")
    id("elasticsearch.esplugin") version "8.11.3"
}

group = "com.mecab.ko"
version = "0.1.0"

esplugin {
    name = "mecab-ko-analyzer"
    description = "MeCab-Ko Korean Morphological Analyzer for Elasticsearch"
    classname = "com.mecab.ko.elasticsearch.plugin.MecabKoPlugin"
    licenseFile.set(rootProject.file("LICENSE"))
    noticeFile.set(rootProject.file("NOTICE"))
}

repositories {
    mavenCentral()
    maven {
        url = uri("https://artifacts.elastic.co/maven")
    }
}

dependencies {
    // Elasticsearch
    compileOnly("org.elasticsearch:elasticsearch:8.11.3")
    compileOnly("org.elasticsearch.plugin:elasticsearch-plugin-analysis-api:8.11.3")

    // Lucene
    compileOnly("org.apache.lucene:lucene-core:9.8.0")
    compileOnly("org.apache.lucene:lucene-analysis-common:9.8.0")
    compileOnly("org.apache.lucene:lucene-analysis-nori:9.8.0")

    // Logging
    compileOnly("org.apache.logging.log4j:log4j-api:2.21.1")
    compileOnly("org.apache.logging.log4j:log4j-core:2.21.1")

    // JNI
    implementation("net.java.dev.jna:jna:5.13.0")

    // Testing
    testImplementation("org.elasticsearch.test:framework:8.11.3")
    testImplementation("junit:junit:4.13.2")
    testImplementation("org.hamcrest:hamcrest:2.2")
}

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}

tasks.withType<JavaCompile> {
    options.encoding = "UTF-8"
    options.compilerArgs.add("-Xlint:deprecation")
    options.compilerArgs.add("-Xlint:unchecked")
}

tasks.test {
    useJUnit()
    testLogging {
        events("passed", "skipped", "failed")
        showStandardStreams = true
    }
}

// Native library integration
tasks.register<Copy>("copyNativeLibs") {
    from("${rootProject.projectDir}/rust/target/release") {
        include("libmecab_ko_elasticsearch.so")
        include("libmecab_ko_elasticsearch.dylib")
        include("mecab_ko_elasticsearch.dll")
    }
    into("${buildDir}/resources/main/native")
}

tasks.named("processResources") {
    dependsOn("copyNativeLibs")
}

// Bundling task
tasks.register<Zip>("bundlePlugin") {
    dependsOn("assemble")
    from(tasks.named("jar"))
    from(configurations.runtimeClasspath) {
        into("lib")
    }
    from("${buildDir}/resources/main/native") {
        into("native")
    }
    archiveBaseName.set("mecab-ko-analyzer")
    archiveVersion.set(project.version.toString())
}

tasks.named("assemble") {
    dependsOn("bundlePlugin")
}

// Integration test configuration
sourceSets {
    create("integTest") {
        java {
            srcDir("src/integTest/java")
            compileClasspath += sourceSets["main"].output + configurations["testRuntimeClasspath"]
            runtimeClasspath += output + compileClasspath
        }
    }
}

val integrationTest = tasks.register<Test>("integrationTest") {
    description = "Runs integration tests"
    group = "verification"
    testClassesDirs = sourceSets["integTest"].output.classesDirs
    classpath = sourceSets["integTest"].runtimeClasspath
    shouldRunAfter("test")
    useJUnit()
}

tasks.check {
    dependsOn(integrationTest)
}

// JaCoCo code coverage
plugins.apply("jacoco")

jacoco {
    toolVersion = "0.8.11"
}

tasks.jacocoTestReport {
    dependsOn(tasks.test)
    reports {
        xml.required.set(true)
        html.required.set(true)
        csv.required.set(false)
    }
}

tasks.jacocoTestCoverageVerification {
    violationRules {
        rule {
            limit {
                minimum = "0.5".toBigDecimal()
            }
        }
    }
}

// Test reporting
tasks.withType<Test> {
    reports {
        html.required.set(true)
        junitXml.required.set(true)
    }

    // Show test output
    testLogging {
        events("passed", "skipped", "failed", "standardOut", "standardError")
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
        showExceptions = true
        showCauses = true
        showStackTraces = true
        showStandardStreams = true
    }

    // Fail fast on first test failure
    failFast = false

    // Set max parallel forks
    maxParallelForks = (Runtime.getRuntime().availableProcessors() / 2).takeIf { it > 0 } ?: 1
}

tasks.named<Test>("integrationTest") {
    reports {
        html.outputLocation.set(file("${buildDir}/reports/tests/integrationTest"))
        junitXml.outputLocation.set(file("${buildDir}/test-results/integrationTest"))
    }

    // Integration test specific settings
    systemProperty("es.set.netty.runtime.available.processors", "false")
    systemProperty("tests.security.manager", "false")
}

// Custom task for running specific test groups
tasks.register<Test>("quickTest") {
    description = "Runs quick tests (excludes performance and large data tests)"
    group = "verification"
    useJUnit {
        excludeCategories("com.mecab.ko.elasticsearch.test.SlowTest")
    }
}

// Test summary task
tasks.register("testSummary") {
    description = "Prints test summary"
    group = "verification"
    dependsOn(tasks.test, integrationTest)

    doLast {
        val testResults = tasks.test.get().reports.html.outputLocation.get().asFile
        val integTestResults = tasks.named<Test>("integrationTest").get()
            .reports.html.outputLocation.get().asFile

        println("\n=== Test Summary ===")
        println("Unit tests: ${testResults.absolutePath}")
        println("Integration tests: ${integTestResults.absolutePath}")
    }
}
