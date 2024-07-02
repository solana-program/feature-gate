#!/usr/bin/env zx
import 'zx/globals';
import {
  workingDirectory,
  getClippyToolchain,
  getProgramFolders,
  getToolchainArg,
} from '../utils.mjs';

// Lint the programs using clippy.
await Promise.all(
  getProgramFolders().map(async (folder) => {
    const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');
    await $`cargo ${getToolchainArg(getClippyToolchain())} clippy --manifest-path ${manifestPath} ${process.argv.slice(3)}`;
  })
);
