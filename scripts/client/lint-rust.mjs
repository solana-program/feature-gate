#!/usr/bin/env zx
import 'zx/globals';
import {
    getClippyToolchain,
    getToolchainArg,
    processFormatAndLintArgs,
    workingDirectory,
} from '../utils.mjs';

const { fix, args } = processFormatAndLintArgs();
// Configure additional clippy args here, ie:
// ['--arg1', '--arg2', ...args]
const clippyArgs = args;

// Check the client using Clippy.
const manifestPath = path.join(workingDirectory, 'clients', 'rust', 'Cargo.toml');

if (fix) {
    await $`cargo ${getToolchainArg(getClippyToolchain())} clippy --manifest-path ${manifestPath} --fix ${clippyArgs}`;
} else {
    await $`cargo ${getToolchainArg(getClippyToolchain())} clippy --manifest-path ${manifestPath} ${clippyArgs}`;
}
