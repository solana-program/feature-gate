#!/usr/bin/env zx
import {
    getRustfmtToolchain,
    getClippyToolchain,
} from '../utils.mjs';

const rustfmtToolchain = getRustfmtToolchain();
const clippyToolchain = getClippyToolchain();

await $`echo "RUSTFMT_NIGHTLY_VERSION=${rustfmtToolchain}" >> $GITHUB_ENV`;
await $`echo "CLIPPY_NIGHTLY_VERSION=${clippyToolchain}" >> $GITHUB_ENV`;
