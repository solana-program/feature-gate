#!/usr/bin/env zx
import 'zx/globals';
import { getClippyToolchain, getToolchainArg, workingDirectory } from '../utils.mjs';

// Check the client using Clippy.
cd(path.join(workingDirectory, 'clients', 'rust'));
await $`cargo ${getToolchainArg(getClippyToolchain())} clippy ${process.argv.slice(3)}`;
