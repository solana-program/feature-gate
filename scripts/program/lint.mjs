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
    await $`cd ${path.join(workingDirectory, folder)}`.quiet();
    await $`cargo ${getToolchainArg(getClippyToolchain())} clippy ${process.argv.slice(3)}`;
  })
);
