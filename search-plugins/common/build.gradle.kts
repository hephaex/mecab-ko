plugins {
    id("java-library")
}

dependencies {
    // JSON processing
    api("com.fasterxml.jackson.core:jackson-core:2.15.3")
    api("com.fasterxml.jackson.core:jackson-databind:2.15.3")

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
            "Implementation-Title" to "MeCab-Ko Search Common",
            "Implementation-Version" to project.version,
            "Automatic-Module-Name" to "com.mecab.ko.search.common"
        )
    }
}
