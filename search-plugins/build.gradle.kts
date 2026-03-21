plugins {
    id("java")
    id("jacoco")
}

allprojects {
    group = "com.mecab.ko"
    version = "0.7.0"

    repositories {
        mavenCentral()
        maven {
            url = uri("https://artifacts.elastic.co/maven")
        }
        maven {
            url = uri("https://aws.oss.sonatype.org/content/repositories/releases")
        }
    }
}

subprojects {
    apply(plugin = "java")
    apply(plugin = "jacoco")

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

    tasks.jacocoTestReport {
        dependsOn(tasks.test)
        reports {
            xml.required.set(true)
            html.required.set(true)
        }
    }
}

// Root build task
tasks.register("buildAll") {
    description = "Build all plugins"
    group = "build"
    dependsOn(":common:build", ":elasticsearch:bundlePlugin", ":opensearch:bundlePlugin")
}

// Clean all
tasks.register("cleanAll") {
    description = "Clean all projects"
    group = "build"
    dependsOn(":common:clean", ":elasticsearch:clean", ":opensearch:clean")
}
