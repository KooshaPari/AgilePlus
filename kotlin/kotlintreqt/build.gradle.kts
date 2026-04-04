plugins {
    id 'java-library'
    id 'maven-publish'
}

group = 'com.phenotype'
version = '0.1.0'

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
    withSourcesJar()
    withJavadocJar()
}

repositories {
    mavenCentral()
}

dependencies {
    compileOnly 'org.junit.jupiter:junit-jupiter:5.10.0'
}

publishing {
    publications {
        maven(MavenPublication) {
            from components.java
        }
    }
}
