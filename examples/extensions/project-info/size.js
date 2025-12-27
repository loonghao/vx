#!/usr/bin/env node
/**
 * Size subcommand - shows project size statistics.
 *
 * Usage:
 *     vx x project-info size
 */

const fs = require("fs");
const path = require("path");

// Directories to ignore
const IGNORE_DIRS = new Set([
  "node_modules",
  ".git",
  "target",
  "dist",
  "build",
  ".venv",
  "__pycache__",
  ".next",
  ".nuxt",
]);

function formatSize(bytes) {
  const units = ["B", "KB", "MB", "GB"];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`;
}

function getDirectoryStats(dir, stats = { files: 0, dirs: 0, size: 0 }) {
  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);

      if (entry.isDirectory()) {
        if (!IGNORE_DIRS.has(entry.name)) {
          stats.dirs++;
          getDirectoryStats(fullPath, stats);
        }
      } else if (entry.isFile()) {
        stats.files++;
        try {
          const fileStat = fs.statSync(fullPath);
          stats.size += fileStat.size;
        } catch {
          // Ignore files we can't stat
        }
      }
    }
  } catch {
    // Ignore directories we can't read
  }

  return stats;
}

function main() {
  const projectDir = process.env.VX_PROJECT_DIR || process.cwd();

  console.log("📊 Project Size Statistics");
  console.log("=".repeat(50));
  console.log();
  console.log(`Project: ${projectDir}`);
  console.log();

  const stats = getDirectoryStats(projectDir);

  console.log("Statistics (excluding node_modules, .git, target, etc.):");
  console.log(`  Files:       ${stats.files.toLocaleString()}`);
  console.log(`  Directories: ${stats.dirs.toLocaleString()}`);
  console.log(`  Total Size:  ${formatSize(stats.size)}`);
  console.log();

  // Check for large directories
  console.log("Ignored directories:");
  for (const dir of IGNORE_DIRS) {
    const dirPath = path.join(projectDir, dir);
    if (fs.existsSync(dirPath)) {
      console.log(`  ✓ ${dir}`);
    }
  }
}

main();
