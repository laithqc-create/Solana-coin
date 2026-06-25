'use client';

import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';
import { TrendingUp, Users, DollarSign, Percent } from 'lucide-react';

// Mock protocol growth data
const protocolGrowth = [
  { week: 'Week 1', staked: 50000, users: 250 },
  { week: 'Week 2', staked: 120000, users: 580 },
  { week: 'Week 3', staked: 250000, users: 1200 },
  { week: 'Week 4', staked: 450000, users: 2100 },
  { week: 'Week 5', staked: 700000, users: 3200 },
  { week: 'Week 6', staked: 950000, users: 4300 },
  { week: 'Week 7', staked: 1250000, users: 5400 },
  { week: 'Week 8', staked: 1520000, users: 6200 },
];

// Mock yield distribution
const yieldData = [
  { week: 'Week 1', yield: 1200 },
  { week: 'Week 2', yield: 3500 },
  { week: 'Week 3', yield: 6800 },
  { week: 'Week 4', yield: 11200 },
  { week: 'Week 5', yield: 16500 },
  { week: 'Week 6', yield: 22300 },
  { week: 'Week 7', yield: 29100 },
  { week: 'Week 8', yield: 36800 },
];

// Mock top stakers
const topStakers = [
  { address: '7Qp...abc', staked: 125000, yield: 2500, percentage: '8.2%' },
  { address: '5Km...def', staked: 98000, yield: 1960, percentage: '6.4%' },
  { address: '3Wx...ghi', staked: 87500, yield: 1750, percentage: '5.8%' },
  { address: '9Py...jkl', staked: 72000, yield: 1440, percentage: '4.7%' },
  { address: '2Mn...mno', staked: 65000, yield: 1300, percentage: '4.3%' },
];

export default function AnalyticsPage() {
  const { connected } = useWallet();

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
      {/* Key Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <div className="card">
          <div className="flex items-center justify-between mb-3">
            <p className="text-slate-400 text-sm">Total Staked</p>
            <DollarSign size={20} className="text-blue-400" />
          </div>
          <p className="text-3xl font-bold text-blue-400">1.52M</p>
          <p className="text-slate-500 text-xs mt-2">tokens</p>
        </div>

        <div className="card">
          <div className="flex items-center justify-between mb-3">
            <p className="text-slate-400 text-sm">Active Users</p>
            <Users size={20} className="text-green-400" />
          </div>
          <p className="text-3xl font-bold text-green-400">6,200</p>
          <div className="flex items-center gap-1 mt-2 text-green-400 text-xs">
            <TrendingUp size={14} />
            <span>+800 this week</span>
          </div>
        </div>

        <div className="card">
          <div className="flex items-center justify-between mb-3">
            <p className="text-slate-400 text-sm">Weekly Yield</p>
            <Percent size={20} className="text-purple-400" />
          </div>
          <p className="text-3xl font-bold text-purple-400">$36.8K</p>
          <p className="text-slate-500 text-xs mt-2">USDC distributed</p>
        </div>

        <div className="card">
          <div className="flex items-center justify-between mb-3">
            <p className="text-slate-400 text-sm">Protocol APY</p>
            <TrendingUp size={20} className="text-yellow-400" />
          </div>
          <p className="text-3xl font-bold text-yellow-400">4.0%</p>
          <p className="text-slate-500 text-xs mt-2">annualized</p>
        </div>
      </div>

      {/* Protocol Growth */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="card">
          <h3 className="text-lg font-semibold mb-4">Staked Tokens Growth</h3>
          <ResponsiveContainer width="100%" height={300}>
            <AreaChart data={protocolGrowth}>
              <defs>
                <linearGradient id="colorStaked" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3} />
                  <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
              <XAxis dataKey="week" stroke="#94a3b8" />
              <YAxis stroke="#94a3b8" />
              <Tooltip
                contentStyle={{
                  backgroundColor: '#1e293b',
                  border: '1px solid #475569',
                }}
              />
              <Area
                type="monotone"
                dataKey="staked"
                stroke="#3b82f6"
                fillOpacity={1}
                fill="url(#colorStaked)"
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>

        <div className="card">
          <h3 className="text-lg font-semibold mb-4">User Growth</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={protocolGrowth}>
              <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
              <XAxis dataKey="week" stroke="#94a3b8" />
              <YAxis stroke="#94a3b8" />
              <Tooltip
                contentStyle={{
                  backgroundColor: '#1e293b',
                  border: '1px solid #475569',
                }}
              />
              <Line
                type="monotone"
                dataKey="users"
                stroke="#8b5cf6"
                strokeWidth={2}
                dot={{ fill: '#8b5cf6' }}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Yield Distribution */}
      <div className="card">
        <h3 className="text-lg font-semibold mb-4">Weekly Yield Distribution</h3>
        <ResponsiveContainer width="100%" height={400}>
          <AreaChart data={yieldData}>
            <defs>
              <linearGradient id="colorYield" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#f59e0b" stopOpacity={0.3} />
                <stop offset="95%" stopColor="#f59e0b" stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
            <XAxis dataKey="week" stroke="#94a3b8" />
            <YAxis stroke="#94a3b8" />
            <Tooltip
              contentStyle={{
                backgroundColor: '#1e293b',
                border: '1px solid #475569',
              }}
            />
            <Area
              type="monotone"
              dataKey="yield"
              stroke="#f59e0b"
              fillOpacity={1}
              fill="url(#colorYield)"
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>

      {/* Top Stakers */}
      <div className="card">
        <h3 className="text-lg font-semibold mb-4">Top Stakers Leaderboard</h3>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-slate-700">
                <th className="text-left py-3 px-4 text-slate-400 font-medium text-sm">
                  Rank
                </th>
                <th className="text-left py-3 px-4 text-slate-400 font-medium text-sm">
                  Address
                </th>
                <th className="text-right py-3 px-4 text-slate-400 font-medium text-sm">
                  Staked
                </th>
                <th className="text-right py-3 px-4 text-slate-400 font-medium text-sm">
                  Weekly Yield
                </th>
                <th className="text-right py-3 px-4 text-slate-400 font-medium text-sm">
                  % of Total
                </th>
              </tr>
            </thead>
            <tbody>
              {topStakers.map((staker, idx) => (
                <tr
                  key={idx}
                  className="border-b border-slate-700/50 hover:bg-slate-700/20"
                >
                  <td className="py-3 px-4 text-slate-300 font-semibold">
                    #{idx + 1}
                  </td>
                  <td className="py-3 px-4">
                    <span className="font-mono text-blue-400 text-sm">
                      {staker.address}
                    </span>
                  </td>
                  <td className="py-3 px-4 text-right text-slate-300 font-medium">
                    {(staker.staked / 1000).toFixed(0)}K
                  </td>
                  <td className="py-3 px-4 text-right text-green-400">
                    ${staker.yield.toLocaleString()}
                  </td>
                  <td className="py-3 px-4 text-right text-slate-400">
                    {staker.percentage}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Protocol Health */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="card">
          <h3 className="text-lg font-semibold mb-4">Protocol Health</h3>
          <div className="space-y-4">
            <div>
              <div className="flex justify-between mb-2">
                <span className="text-slate-300 text-sm">Collateralization</span>
                <span className="text-green-400 font-semibold">140%</span>
              </div>
              <div className="w-full bg-slate-700 rounded-full h-2">
                <div className="bg-green-500 h-2 rounded-full" style={{ width: '140%' }}></div>
              </div>
            </div>
            <div>
              <div className="flex justify-between mb-2">
                <span className="text-slate-300 text-sm">Staking Ratio</span>
                <span className="text-blue-400 font-semibold">65%</span>
              </div>
              <div className="w-full bg-slate-700 rounded-full h-2">
                <div className="bg-blue-500 h-2 rounded-full" style={{ width: '65%' }}></div>
              </div>
            </div>
            <div>
              <div className="flex justify-between mb-2">
                <span className="text-slate-300 text-sm">Treasury Utilization</span>
                <span className="text-purple-400 font-semibold">78%</span>
              </div>
              <div className="w-full bg-slate-700 rounded-full h-2">
                <div className="bg-purple-500 h-2 rounded-full" style={{ width: '78%' }}></div>
              </div>
            </div>
          </div>
        </div>

        <div className="card">
          <h3 className="text-lg font-semibold mb-4">Risk Indicators</h3>
          <div className="space-y-3">
            <div className="flex items-center justify-between p-3 bg-slate-700/30 rounded">
              <span className="text-slate-300">Vesting Cliff Risk</span>
              <span className="text-yellow-400 font-semibold">Low</span>
            </div>
            <div className="flex items-center justify-between p-3 bg-slate-700/30 rounded">
              <span className="text-slate-300">Liquidity Risk</span>
              <span className="text-green-400 font-semibold">Very Low</span>
            </div>
            <div className="flex items-center justify-between p-3 bg-slate-700/30 rounded">
              <span className="text-slate-300">Concentration Risk</span>
              <span className="text-yellow-400 font-semibold">Medium</span>
            </div>
            <div className="flex items-center justify-between p-3 bg-slate-700/30 rounded">
              <span className="text-slate-300">Smart Contract Risk</span>
              <span className="text-green-400 font-semibold">Low</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
