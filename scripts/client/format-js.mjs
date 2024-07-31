#!/usr/bin/env zx
import 'zx/globals';
import { workingDirectory } from '../utils.mjs';

// Format the client using Prettier.
cd(path.join(workingDirectory, 'clients', 'js'));
await $`pnpm install`;
await $`pnpm format ${process.argv.slice(3)}`;
