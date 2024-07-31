#!/usr/bin/env zx
import 'zx/globals';
import {
  workingDirectory,
  getRustfmtToolchain,
  getProgramFolders,
  getToolchainArg,
  processFormatAndLintArgs,
} from '../utils.mjs';

const { fix, args } = processFormatAndLintArgs();
// Configure additional rustfmt args here, ie:
// ['--arg1', '--arg2', ...args]
const rustFmtArgs = args;

// Format the programs.
await Promise.all(
  getProgramFolders().map(async (folder) => {    
    const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

    if (fix) {
      await $`cargo ${getToolchainArg(getRustfmtToolchain())} fmt --manifest-path ${manifestPath} -- ${rustFmtArgs}`;
    } else {
      await $`cargo ${getToolchainArg(getRustfmtToolchain())} fmt --manifest-path ${manifestPath} -- --check ${rustFmtArgs}`;
    }
  })
);
