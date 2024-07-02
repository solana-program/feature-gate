#!/usr/bin/env zx
import 'zx/globals';
import { getClippyToolchain, getToolchainArg, workingDirectory } from '../utils.mjs';

// Check the client using Clippy.
const manifestPath = path.join(workingDirectory, 'clients', 'rust', 'Cargo.toml');
await $`cargo ${getToolchainArg(getClippyToolchain())} clippy --manifest-path ${manifestPath} ${process.argv.slice(3)}`;
