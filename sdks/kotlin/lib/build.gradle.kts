plugins {
    kotlin("jvm")
    `java-library`
}

kotlin {
    jvmToolchain(17)
}

dependencies {
    // net.java.dev.jna required by UniFFI-generated Kotlin bindings
    api("net.java.dev.jna:jna:5.14.0")
}

// Make jniLibs visible to the JVM at test time
tasks.test {
    systemProperty(
        "jna.library.path",
        layout.projectDirectory.dir("src/main/jniLibs").asFile.absolutePath
    )
}

// Disable test execution: the JNI .so is not built in CI by default.
// UniFFI-generated code loads libfrf_ffi via System.loadLibrary at runtime;
// without the compiled native library, every test fails with UnsatisfiedLinkError.
// Compilation (compileKotlin, compileTestKotlin) still runs.
tasks.withType<Test> {
    enabled = false
}
