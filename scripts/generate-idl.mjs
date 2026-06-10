#!/usr/bin/env zx
import 'zx/globals';
import { generateIdl } from '@metaplex-foundation/shank-js';
import { cliArguments, getCargo, getProgramFolders } from './utils.mjs';

const binaryInstallDir = path.join(__dirname, '..', '.cargo');
const rootDir = path.join(__dirname, '..');
const [folder] = cliArguments();

const cargo = getCargo(folder);
const isShank = Object.keys(cargo.dependencies).includes('shank');
const programDir = path.join(__dirname, '..', folder);

generateIdl({
  generator: isShank ? 'shank' : 'anchor',
  programName: cargo.package.name.replace(/-/g, '_'),
  programId: cargo.package.metadata.solana['program-id'],
  idlDir: rootDir,
  idlName: 'idl',
  programDir,
  binaryInstallDir,
});
