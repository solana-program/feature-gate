#!/usr/bin/env zx
import 'zx/globals';
import { workingDirectory, getProgramFolders } from '../utils.mjs';

// Save external programs binaries to the output directory.
import './dump.mjs';

// Configure additional build args here, ie:
// ['--arg1', '--arg2', ...process.argv.slice(3)]
const buildArgs = process.argv.slice(3);

// Build the programs.
await Promise.all(
  getProgramFolders().map(async (folder) => {
    const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

    await $`cargo-build-sbf --manifest-path ${manifestPath} ${buildArgs}`;
  })
);
