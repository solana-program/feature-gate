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
    const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');
    await $`cargo ${getToolchainArg(getRustfmtToolchain())} fmt --manifest-path ${manifestPath} ${process.argv.slice(3)}`;
  })
);
