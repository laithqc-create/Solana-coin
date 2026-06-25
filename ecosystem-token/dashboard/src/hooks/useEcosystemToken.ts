'use client';

import { useCallback, useState, useEffect } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import * as anchor from '@project-serum/anchor';
import {
  PublicKey,
  Transaction,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';

// For devnet - update with actual program ID after deployment
const PROGRAM_ID = new PublicKey(
  process.env.NEXT_PUBLIC_PROGRAM_ID ||
    'EcosystemTokenProgramID123456789'
);

interface UserState {
  userTier: 'tier1' | 'tier2' | null;
  stakedAmount: number;
  totalTokensMinted: number;
  pendingYield: number;
  vestingProgress: number;
  isLoading: boolean;
  error: string | null;
}

interface TransactionState {
  mintAmount: number;
  stakeAmount: number;
  isProcessing: boolean;
  txSignature: string | null;
}

export function useEcosystemToken() {
  const { connection } = useConnection();
  const { publicKey, signTransaction, connected } = useWallet();
  const [userState, setUserState] = useState<UserState>({
    userTier: null,
    stakedAmount: 0,
    totalTokensMinted: 0,
    pendingYield: 0,
    vestingProgress: 0,
    isLoading: false,
    error: null,
  });

  const [txState, setTxState] = useState<TransactionState>({
    mintAmount: 0,
    stakeAmount: 0,
    isProcessing: false,
    txSignature: null,
  });

  // Fetch user state from blockchain
  const fetchUserState = useCallback(async () => {
    if (!publicKey || !connected) {
      setUserState((prev) => ({ ...prev, error: 'Wallet not connected' }));
      return;
    }

    setUserState((prev) => ({ ...prev, isLoading: true, error: null }));

    try {
      // TODO: Fetch actual state from on-chain accounts
      // This requires:
      // 1. UserTierInfo PDA
      // 2. StakingInfo PDA
      // 3. YieldSnapshot PDA
      // For now, mock data:

      setUserState((prev) => ({
        ...prev,
        userTier: 'tier1',
        stakedAmount: 500000,
        totalTokensMinted: 1000000,
        pendingYield: 1500,
        vestingProgress: 100,
        isLoading: false,
      }));
    } catch (err) {
      setUserState((prev) => ({
        ...prev,
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to fetch user state',
      }));
    }
  }, [publicKey, connected]);

  // Mint tokens
  const mintTokens = useCallback(
    async (usdcAmount: number, isTier2: boolean = false) => {
      if (!publicKey || !signTransaction) {
        setUserState((prev) => ({
          ...prev,
          error: 'Wallet not connected',
        }));
        return;
      }

      setTxState((prev) => ({ ...prev, isProcessing: true, txSignature: null }));

      try {
        // TODO: Build actual CPI transaction
        // This requires:
        // 1. Create MintTokens context from instructions
        // 2. Set up USDC transfer
        // 3. Call mint_tokens instruction
        // 4. Sign and send transaction

        // For now, mock success:
        const mockTxSig =
          'mockTx' + Math.random().toString(36).substring(2, 15);

        setTxState((prev) => ({
          ...prev,
          txSignature: mockTxSig,
          isProcessing: false,
        }));

        // Refresh user state
        await fetchUserState();
      } catch (err) {
        setUserState((prev) => ({
          ...prev,
          error: err instanceof Error ? err.message : 'Mint failed',
        }));
        setTxState((prev) => ({ ...prev, isProcessing: false }));
      }
    },
    [publicKey, signTransaction, fetchUserState]
  );

  // Stake tokens
  const stakeTokens = useCallback(
    async (amount: number) => {
      if (!publicKey || !signTransaction) {
        setUserState((prev) => ({
          ...prev,
          error: 'Wallet not connected',
        }));
        return;
      }

      setTxState((prev) => ({ ...prev, isProcessing: true }));

      try {
        // TODO: Build StakeTokens transaction
        // This requires:
        // 1. Transfer tokens to staking vault
        // 2. Call stake_tokens instruction
        // 3. Update StakingInfo PDA

        const mockTxSig =
          'mockTx' + Math.random().toString(36).substring(2, 15);

        setTxState((prev) => ({
          ...prev,
          txSignature: mockTxSig,
          isProcessing: false,
        }));

        await fetchUserState();
      } catch (err) {
        setUserState((prev) => ({
          ...prev,
          error: err instanceof Error ? err.message : 'Stake failed',
        }));
        setTxState((prev) => ({ ...prev, isProcessing: false }));
      }
    },
    [publicKey, signTransaction, fetchUserState]
  );

  // Unstake tokens
  const unstakeTokens = useCallback(
    async (amount: number) => {
      if (!publicKey || !signTransaction) {
        setUserState((prev) => ({
          ...prev,
          error: 'Wallet not connected',
        }));
        return;
      }

      setTxState((prev) => ({ ...prev, isProcessing: true }));

      try {
        // TODO: Build UnstakeTokens transaction
        const mockTxSig =
          'mockTx' + Math.random().toString(36).substring(2, 15);

        setTxState((prev) => ({
          ...prev,
          txSignature: mockTxSig,
          isProcessing: false,
        }));

        await fetchUserState();
      } catch (err) {
        setUserState((prev) => ({
          ...prev,
          error: err instanceof Error ? err.message : 'Unstake failed',
        }));
        setTxState((prev) => ({ ...prev, isProcessing: false }));
      }
    },
    [publicKey, signTransaction, fetchUserState]
  );

  // Claim yield
  const claimYield = useCallback(async () => {
    if (!publicKey || !signTransaction) {
      setUserState((prev) => ({
        ...prev,
        error: 'Wallet not connected',
      }));
      return;
    }

    setTxState((prev) => ({ ...prev, isProcessing: true }));

    try {
      // TODO: Build ClaimYield transaction
      // This requires:
      // 1. Fetch YieldSnapshot
      // 2. Calculate pro-rata share
      // 3. Call claim_yield instruction

      const mockTxSig =
        'mockTx' + Math.random().toString(36).substring(2, 15);

      setTxState((prev) => ({
        ...prev,
        txSignature: mockTxSig,
        isProcessing: false,
      }));

      await fetchUserState();
    } catch (err) {
      setUserState((prev) => ({
        ...prev,
        error: err instanceof Error ? err.message : 'Claim failed',
      }));
      setTxState((prev) => ({ ...prev, isProcessing: false }));
    }
  }, [publicKey, signTransaction, fetchUserState]);

  // Fetch state on mount and when wallet changes
  useEffect(() => {
    if (connected && publicKey) {
      fetchUserState();
    }
  }, [connected, publicKey, fetchUserState]);

  return {
    userState,
    txState,
    mintTokens,
    stakeTokens,
    unstakeTokens,
    claimYield,
    refreshState: fetchUserState,
  };
}
