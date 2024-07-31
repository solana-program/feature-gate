/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import { containsBytes, getU8Encoder, type Address } from '@solana/web3.js';
import { type ParsedRevokePendingActivationInstruction } from '../instructions';

export const SOLANA_FEATURE_GATE_PROGRAM_PROGRAM_ADDRESS =
  'Feature111111111111111111111111111111111111' as Address<'Feature111111111111111111111111111111111111'>;

export enum SolanaFeatureGateProgramInstruction {
  RevokePendingActivation,
}

export function identifySolanaFeatureGateProgramInstruction(
  instruction: { data: Uint8Array } | Uint8Array
): SolanaFeatureGateProgramInstruction {
  const data =
    instruction instanceof Uint8Array ? instruction : instruction.data;
  if (containsBytes(data, getU8Encoder().encode(0), 0)) {
    return SolanaFeatureGateProgramInstruction.RevokePendingActivation;
  }
  throw new Error(
    'The provided instruction could not be identified as a solanaFeatureGateProgram instruction.'
  );
}

export type ParsedSolanaFeatureGateProgramInstruction<
  TProgram extends string = 'Feature111111111111111111111111111111111111',
> = {
  instructionType: SolanaFeatureGateProgramInstruction.RevokePendingActivation;
} & ParsedRevokePendingActivationInstruction<TProgram>;
