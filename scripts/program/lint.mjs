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
const clippyArgs = [
  '-Zunstable-options',
  '--features',
  'bpf-entrypoint,test-sbf',
  '--',
  '--deny=warnings',
  '--deny=clippy::arithmetic_side_effects',
  ...args
];

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
