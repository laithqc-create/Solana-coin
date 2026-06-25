'use client';

import { useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { Lock, Unlock, Users, Settings } from 'lucide-react';

// Mock whitelist
const whitelistData = [
  { address: '7Qp...abc', whitelisted: true, addedAt: '2024-06-10' },
  { address: '5Km...def', whitelisted: true, addedAt: '2024-06-12' },
  { address: '3Wx...ghi', whitelisted: false, addedAt: '2024-06-15' },
  { address: '9Py...jkl', whitelisted: true, addedAt: '2024-06-18' },
  { address: '2Mn...mno', whitelisted: true, addedAt: '2024-06-20' },
];

export default function AdminPage() {
  const { connected } = useWallet();
  const [launchpadPaused, setLaunchpadPaused] = useState(false);
  const [currentDiscount, setCurrentDiscount] = useState(50);
  const [newWhitelistAddress, setNewWhitelistAddress] = useState('');
  const [whitelist, setWhitelist] = useState(whitelistData);
  const [activeTab, setActiveTab] = useState<'whitelist' | 'controls' | 'tiers'>('whitelist');

  if (!connected) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh]">
        <h2 className="text-2xl font-bold mb-4">Connect Your Wallet</h2>
        <WalletMultiButton />
      </div>
    );
  }

  const handleAddToWhitelist = () => {
    if (newWhitelistAddress) {
      setWhitelist([
        ...whitelist,
        {
          address: newWhitelistAddress,
          whitelisted: true,
          addedAt: new Date().toISOString().split('T')[0],
        },
      ]);
      setNewWhitelistAddress('');
    }
  };

  const handleToggleWhitelist = (address: string) => {
    setWhitelist(
      whitelist.map((item) =>
        item.address === address
          ? { ...item, whitelisted: !item.whitelisted }
          : item
      )
    );
  };

  return (
    <div className="space-y-8">
      <div className="bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-4 flex items-start gap-3">
        <Lock className="text-yellow-400 mt-1 flex-shrink-0" size={20} />
        <div>
          <h3 className="font-semibold text-yellow-400 mb-1">Admin Functions</h3>
          <p className="text-yellow-300/80 text-sm">
            Only authorized users can access these controls. All changes require multisig approval.
          </p>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="flex gap-2 border-b border-slate-700">
        {[
          { id: 'whitelist', label: 'Tier 2 Whitelist', icon: Users },
          { id: 'controls', label: 'Emergency Controls', icon: Unlock },
          { id: 'tiers', label: 'Discount Tiers', icon: Settings },
        ].map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => setActiveTab(id as any)}
            className={`px-4 py-3 border-b-2 transition-colors flex items-center gap-2 ${
              activeTab === id
                ? 'border-blue-500 text-blue-400'
                : 'border-transparent text-slate-400 hover:text-slate-300'
            }`}
          >
            <Icon size={16} />
            {label}
          </button>
        ))}
      </div>

      {/* Whitelist Tab */}
      {activeTab === 'whitelist' && (
        <div className="space-y-6">
          <div className="card">
            <h3 className="text-lg font-semibold mb-4">Add to Whitelist</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  Wallet Address
                </label>
                <input
                  type="text"
                  value={newWhitelistAddress}
                  onChange={(e) => setNewWhitelistAddress(e.target.value)}
                  placeholder="Enter Solana wallet address..."
                  className="w-full"
                />
              </div>
              <button
                onClick={handleAddToWhitelist}
                disabled={!newWhitelistAddress}
                className="w-full btn-primary disabled:opacity-50"
              >
                Add to Whitelist
              </button>
            </div>
          </div>

          <div className="card">
            <h3 className="text-lg font-semibold mb-4">Whitelist Management</h3>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-slate-700">
                    <th className="text-left py-3 px-4 text-slate-400 font-medium text-sm">
                      Address
                    </th>
                    <th className="text-left py-3 px-4 text-slate-400 font-medium text-sm">
                      Status
                    </th>
                    <th className="text-left py-3 px-4 text-slate-400 font-medium text-sm">
                      Added
                    </th>
                    <th className="text-center py-3 px-4 text-slate-400 font-medium text-sm">
                      Action
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {whitelist.map((item, idx) => (
                    <tr key={idx} className="border-b border-slate-700/50 hover:bg-slate-700/20">
                      <td className="py-3 px-4 text-slate-300 font-mono text-sm">
                        {item.address}
                      </td>
                      <td className="py-3 px-4">
                        <span
                          className={`inline-block px-2 py-1 rounded text-xs font-medium ${
                            item.whitelisted
                              ? 'bg-green-500/20 text-green-300'
                              : 'bg-red-500/20 text-red-300'
                          }`}
                        >
                          {item.whitelisted ? '✓ Whitelisted' : '✗ Not Whitelisted'}
                        </span>
                      </td>
                      <td className="py-3 px-4 text-slate-400 text-sm">
                        {item.addedAt}
                      </td>
                      <td className="py-3 px-4 text-center">
                        <button
                          onClick={() => handleToggleWhitelist(item.address)}
                          className="text-xs btn-secondary px-3 py-1"
                        >
                          {item.whitelisted ? 'Remove' : 'Add'}
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* Emergency Controls Tab */}
      {activeTab === 'controls' && (
        <div className="space-y-6">
          <div className="card">
            <h3 className="text-lg font-semibold mb-4">Launchpad Status</h3>
            <div className="space-y-4">
              <div className="flex items-center justify-between p-4 bg-slate-700/30 rounded">
                <div>
                  <p className="font-medium text-slate-300">Launchpad State</p>
                  <p className="text-sm text-slate-500 mt-1">
                    {launchpadPaused ? 'Currently paused' : 'Currently active'}
                  </p>
                </div>
                <span
                  className={`inline-block px-3 py-1 rounded font-medium text-sm ${
                    launchpadPaused
                      ? 'bg-red-500/20 text-red-300'
                      : 'bg-green-500/20 text-green-300'
                  }`}
                >
                  {launchpadPaused ? 'PAUSED' : 'ACTIVE'}
                </span>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setLaunchpadPaused(true)}
                  disabled={launchpadPaused}
                  className="flex-1 btn-primary disabled:opacity-50 flex items-center justify-center gap-2"
                >
                  <Lock size={16} />
                  Pause Launchpad
                </button>
                <button
                  onClick={() => setLaunchpadPaused(false)}
                  disabled={!launchpadPaused}
                  className="flex-1 btn-secondary disabled:opacity-50 flex items-center justify-center gap-2"
                >
                  <Unlock size={16} />
                  Resume Launchpad
                </button>
              </div>

              <div className="bg-blue-500/10 border border-blue-500/20 rounded p-3 text-sm">
                <p className="text-blue-300">ℹ️ Pausing will:</p>
                <ul className="list-disc list-inside text-blue-300/80 text-xs mt-2 space-y-1">
                  <li>Prevent new token minting</li>
                  <li>Allow existing stakes/yields to continue</li>
                  <li>Require multisig approval to resume</li>
                </ul>
              </div>
            </div>
          </div>

          <div className="card">
            <h3 className="text-lg font-semibold mb-4">Emergency Procedures</h3>
            <div className="space-y-3">
              <button className="w-full p-3 border border-slate-600 rounded hover:bg-slate-700/20 text-slate-300 transition-colors">
                Invoke Emergency Freeze
              </button>
              <button className="w-full p-3 border border-slate-600 rounded hover:bg-slate-700/20 text-slate-300 transition-colors">
                Recover Funds (Multisig Only)
              </button>
              <button className="w-full p-3 border border-slate-600 rounded hover:bg-slate-700/20 text-slate-300 transition-colors">
                Upgrade Smart Contract
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Discount Tiers Tab */}
      {activeTab === 'tiers' && (
        <div className="space-y-6">
          <div className="card">
            <h3 className="text-lg font-semibold mb-6">Current Discount Tier</h3>
            <div className="bg-gradient-to-r from-blue-500/10 to-purple-500/10 border border-blue-500/20 rounded-lg p-6 text-center">
              <p className="text-slate-400 mb-2">Current Tier Discount</p>
              <p className="text-5xl font-bold text-blue-400 mb-2">{currentDiscount}%</p>
              <p className="text-slate-500">
                1 USDC = {(1 + currentDiscount / 100).toFixed(2)} tokens
              </p>
            </div>
          </div>

          <div className="card">
            <h3 className="text-lg font-semibold mb-4">Discount Thresholds</h3>
            <div className="space-y-4">
              <div className="p-4 bg-slate-700/30 rounded">
                <div className="flex justify-between mb-2">
                  <span className="text-slate-300">$0 - $1M raised</span>
                  <span className="font-semibold text-blue-400">50% discount</span>
                </div>
                <p className="text-xs text-slate-500">Price: 0.5 USDC = 1 token</p>
              </div>

              <div className="p-4 bg-slate-700/30 rounded">
                <div className="flex justify-between mb-2">
                  <span className="text-slate-300">$1M - $2M raised</span>
                  <span className="font-semibold text-purple-400">40% discount</span>
                </div>
                <p className="text-xs text-slate-500">Price: 0.6 USDC = 1 token</p>
              </div>

              <div className="p-4 bg-slate-700/30 rounded">
                <div className="flex justify-between mb-2">
                  <span className="text-slate-300">$2M+ raised</span>
                  <span className="font-semibold text-yellow-400">50% discount</span>
                </div>
                <p className="text-xs text-slate-500">Price: 0.5 USDC = 1 token</p>
              </div>
            </div>
          </div>

          <div className="card">
            <h3 className="text-lg font-semibold mb-4">Update Tier Settings</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  Tier 1 Threshold ($)
                </label>
                <input
                  type="number"
                  defaultValue="1000000"
                  placeholder="Threshold in USDC"
                  className="w-full"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  Tier 2 Threshold ($)
                </label>
                <input
                  type="number"
                  defaultValue="2000000"
                  placeholder="Threshold in USDC"
                  className="w-full"
                />
              </div>
              <button className="w-full btn-primary">
                Update Thresholds (Multisig)
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
