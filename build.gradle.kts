plugins {
    java
    id("me.champeau.gradle.jmh") version "0.5.0"
}

group = "dev.evo"
version = "0.1-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    jmh("org.openjdk.jmh", "jmh-core", "1.22")
    testCompile("junit", "junit", "4.12")
}

configure<JavaPluginConvention> {
    sourceCompatibility = JavaVersion.VERSION_1_8
}

jmh {
    warmupIterations = 1
    fork = 1
    iterations = 4
    timeOnIteration = "2s"
}
