#!/usr/bin/env zx
import 'zx/globals';
import {
  workingDirectory,
  getClippyToolchain,
  getProgramFolders,
  getToolchainArg,
  processFormatAndLintArgs,
} from '../utils.mjs';

const { fix, args } = processFormatAndLintArgs();
// Configure additional clippy args here, ie:
// ['--arg1', '--arg2', ...args]
const clippyArgs = args;

// Lint the programs using clippy.
await Promise.all(
  getProgramFolders().map(async (folder) => {
    const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

    if (fix) {
      await $`cargo ${getToolchainArg(getClippyToolchain())} clippy --manifest-path ${manifestPath} --fix ${clippyArgs}`;
    } else {
      await $`cargo ${getToolchainArg(getClippyToolchain())} clippy --manifest-path ${manifestPath} ${clippyArgs}`;
    }
  })
);
