'use client';

import { useState, useEffect } from 'react';
import { Key, Copy, Trash2, Plus, Eye, EyeOff } from 'lucide-react';

interface ApiKey {
  id: number;
  name: string;
  keyPrefix: string;
  lastUsedAt: string | null;
  createdAt: string;
  expiresAt: string | null;
  isActive: boolean;
}

export default function ApiKeysPage() {
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([]);
  const [showNewKeyModal, setShowNewKeyModal] = useState(false);
  const [newKeyName, setNewKeyName] = useState('');
  const [generatedKey, setGeneratedKey] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchApiKeys();
  }, []);

  const fetchApiKeys = async () => {
    try {
      const res = await fetch('/api/api-keys');
      const data = await res.json();
      setApiKeys(data.apiKeys || []);
    } catch (error) {
      console.error('Failed to fetch API keys:', error);
    } finally {
      setLoading(false);
    }
  };

  const createApiKey = async () => {
    if (!newKeyName.trim()) return;

    try {
      const res = await fetch('/api/api-keys', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: newKeyName }),
      });
      const data = await res.json();
      
      setGeneratedKey(data.key);
      setNewKeyName('');
      fetchApiKeys();
    } catch (error) {
      console.error('Failed to create API key:', error);
    }
  };

  const deleteApiKey = async (id: number) => {
    if (!confirm('Are you sure you want to delete this API key?')) return;

    try {
      await fetch(`/api/api-keys/${id}`, { method: 'DELETE' });
      fetchApiKeys();
    } catch (error) {
      console.error('Failed to delete API key:', error);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  if (loading) {
    return <div className="p-8">Loading...</div>;
  }

  return (
    <div className="p-8">
      <div className="mb-8">
        <h1 className="text-4xl font-medium text-gray-900 mb-2 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          API Keys
        </h1>
        <p className="text-gray-600">Manage your Astra CLI authentication keys</p>
      </div>

      {/* Create New Key Button */}
      <div className="mb-6">
        <button
          onClick={() => setShowNewKeyModal(true)}
          className="relative group overflow-hidden bg-gray-900 text-white px-4 py-2 text-sm font-medium transition-all hover:shadow-lg flex items-center gap-2"
        >
          <Plus className="w-4 h-4" />
          <span className="relative z-10">Create New API Key</span>
          <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
        </button>
      </div>

      {/* API Keys List */}
      <div className="bg-white border border-gray-200">
        {apiKeys.length === 0 ? (
          <div className="p-12 text-center">
            <Key className="w-16 h-16 text-gray-300 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-gray-900 mb-2">No API keys yet</h3>
            <p className="text-sm text-gray-600 mb-6">
              Create an API key to authenticate your Astra CLI
            </p>
          </div>
        ) : (
          <div className="divide-y divide-gray-200">
            {apiKeys.map((key) => (
              <div key={key.id} className="p-6 hover:bg-gray-50 transition-colors">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <h3 className="text-lg font-medium text-gray-900">{key.name}</h3>
                      {key.isActive ? (
                        <span className="px-2 py-0.5 bg-green-100 text-green-800 text-xs font-medium">Active</span>
                      ) : (
                        <span className="px-2 py-0.5 bg-gray-100 text-gray-800 text-xs font-medium">Inactive</span>
                      )}
                    </div>
                    <div className="space-y-1 text-sm text-gray-600">
                      <p>Key: {key.keyPrefix}••••••••••••••••</p>
                      <p>Created: {new Date(key.createdAt).toLocaleDateString()}</p>
                      {key.lastUsedAt && (
                        <p>Last used: {new Date(key.lastUsedAt).toLocaleDateString()}</p>
                      )}
                      {key.expiresAt && (
                        <p>Expires: {new Date(key.expiresAt).toLocaleDateString()}</p>
                      )}
                    </div>
                  </div>
                  <button
                    onClick={() => deleteApiKey(key.id)}
                    className="relative group overflow-hidden p-2 border border-red-300 text-red-600 transition-all hover:text-white"
                  >
                    <Trash2 className="w-4 h-4 relative z-10" />
                    <div className="absolute inset-0 bg-red-600 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Create Key Modal */}
      {showNewKeyModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white p-8 max-w-md w-full mx-4">
            <h2 className="text-2xl font-medium text-gray-900 mb-4">Create New API Key</h2>
            
            {generatedKey ? (
              <div>
                <p className="text-sm text-gray-600 mb-4">
                  Save this key now. You won't be able to see it again!
                </p>
                <div className="bg-gray-50 p-4 mb-4 border border-gray-200">
                  <code className="text-sm break-all">{generatedKey}</code>
                </div>
                <div className="flex gap-3">
                  <button
                    onClick={() => copyToClipboard(generatedKey)}
                    className="flex-1 relative group overflow-hidden border border-gray-900 text-gray-900 px-4 py-2 text-sm font-medium transition-all hover:text-white flex items-center justify-center gap-2"
                  >
                    <Copy className="w-4 h-4 relative z-10" />
                    <span className="relative z-10">Copy Key</span>
                    <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
                  </button>
                  <button
                    onClick={() => {
                      setGeneratedKey(null);
                      setShowNewKeyModal(false);
                    }}
                    className="flex-1 bg-gray-900 text-white px-4 py-2 text-sm font-medium hover:bg-gray-800"
                  >
                    Done
                  </button>
                </div>
              </div>
            ) : (
              <div>
                <input
                  type="text"
                  value={newKeyName}
                  onChange={(e) => setNewKeyName(e.target.value)}
                  placeholder="e.g., Production CLI Key"
                  className="w-full px-4 py-2 border border-gray-300 mb-4 focus:outline-none focus:border-gray-900"
                />
                <div className="flex gap-3">
                  <button
                    onClick={() => {
                      setShowNewKeyModal(false);
                      setNewKeyName('');
                    }}
                    className="flex-1 border border-gray-300 text-gray-700 px-4 py-2 text-sm font-medium hover:bg-gray-50"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={createApiKey}
                    disabled={!newKeyName.trim()}
                    className="flex-1 bg-gray-900 text-white px-4 py-2 text-sm font-medium hover:bg-gray-800 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Create Key
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
