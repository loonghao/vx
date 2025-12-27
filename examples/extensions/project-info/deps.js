#!/usr/bin/env node
/**
 * Dependencies subcommand - shows project dependencies.
 *
 * Usage:
 *     vx x project-info deps
 */

const fs = require("fs");
const path = require("path");

function main() {
  const projectDir = process.env.VX_PROJECT_DIR || process.cwd();

  console.log("📦 Project Dependencies");
  console.log("=".repeat(50));
  console.log();

  // Check for package.json
  const packageJsonPath = path.join(projectDir, "package.json");
  if (fs.existsSync(packageJsonPath)) {
    const pkg = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));

    if (pkg.dependencies) {
      const deps = Object.keys(pkg.dependencies);
      console.log(`Dependencies (${deps.length}):`);
      deps.forEach((dep) => {
        console.log(`  ${dep}: ${pkg.dependencies[dep]}`);
      });
      console.log();
    }

    if (pkg.devDependencies) {
      const devDeps = Object.keys(pkg.devDependencies);
      console.log(`Dev Dependencies (${devDeps.length}):`);
      devDeps.forEach((dep) => {
        console.log(`  ${dep}: ${pkg.devDependencies[dep]}`);
      });
      console.log();
    }

    if (!pkg.dependencies && !pkg.devDependencies) {
      console.log("No dependencies found in package.json");
    }
  } else {
    console.log("No package.json found in project directory.");
    console.log(`Searched in: ${projectDir}`);
  }
}

main();
