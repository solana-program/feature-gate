#!/usr/bin/env zx
import 'zx/globals';
import {
    getRustfmtToolchain,
    getToolchainArg,
    processFormatAndLintArgs,
    workingDirectory,
} from '../utils.mjs';

const { fix, args } = processFormatAndLintArgs();
const rustFmtArgs = args;

const manifestPath = path.join(workingDirectory, 'clients', 'rust', 'Cargo.toml');

// Format the client.
if (fix) {
    await $`cargo ${getToolchainArg(getRustfmtToolchain())} fmt --manifest-path ${manifestPath} -- ${rustFmtArgs}`;
} else {
    await $`cargo ${getToolchainArg(getRustfmtToolchain())} fmt --manifest-path ${manifestPath} -- --check ${rustFmtArgs}`;
}
