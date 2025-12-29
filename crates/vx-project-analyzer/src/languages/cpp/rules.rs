//! C++ script detection rules

use crate::languages::rules::ScriptRule;

/// Rules for detecting common C++ project scripts
pub static CPP_RULES: &[ScriptRule] = &[
    // CMake build
    ScriptRule {
        name: "configure",
        command: "cmake -B build -S .",
        description: "Configure CMake project",
        trigger_files: &["CMakeLists.txt"],
        exclude_if_exists: &[],
        priority: 50,
    },
    ScriptRule {
        name: "build",
        command: "cmake --build build",
        description: "Build project",
        trigger_files: &["CMakeLists.txt"],
        exclude_if_exists: &[],
        priority: 50,
    },
    ScriptRule {
        name: "build:release",
        command: "cmake --build build --config Release",
        description: "Build project in release mode",
        trigger_files: &["CMakeLists.txt"],
        exclude_if_exists: &[],
        priority: 50,
    },
    ScriptRule {
        name: "clean",
        command: "cmake --build build --target clean",
        description: "Clean build artifacts",
        trigger_files: &["CMakeLists.txt"],
        exclude_if_exists: &[],
        priority: 50,
    },
    ScriptRule {
        name: "test",
        command: "ctest --test-dir build",
        description: "Run tests",
        trigger_files: &["CMakeLists.txt"],
        exclude_if_exists: &[],
        priority: 50,
    },
    ScriptRule {
        name: "install",
        command: "cmake --install build",
        description: "Install project",
        trigger_files: &["CMakeLists.txt"],
        exclude_if_exists: &[],
        priority: 50,
    },
    // Makefile (only if no CMakeLists.txt)
    ScriptRule {
        name: "build",
        command: "make",
        description: "Build project",
        trigger_files: &["Makefile"],
        exclude_if_exists: &["CMakeLists.txt"],
        priority: 40,
    },
    ScriptRule {
        name: "clean",
        command: "make clean",
        description: "Clean build artifacts",
        trigger_files: &["Makefile"],
        exclude_if_exists: &["CMakeLists.txt"],
        priority: 40,
    },
    ScriptRule {
        name: "test",
        command: "make test",
        description: "Run tests",
        trigger_files: &["Makefile"],
        exclude_if_exists: &["CMakeLists.txt"],
        priority: 40,
    },
    // Meson
    ScriptRule {
        name: "configure",
        command: "meson setup build",
        description: "Configure Meson project",
        trigger_files: &["meson.build"],
        exclude_if_exists: &[],
        priority: 50,
    },
    ScriptRule {
        name: "build",
        command: "meson compile -C build",
        description: "Build project",
        trigger_files: &["meson.build"],
        exclude_if_exists: &[],
        priority: 50,
    },
    ScriptRule {
        name: "test",
        command: "meson test -C build",
        description: "Run tests",
        trigger_files: &["meson.build"],
        exclude_if_exists: &[],
        priority: 50,
    },
];
