'use client';

import { useState, useEffect } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { useEcosystemToken } from '@/hooks/useEcosystemToken';
import {
  BarChart,
  Bar,
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';
import { TrendingUp, Lock, Unlock } from 'lucide-react';

// Mock transaction history
const transactionHistory = [
  {
    date: '2024-06-20',
    type: 'Stake',
    amount: 500,
    txHash: '3aBc...eFg',
  },
  {
    date: '2024-06-19',
    type: 'Claim Yield',
    amount: 125.5,
    txHash: '2xYz...aB1',
  },
  {
    date: '2024-06-18',
    type: 'Mint',
    amount: 1000,
    txHash: '1mNo...pqR',
  },
];

// Mock APY history
const apyHistory = [
  { month: 'Week 1', apy: 3.2 },
  { month: 'Week 2', apy: 3.5 },
  { month: 'Week 3', apy: 3.8 },
  { month: 'Week 4', apy: 4.1 },
];

// Mock staking data
const stakingData = [
  { day: 'Mon', staked: 500 },
  { day: 'Tue', staked: 620 },
  { day: 'Wed', staked: 800 },
  { day: 'Thu', staked: 950 },
  { day: 'Fri', staked: 1200 },
  { day: 'Sat', staked: 1100 },
  { day: 'Sun', staked: 1050 },
];

export default function DashboardPage() {
  const { connected } = useWallet();
  const { userState, txState, stakeTokens, unstakeTokens, claimYield } =
    useEcosystemToken();

  const [mintAmount, setMintAmount] = useState('');
  const [stakeAmount, setStakeAmount] = useState('');
  const [activeTab, setActiveTab] = useState<'stake' | 'mint'>('stake');

  if (!connected) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh]">
        <h2 className="text-2xl font-bold mb-4">Connect Your Wallet</h2>
        <WalletMultiButton />
      </div>
    );
  }

  return (
    <div className="space-y-8">
      {/* Hero Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <div className="card">
          <p className="text-slate-400 text-sm mb-2">Total Tokens</p>
          <p className="text-3xl font-bold text-blue-400">
            {(userState.totalTokensMinted / 1_000_000).toFixed(2)}M
          </p>
          <p className="text-slate-500 text-xs mt-2">Tier {userState.userTier === 'tier1' ? '1' : '2'}</p>
        </div>

        <div className="card">
          <p className="text-slate-400 text-sm mb-2">Staked Amount</p>
          <p className="text-3xl font-bold text-green-400">
            {(userState.stakedAmount / 1000).toFixed(1)}K
          </p>
          <p className="text-slate-500 text-xs mt-2">Earning yield</p>
        </div>

        <div className="card">
          <p className="text-slate-400 text-sm mb-2">Pending Yield</p>
          <p className="text-3xl font-bold text-yellow-400">
            ${userState.pendingYield.toFixed(2)}
          </p>
          <p className="text-slate-500 text-xs mt-2">Next claim: 3 days</p>
        </div>

        <div className="card">
          <p className="text-slate-400 text-sm mb-2">Current APY</p>
          <p className="text-3xl font-bold text-purple-400">4.0%</p>
          <div className="flex items-center gap-1 mt-2 text-green-400 text-xs">
            <TrendingUp size={14} />
            <span>+0.3% this week</span>
          </div>
        </div>
      </div>

      {/* Charts Section */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Staking Trend */}
        <div className="card">
          <h3 className="text-lg font-semibold mb-4">Staking Trend</h3>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={stakingData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
              <XAxis dataKey="day" stroke="#94a3b8" />
              <YAxis stroke="#94a3b8" />
              <Tooltip
                contentStyle={{
                  backgroundColor: '#1e293b',
                  border: '1px solid #475569',
                }}
              />
              <Bar dataKey="staked" fill="#3b82f6" radius={[8, 8, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* APY History */}
        <div className="card">
          <h3 className="text-lg font-semibold mb-4">APY History</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={apyHistory}>
              <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
              <XAxis dataKey="month" stroke="#94a3b8" />
              <YAxis stroke="#94a3b8" />
              <Tooltip
                contentStyle={{
                  backgroundColor: '#1e293b',
                  border: '1px solid #475569',
                }}
              />
              <Line
                type="monotone"
                dataKey="apy"
                stroke="#8b5cf6"
                strokeWidth={2}
                dot={{ fill: '#8b5cf6' }}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Actions */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Staking/Mint Actions */}
        <div className="card">
          <h3 className="text-lg font-semibold mb-6">Manage Stake</h3>

          {/* Tabs */}
          <div className="flex gap-2 mb-6 border-b border-slate-700">
            <button
              onClick={() => setActiveTab('stake')}
              className={`px-4 py-2 border-b-2 transition-colors ${
                activeTab === 'stake'
                  ? 'border-blue-500 text-blue-400'
                  : 'border-transparent text-slate-400 hover:text-slate-300'
              }`}
            >
              Stake
            </button>
            <button
              onClick={() => setActiveTab('mint')}
              className={`px-4 py-2 border-b-2 transition-colors ${
                activeTab === 'mint'
                  ? 'border-blue-500 text-blue-400'
                  : 'border-transparent text-slate-400 hover:text-slate-300'
              }`}
            >
              Mint
            </button>
          </div>

          {/* Stake Tab */}
          {activeTab === 'stake' && (
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  Amount to Stake
                </label>
                <input
                  type="number"
                  value={stakeAmount}
                  onChange={(e) => setStakeAmount(e.target.value)}
                  placeholder="Enter amount..."
                  className="w-full"
                />
              </div>
              <div className="flex gap-2">
                <button
                  onClick={() => stakeTokens(parseFloat(stakeAmount))}
                  disabled={!stakeAmount || txState.isProcessing}
                  className="flex-1 btn-primary disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                >
                  <Lock size={16} />
                  {txState.isProcessing ? 'Processing...' : 'Stake Tokens'}
                </button>
                <button
                  onClick={() => unstakeTokens(parseFloat(stakeAmount))}
                  disabled={!stakeAmount || txState.isProcessing}
                  className="flex-1 btn-secondary disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                >
                  <Unlock size={16} />
                  {txState.isProcessing ? 'Processing...' : 'Unstake'}
                </button>
              </div>
            </div>
          )}

          {/* Mint Tab */}
          {activeTab === 'mint' && (
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  USDC Amount
                </label>
                <input
                  type="number"
                  value={mintAmount}
                  onChange={(e) => setMintAmount(e.target.value)}
                  placeholder="Enter USDC amount..."
                  className="w-full"
                />
              </div>
              <div className="bg-slate-700/30 rounded p-3 text-sm">
                <p className="text-slate-400 mb-2">Current Tier 1 Price:</p>
                <p className="text-blue-400 font-semibold">
                  1 USDC = 1 Token
                </p>
              </div>
              <button
                onClick={() => alert('Mint functionality will be wired in Session 4')}
                className="w-full btn-primary"
              >
                Mint Tokens
              </button>
            </div>
          )}
        </div>

        {/* Yield Claiming */}
        <div className="card">
          <h3 className="text-lg font-semibold mb-6">Claim Yield</h3>

          <div className="space-y-4">
            <div className="bg-gradient-to-r from-blue-500/10 to-purple-500/10 border border-blue-500/20 rounded-lg p-4">
              <p className="text-slate-400 text-sm mb-2">Available to Claim</p>
              <p className="text-3xl font-bold text-blue-400 mb-1">
                ${userState.pendingYield.toFixed(2)}
              </p>
              <p className="text-slate-500 text-xs">
                Next snapshot in 3 days
              </p>
            </div>

            <div className="grid grid-cols-3 gap-3 text-center">
              <div>
                <p className="text-slate-500 text-xs mb-1">Frequency</p>
                <p className="font-semibold">Weekly</p>
              </div>
              <div>
                <p className="text-slate-500 text-xs mb-1">Distribution</p>
                <p className="font-semibold">Pro-rata</p>
              </div>
              <div>
                <p className="text-slate-500 text-xs mb-1">Token</p>
                <p className="font-semibold">USDC</p>
              </div>
            </div>

            <button
              onClick={claimYield}
              disabled={userState.pendingYield === 0 || txState.isProcessing}
              className="w-full btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {txState.isProcessing ? 'Processing...' : 'Claim Yield'}
            </button>

            {txState.txSignature && (
              <div className="bg-green-500/10 border border-green-500/20 rounded p-3 text-sm">
                <p className="text-green-400">✓ Transaction successful</p>
                <p className="text-slate-500 text-xs mt-1 truncate">
                  {txState.txSignature}
                </p>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Transaction History */}
      <div className="card">
        <h3 className="text-lg font-semibold mb-4">Recent Transactions</h3>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-slate-700">
                <th className="text-left py-3 px-4 text-slate-400 font-medium text-sm">
                  Date
                </th>
                <th className="text-left py-3 px-4 text-slate-400 font-medium text-sm">
                  Type
                </th>
                <th className="text-right py-3 px-4 text-slate-400 font-medium text-sm">
                  Amount
                </th>
                <th className="text-left py-3 px-4 text-slate-400 font-medium text-sm">
                  Tx Hash
                </th>
              </tr>
            </thead>
            <tbody>
              {transactionHistory.map((tx, idx) => (
                <tr
                  key={idx}
                  className="border-b border-slate-700/50 hover:bg-slate-700/20"
                >
                  <td className="py-3 px-4 text-slate-300 text-sm">{tx.date}</td>
                  <td className="py-3 px-4">
                    <span className="inline-block px-2 py-1 bg-blue-500/20 text-blue-300 rounded text-xs font-medium">
                      {tx.type}
                    </span>
                  </td>
                  <td className="py-3 px-4 text-right text-slate-300 font-medium">
                    {typeof tx.amount === 'number'
                      ? `${tx.amount.toFixed(2)}`
                      : tx.amount}
                  </td>
                  <td className="py-3 px-4 text-slate-400 text-sm font-mono">
                    {tx.txHash}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Error Display */}
      {userState.error && (
        <div className="bg-red-500/10 border border-red-500/20 rounded p-4 text-red-400">
          {userState.error}
        </div>
      )}
    </div>
  );
}
