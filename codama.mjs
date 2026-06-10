import { readFileSync } from 'node:fs';
import { parse as parseToml } from '@iarna/toml';

const cargo = parseToml(readFileSync('Cargo.toml', 'utf-8'));
const nightly = cargo?.workspace?.metadata?.toolchains?.format;
const prettierOptions = JSON.parse(readFileSync('clients/js/.prettierrc.json', 'utf-8'));

export default {
  idl: 'idl.json',
  before: [
    {
      from: 'codama#updateProgramsVisitor',
      args: [{ solanaFeatureGateProgram: { name: 'featureGate' } }],
    },
  ],
  scripts: {
    js: {
      from: '@codama/renderers-js',
      args: ['clients/js', { kitImportStrategy: 'rootOnly', prettierOptions }],
    },
    rust: {
      from: '@codama/renderers-rust',
      args: [
        'clients/rust',
        {
          anchorTraits: false,
          formatCode: true,
          toolchain: nightly ? `+${nightly}` : '',
        },
      ],
    },
  },
};
