#!/usr/bin/env zx
import 'zx/globals';
import { getRustfmtToolchain, getToolchainArg, workingDirectory } from '../utils.mjs';

// Format the client.
const manifestPath = path.join(workingDirectory, 'clients', 'rust', 'Cargo.toml');
await $`cargo ${getToolchainArg(getRustfmtToolchain())} fmt --manifest-path ${manifestPath} ${process.argv.slice(3)}`;
