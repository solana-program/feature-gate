/**
 * This code was AUTOGENERATED using the codama library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun codama to update it.
 *
 * @see https://github.com/codama-idl/codama
 */

import {
  isProgramError,
  type Address,
  type SOLANA_ERROR__INSTRUCTION_ERROR__CUSTOM,
  type SolanaError,
} from '@solana/web3.js';
import { SOLANA_FEATURE_GATE_PROGRAM_ADDRESS } from '../programs';

/** FeatureAlreadyActivated: Feature already activated */
export const SOLANA_FEATURE_GATE_ERROR__FEATURE_ALREADY_ACTIVATED = 0x0; // 0

export type SolanaFeatureGateError =
  typeof SOLANA_FEATURE_GATE_ERROR__FEATURE_ALREADY_ACTIVATED;

let solanaFeatureGateErrorMessages:
  | Record<SolanaFeatureGateError, string>
  | undefined;
if (process.env.NODE_ENV !== 'production') {
  solanaFeatureGateErrorMessages = {
    [SOLANA_FEATURE_GATE_ERROR__FEATURE_ALREADY_ACTIVATED]: `Feature already activated`,
  };
}

export function getSolanaFeatureGateErrorMessage(
  code: SolanaFeatureGateError
): string {
  if (process.env.NODE_ENV !== 'production') {
    return (
      solanaFeatureGateErrorMessages as Record<SolanaFeatureGateError, string>
    )[code];
  }

  return 'Error message not available in production bundles.';
}

export function isSolanaFeatureGateError<
  TProgramErrorCode extends SolanaFeatureGateError,
>(
  error: unknown,
  transactionMessage: {
    instructions: Record<number, { programAddress: Address }>;
  },
  code?: TProgramErrorCode
): error is SolanaError<typeof SOLANA_ERROR__INSTRUCTION_ERROR__CUSTOM> &
  Readonly<{ context: Readonly<{ code: TProgramErrorCode }> }> {
  return isProgramError<TProgramErrorCode>(
    error,
    transactionMessage,
    SOLANA_FEATURE_GATE_PROGRAM_ADDRESS,
    code
  );
}
