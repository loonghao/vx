#!/usr/bin/env node
/**
 * Project Info Extension - Main Entry Point
 *
 * This extension demonstrates a Node.js-based vx extension.
 *
 * Usage:
 *     vx x project-info              # Run main entry point
 *     vx x project-info deps         # Show dependencies
 *     vx x project-info size         # Show project size
 */

const fs = require("fs");
const path = require("path");

function main() {
  const projectDir = process.env.VX_PROJECT_DIR || process.cwd();

  console.log("📊 Project Information");
  console.log("=".repeat(50));
  console.log();

  // Check for package.json
  const packageJsonPath = path.join(projectDir, "package.json");
  if (fs.existsSync(packageJsonPath)) {
    const pkg = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
    console.log("📦 Node.js Project:");
    console.log(`  Name:        ${pkg.name || "N/A"}`);
    console.log(`  Version:     ${pkg.version || "N/A"}`);
    console.log(`  Description: ${pkg.description || "N/A"}`);
    console.log();
  }

  // Check for Cargo.toml
  const cargoTomlPath = path.join(projectDir, "Cargo.toml");
  if (fs.existsSync(cargoTomlPath)) {
    console.log("🦀 Rust Project detected (Cargo.toml found)");
    console.log();
  }

  // Check for pyproject.toml
  const pyprojectPath = path.join(projectDir, "pyproject.toml");
  if (fs.existsSync(pyprojectPath)) {
    console.log("🐍 Python Project detected (pyproject.toml found)");
    console.log();
  }

  // Check for go.mod
  const goModPath = path.join(projectDir, "go.mod");
  if (fs.existsSync(goModPath)) {
    console.log("🐹 Go Project detected (go.mod found)");
    console.log();
  }

  // VX info
  console.log("🔧 VX Environment:");
  console.log(`  VX_VERSION:        ${process.env.VX_VERSION || "N/A"}`);
  console.log(`  VX_EXTENSION_NAME: ${process.env.VX_EXTENSION_NAME || "N/A"}`);
  console.log(`  VX_PROJECT_DIR:    ${projectDir}`);
  console.log();

  console.log("Available commands:");
  console.log("  vx x project-info deps  - Show project dependencies");
  console.log("  vx x project-info size  - Show project size statistics");
}

main();
