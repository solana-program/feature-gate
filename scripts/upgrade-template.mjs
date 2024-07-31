#!/usr/bin/env zx
import "zx/globals";
import { getCargo } from "./utils.mjs";

// Arguments to pass to the `create-solana-program` command.
const rustClientCargo = getCargo(path.join("clients", "rust"));
const jsClientPkg = require(
  path.join(__dirname, "..", "clients", "js", "package.json")
);
const templateArgs = [
  "feature-gate",
  "--address",
  "Feature111111111111111111111111111111111111",
  "--org",
  "solana-program",
  "--rust-client-crate-name",
  rustClientCargo.package.name,
  "--js-client-package-name",
  jsClientPkg.name,
  "--default",
  "--force",
];

// File and folder patterns that should not be overwritten by the template upgrade.
const unchangedGlobs = [
  "clients/**/src/**",
  "clients/**/src/*",
  "program/**/*",
  "program/*",
  "scripts/generate-clients.mjs",
  "scripts/generate-idls.mjs",
  "Cargo.lock",
  "**/pnpm-lock.yaml",
  "pnpm-lock.yaml",
];

// Prevent CLI arguments from being escaped.
$.quote = (command) => command;

// Re-generate the repo from the parent directory.
cd("..");
await $`pnpm create solana-program@latest ${templateArgs}`;

// Go back inside the updated repo.
cd("feature-gate");

// Restore files and folders that should not be overwritten.
await $`git add --all`;
for (const glob of unchangedGlobs) {
  await $`git restore --worktree --staged "${glob}"`;
}

// Re-install dependencies.
await $`pnpm install`;