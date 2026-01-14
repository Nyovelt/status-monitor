'use client';

import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { RefreshCw, Plus, Trash2, Copy, Check } from 'lucide-react';
import {
  getSettings,
  updateSettings,
  getAlertRules,
  createAlertRule,
  deleteAlertRule,
  getClients,
  createClient,
  deleteClient,
} from '@/lib/api';
import { cn } from '@/lib/utils';

export default function SettingsPage() {
  const queryClient = useQueryClient();

  return (
    <div className="space-y-8">
      <h1 className="text-2xl font-bold text-white">Settings</h1>

      <SlackSection />
      <AlertRulesSection />
      <ClientsSection />
    </div>
  );
}

function SlackSection() {
  const queryClient = useQueryClient();
  const [webhookUrl, setWebhookUrl] = useState('');

  const { data: settings, isLoading } = useQuery({
    queryKey: ['settings'],
    queryFn: getSettings,
  });

  const mutation = useMutation({
    mutationFn: (url: string) => updateSettings({ slack_webhook_url: url }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings'] });
    },
  });

  const currentUrl = settings?.slack_webhook_url || '';

  return (
    <section className="bg-gray-800 rounded-lg p-6">
      <h2 className="text-lg font-semibold text-white mb-4">Slack Notifications</h2>

      {isLoading ? (
        <RefreshCw className="w-5 h-5 animate-spin text-gray-500" />
      ) : (
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">
              Webhook URL
            </label>
            <div className="flex gap-2">
              <input
                type="url"
                value={webhookUrl || currentUrl}
                onChange={(e) => setWebhookUrl(e.target.value)}
                placeholder="https://hooks.slack.com/services/..."
                className="flex-1 bg-gray-900 border border-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-blue-500"
              />
              <button
                onClick={() => mutation.mutate(webhookUrl || currentUrl)}
                disabled={mutation.isPending}
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors disabled:opacity-50"
              >
                {mutation.isPending ? 'Saving...' : 'Save'}
              </button>
            </div>
          </div>

          {currentUrl && (
            <p className="text-sm text-green-400">
              Slack notifications are configured
            </p>
          )}
        </div>
      )}
    </section>
  );
}

function AlertRulesSection() {
  const queryClient = useQueryClient();
  const [newRule, setNewRule] = useState({
    metric_type: 'cpu',
    threshold: 90,
    duration_sec: 30,
  });

  const { data: rules, isLoading } = useQuery({
    queryKey: ['alertRules'],
    queryFn: getAlertRules,
  });

  const createMutation = useMutation({
    mutationFn: () => createAlertRule({
      client_id: null,
      metric_type: newRule.metric_type,
      threshold: newRule.threshold,
      duration_sec: newRule.duration_sec,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['alertRules'] });
      setNewRule({ metric_type: 'cpu', threshold: 90, duration_sec: 30 });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: deleteAlertRule,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['alertRules'] });
    },
  });

  return (
    <section className="bg-gray-800 rounded-lg p-6">
      <h2 className="text-lg font-semibold text-white mb-4">Alert Rules</h2>

      {/* Add new rule */}
      <div className="flex flex-wrap gap-3 mb-6 pb-6 border-b border-gray-700">
        <select
          value={newRule.metric_type}
          onChange={(e) => setNewRule({ ...newRule, metric_type: e.target.value })}
          className="bg-gray-900 border border-gray-700 rounded-lg px-3 py-2 text-white"
        >
          <option value="cpu">CPU</option>
          <option value="ram">RAM</option>
          <option value="disk">Disk</option>
          <option value="inode">Inodes</option>
          <option value="gpu">GPU</option>
        </select>

        <div className="flex items-center gap-2">
          <span className="text-gray-400">&gt;</span>
          <input
            type="number"
            value={newRule.threshold}
            onChange={(e) => setNewRule({ ...newRule, threshold: Number(e.target.value) })}
            className="w-20 bg-gray-900 border border-gray-700 rounded-lg px-3 py-2 text-white"
          />
          <span className="text-gray-400">%</span>
        </div>

        <div className="flex items-center gap-2">
          <span className="text-gray-400">for</span>
          <input
            type="number"
            value={newRule.duration_sec}
            onChange={(e) => setNewRule({ ...newRule, duration_sec: Number(e.target.value) })}
            className="w-20 bg-gray-900 border border-gray-700 rounded-lg px-3 py-2 text-white"
          />
          <span className="text-gray-400">seconds</span>
        </div>

        <button
          onClick={() => createMutation.mutate()}
          disabled={createMutation.isPending}
          className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors disabled:opacity-50"
        >
          <Plus className="w-4 h-4" />
          Add Rule
        </button>
      </div>

      {/* Rules list */}
      {isLoading ? (
        <RefreshCw className="w-5 h-5 animate-spin text-gray-500" />
      ) : rules && rules.length > 0 ? (
        <div className="space-y-2">
          {rules.map(rule => (
            <div
              key={rule.id}
              className="flex items-center justify-between bg-gray-900 rounded-lg px-4 py-3"
            >
              <span className="text-white">
                Alert when <span className="font-medium text-blue-400">{rule.metric_type.toUpperCase()}</span>
                {' > '}
                <span className="font-medium text-yellow-400">{rule.threshold}%</span>
                {' for '}
                <span className="font-medium text-gray-300">{rule.duration_sec}s</span>
              </span>
              <button
                onClick={() => deleteMutation.mutate(rule.id)}
                className="p-2 text-gray-400 hover:text-red-400 transition-colors"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          ))}
        </div>
      ) : (
        <p className="text-gray-500">No alert rules configured</p>
      )}
    </section>
  );
}

function ClientsSection() {
  const queryClient = useQueryClient();
  const [newHostname, setNewHostname] = useState('');
  const [copiedToken, setCopiedToken] = useState<string | null>(null);

  const { data: clients, isLoading } = useQuery({
    queryKey: ['clients'],
    queryFn: getClients,
  });

  const createMutation = useMutation({
    mutationFn: () => createClient(newHostname),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['clients'] });
      setNewHostname('');
      // Auto-copy token
      navigator.clipboard.writeText(data.token);
      setCopiedToken(data.token);
      setTimeout(() => setCopiedToken(null), 3000);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: deleteClient,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['clients'] });
    },
  });

  const copyToken = async (token: string) => {
    await navigator.clipboard.writeText(token);
    setCopiedToken(token);
    setTimeout(() => setCopiedToken(null), 2000);
  };

  return (
    <section className="bg-gray-800 rounded-lg p-6">
      <h2 className="text-lg font-semibold text-white mb-4">Clients</h2>

      {/* Add new client */}
      <div className="flex gap-3 mb-6 pb-6 border-b border-gray-700">
        <input
          type="text"
          value={newHostname}
          onChange={(e) => setNewHostname(e.target.value)}
          placeholder="hostname"
          className="flex-1 max-w-xs bg-gray-900 border border-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-blue-500"
        />
        <button
          onClick={() => createMutation.mutate()}
          disabled={createMutation.isPending || !newHostname}
          className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors disabled:opacity-50"
        >
          <Plus className="w-4 h-4" />
          Add Client
        </button>
      </div>

      {/* Clients list */}
      {isLoading ? (
        <RefreshCw className="w-5 h-5 animate-spin text-gray-500" />
      ) : clients && clients.length > 0 ? (
        <div className="space-y-2">
          {clients.map(client => (
            <div
              key={client.id}
              className="flex items-center justify-between bg-gray-900 rounded-lg px-4 py-3"
            >
              <div>
                <span className="text-white font-medium">{client.hostname}</span>
                <span className="ml-3 text-xs text-gray-500 font-mono">{client.id}</span>
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => copyToken(client.id)}
                  className="p-2 text-gray-400 hover:text-white transition-colors"
                  title="Copy client ID"
                >
                  {copiedToken === client.id ? (
                    <Check className="w-4 h-4 text-green-400" />
                  ) : (
                    <Copy className="w-4 h-4" />
                  )}
                </button>
                <button
                  onClick={() => {
                    if (confirm(`Delete client "${client.hostname}"?`)) {
                      deleteMutation.mutate(client.id);
                    }
                  }}
                  className="p-2 text-gray-400 hover:text-red-400 transition-colors"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <p className="text-gray-500">No clients registered</p>
      )}
    </section>
  );
}
