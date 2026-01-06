rootProject.name = "mecab-ko-elasticsearch-plugin"

pluginManagement {
    repositories {
        gradlePluginPortal()
        maven {
            url = uri("https://artifacts.elastic.co/maven")
        }
    }
}
