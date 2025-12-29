//! Node.js script detection rules
//!
//! Defines rules for detecting common Node.js scripts based on file presence.

use crate::languages::rules::ScriptRule;

/// All Node.js script detection rules
///
/// Rules are evaluated by priority (highest first).
/// For each script name, only the highest priority matching rule is used.
pub const NODEJS_RULES: &[ScriptRule] = &[
    // =========================================================================
    // Build tools
    // =========================================================================
    ScriptRule::new("build", "npm run build", "Build the project")
        .triggers(&["webpack.config.js", "webpack.config.ts"])
        .priority(80),
    ScriptRule::new("build", "npm run build", "Build the project")
        .triggers(&["vite.config.js", "vite.config.ts"])
        .priority(80),
    ScriptRule::new("build", "npm run build", "Build the project")
        .triggers(&["rollup.config.js", "rollup.config.ts"])
        .priority(80),
    ScriptRule::new("build", "npx tsc", "Compile TypeScript")
        .triggers(&["tsconfig.json"])
        .excludes(&["webpack.config.js", "vite.config.js", "rollup.config.js"])
        .priority(50),
    // =========================================================================
    // Test runners
    // =========================================================================
    ScriptRule::new("test", "npx vitest", "Run tests with Vitest")
        .triggers(&["vitest.config.js", "vitest.config.ts"])
        .priority(100),
    ScriptRule::new("test", "npx jest", "Run tests with Jest")
        .triggers(&["jest.config.js", "jest.config.ts", "jest.config.json"])
        .excludes(&["vitest.config.js", "vitest.config.ts"])
        .priority(90),
    ScriptRule::new("test", "npx mocha", "Run tests with Mocha")
        .triggers(&[".mocharc.js", ".mocharc.json", ".mocharc.yaml"])
        .excludes(&["jest.config.js", "vitest.config.js"])
        .priority(80),
    // =========================================================================
    // Linting
    // =========================================================================
    ScriptRule::new("lint", "npx eslint .", "Run ESLint")
        .triggers(&[
            ".eslintrc",
            ".eslintrc.js",
            ".eslintrc.json",
            ".eslintrc.yaml",
            "eslint.config.js",
            "eslint.config.mjs",
        ])
        .priority(80),
    ScriptRule::new("lint", "npx biome check .", "Run Biome linter")
        .triggers(&["biome.json", "biome.jsonc"])
        .excludes(&[".eslintrc", ".eslintrc.js", "eslint.config.js"])
        .priority(90),
    // =========================================================================
    // Formatting
    // =========================================================================
    ScriptRule::new("format", "npx prettier --write .", "Format with Prettier")
        .triggers(&[
            ".prettierrc",
            ".prettierrc.js",
            ".prettierrc.json",
            "prettier.config.js",
        ])
        .priority(80),
    ScriptRule::new("format", "npx biome format --write .", "Format with Biome")
        .triggers(&["biome.json", "biome.jsonc"])
        .excludes(&[".prettierrc", ".prettierrc.js", "prettier.config.js"])
        .priority(90),
    // =========================================================================
    // Type checking
    // =========================================================================
    ScriptRule::new(
        "typecheck",
        "npx tsc --noEmit",
        "Type check with TypeScript",
    )
    .triggers(&["tsconfig.json"])
    .priority(50),
    // =========================================================================
    // Development server
    // =========================================================================
    ScriptRule::new("dev", "npm run dev", "Start development server")
        .triggers(&["vite.config.js", "vite.config.ts"])
        .priority(80),
    ScriptRule::new("dev", "npx next dev", "Start Next.js dev server")
        .triggers(&["next.config.js", "next.config.mjs", "next.config.ts"])
        .priority(90),
    ScriptRule::new("dev", "npx nuxt dev", "Start Nuxt dev server")
        .triggers(&["nuxt.config.js", "nuxt.config.ts"])
        .priority(90),
];
