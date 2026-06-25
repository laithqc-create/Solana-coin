'use client';

import { useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import {
  LineChart,
  Line,
  BarChart,
  Bar,
  PieChart,
  Pie,
  Cell,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { TrendingUp } from 'lucide-react';

// Mock Aave position history
const aaveHistory = [
  { week: 'Week 1', position: 100000 },
  { week: 'Week 2', position: 150000 },
  { week: 'Week 3', position: 180000 },
  { week: 'Week 4', position: 220000 },
  { week: 'Week 5', position: 270000 },
  { week: 'Week 6', position: 310000 },
  { week: 'Week 7', position: 350000 },
  { week: 'Week 8', position: 395000 },
];

// Mock revenue distribution
const revenueData = [
  { name: 'Users (40%)', value: 40, fill: '#3b82f6' },
  { name: 'Marketing (20%)', value: 20, fill: '#8b5cf6' },
  { name: 'Manager (20%)', value: 20, fill: '#ec4899' },
  { name: 'Owner (20%)', value: 20, fill: '#f59e0b' },
];

// Mock distribution history
const distributionHistory = [
  {
    date: '2024-06-20',
    users: 12500,
    marketing: 6250,
    manager: 6250,
    owner: 6250,
  },
  {
    date: '2024-06-13',
    users: 11000,
    marketing: 5500,
    manager: 5500,
    owner: 5500,
  },
  {
    date: '2024-06-06',
    users: 9500,
    marketing: 4750,
    manager: 4750,
    owner: 4750,
  },
];

export default function TreasuryPage() {
  const { connected } = useWallet();
  const [allocationForm, setAllocationForm] = useState({
    users: 40,
    marketing: 20,
    manager: 20,
    owner: 20,
  });

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
      {/* Treasury Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="card">
          <p className="text-slate-400 text-sm mb-2">Aave Position</p>
          <p className="text-3xl font-bold text-blue-400">$395K</p>
          <p className="text-slate-500 text-xs mt-2">4% APY</p>
        </div>

        <div className="card">
          <p className="text-slate-400 text-sm mb-2">Total Yields Earned</p>
          <p className="text-3xl font-bold text-green-400">$18,750</p>
          <div className="flex items-center gap-1 mt-2 text-green-400 text-xs">
            <TrendingUp size={14} />
            <span>+$2,300 this week</span>
          </div>
        </div>

        <div className="card">
          <p className="text-slate-400 text-sm mb-2">Distributed</p>
          <p className="text-3xl font-bold text-yellow-400">$31,000</p>
          <p className="text-slate-500 text-xs mt-2">All parties combined</p>
        </div>
      </div>

      {/* Aave Position Chart */}
      <div className="card">
        <h3 className="text-lg font-semibold mb-4">Aave Position (8-Week History)</h3>
        <ResponsiveContainer width="100%" height={400}>
          <LineChart data={aaveHistory}>
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
              dataKey="position"
              stroke="#3b82f6"
              strokeWidth={2}
              dot={{ fill: '#3b82f6' }}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>

      {/* Revenue Allocation */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Pie Chart */}
        <div className="card">
          <h3 className="text-lg font-semibold mb-4">Revenue Allocation</h3>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={revenueData}
                cx="50%"
                cy="50%"
                labelLine={false}
                label={({ name, value }) => `${name}: ${value}%`}
                outerRadius={80}
                fill="#8884d8"
                dataKey="value"
              >
                {revenueData.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.fill} />
                ))}
              </Pie>
              <Tooltip />
            </PieChart>
          </ResponsiveContainer>
        </div>

        {/* Update Allocation Form */}
        <div className="card">
          <h3 className="text-lg font-semibold mb-6">Update Allocation (Multisig)</h3>
          <div className="space-y-4">
            {[
              { label: 'Users (%)', key: 'users' },
              { label: 'Marketing (%)', key: 'marketing' },
              { label: 'Manager (%)', key: 'manager' },
              { label: 'Owner (%)', key: 'owner' },
            ].map(({ label, key }) => (
              <div key={key}>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  {label}
                </label>
                <input
                  type="number"
                  value={allocationForm[key as keyof typeof allocationForm]}
                  onChange={(e) =>
                    setAllocationForm({
                      ...allocationForm,
                      [key]: parseInt(e.target.value),
                    })
                  }
                  min="0"
                  max="100"
                  className="w-full"
                />
              </div>
            ))}

            <div className="bg-slate-700/30 rounded p-3 text-sm">
              <p className="text-slate-400 mb-1">Total:</p>
              <p
                className={`text-lg font-bold ${
                  Object.values(allocationForm).reduce((a, b) => a + b) === 100
                    ? 'text-green-400'
                    : 'text-red-400'
                }`}
              >
                {Object.values(allocationForm).reduce((a, b) => a + b)}%
              </p>
            </div>

            <button
              className="w-full btn-primary disabled:opacity-50"
              disabled={
                Object.values(allocationForm).reduce((a, b) => a + b) !== 100
              }
            >
              Update Allocation
            </button>
          </div>
        </div>
      </div>

      {/* Distribution History */}
      <div className="card">
        <h3 className="text-lg font-semibold mb-4">Distribution History</h3>
        <ResponsiveContainer width="100%" height={300}>
          <BarChart data={distributionHistory}>
            <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
            <XAxis dataKey="date" stroke="#94a3b8" />
            <YAxis stroke="#94a3b8" />
            <Tooltip
              contentStyle={{
                backgroundColor: '#1e293b',
                border: '1px solid #475569',
              }}
            />
            <Legend />
            <Bar dataKey="users" stackId="a" fill="#3b82f6" />
            <Bar dataKey="marketing" stackId="a" fill="#8b5cf6" />
            <Bar dataKey="manager" stackId="a" fill="#ec4899" />
            <Bar dataKey="owner" stackId="a" fill="#f59e0b" />
          </BarChart>
        </ResponsiveContainer>
      </div>

      {/* Summary Table */}
      <div className="card">
        <h3 className="text-lg font-semibold mb-4">Weekly Distributions</h3>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-slate-700">
                <th className="text-left py-3 px-4 text-slate-400 font-medium text-sm">
                  Date
                </th>
                <th className="text-right py-3 px-4 text-slate-400 font-medium text-sm">
                  Users
                </th>
                <th className="text-right py-3 px-4 text-slate-400 font-medium text-sm">
                  Marketing
                </th>
                <th className="text-right py-3 px-4 text-slate-400 font-medium text-sm">
                  Manager
                </th>
                <th className="text-right py-3 px-4 text-slate-400 font-medium text-sm">
                  Owner
                </th>
                <th className="text-right py-3 px-4 text-slate-400 font-medium text-sm">
                  Total
                </th>
              </tr>
            </thead>
            <tbody>
              {distributionHistory.map((dist, idx) => (
                <tr key={idx} className="border-b border-slate-700/50 hover:bg-slate-700/20">
                  <td className="py-3 px-4 text-slate-300 text-sm">{dist.date}</td>
                  <td className="py-3 px-4 text-right text-blue-400">${dist.users.toLocaleString()}</td>
                  <td className="py-3 px-4 text-right text-purple-400">${dist.marketing.toLocaleString()}</td>
                  <td className="py-3 px-4 text-right text-pink-400">${dist.manager.toLocaleString()}</td>
                  <td className="py-3 px-4 text-right text-amber-400">${dist.owner.toLocaleString()}</td>
                  <td className="py-3 px-4 text-right font-semibold text-slate-300">
                    ${(dist.users + dist.marketing + dist.manager + dist.owner).toLocaleString()}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
