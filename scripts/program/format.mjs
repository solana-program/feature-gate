#!/usr/bin/env zx
import 'zx/globals';
import {
  workingDirectory,
  getRustfmtToolchain,
  getProgramFolders,
  getToolchainArg,
} from '../utils.mjs';

// Format the programs.
await Promise.all(
  getProgramFolders().map(async (folder) => {
    await $`cd ${path.join(workingDirectory, folder)}`.quiet();
    await $`cargo ${getToolchainArg(getRustfmtToolchain())} fmt ${process.argv.slice(3)}`;
  })
);
