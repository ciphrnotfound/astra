'use client';

import { useState } from 'react';

interface SettingsFormProps {
  onSubmit: (data: SettingsFormData) => Promise<void>;
  initialData?: Partial<SettingsFormData>;
  isLoading?: boolean;
}

export interface SettingsFormData {
  persona: {
    role: string;
    experience: string;
    preferences: Record<string, any>;
  };
  model_config: {
    model: string;
    temperature: number;
    max_tokens: number;
  };
  notifications: {
    email: boolean;
    realtime: boolean;
  };
}

export default function SettingsForm({ onSubmit, initialData, isLoading }: SettingsFormProps) {
  const [formData, setFormData] = useState<SettingsFormData>({
    persona: initialData?.persona || {
      role: '',
      experience: '',
      preferences: {},
    },
    model_config: initialData?.model_config || {
      model: 'gpt-4',
      temperature: 0.7,
      max_tokens: 2000,
    },
    notifications: initialData?.notifications || {
      email: true,
      realtime: true,
    },
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await onSubmit(formData);
    } catch (error) {
      console.error('Form submission error:', error);
    }
  };

  const updatePersona = (field: string, value: string) => {
    setFormData(prev => ({
      ...prev,
      persona: { ...prev.persona, [field]: value },
    }));
  };

  const updateModelConfig = (field: string, value: string | number) => {
    setFormData(prev => ({
      ...prev,
      model_config: { ...prev.model_config, [field]: value },
    }));
  };

  const updateNotifications = (field: string, value: boolean) => {
    setFormData(prev => ({
      ...prev,
      notifications: { ...prev.notifications, [field]: value },
    }));
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-8">
      {/* Persona Configuration */}
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          Developer Persona
        </h3>
        <div className="space-y-4">
          <div>
            <label htmlFor="role" className="block text-sm font-medium text-gray-700 mb-2">
              Role
            </label>
            <select
              id="role"
              value={formData.persona.role}
              onChange={(e) => updatePersona('role', e.target.value)}
              className="w-full px-4 py-2 border border-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-900"
              disabled={isLoading}
            >
              <option value="">Select your role</option>
              <option value="frontend">Frontend Developer</option>
              <option value="backend">Backend Developer</option>
              <option value="fullstack">Full Stack Developer</option>
              <option value="devops">DevOps Engineer</option>
              <option value="architect">Software Architect</option>
              <option value="other">Other</option>
            </select>
          </div>

          <div>
            <label htmlFor="experience" className="block text-sm font-medium text-gray-700 mb-2">
              Experience Level
            </label>
            <select
              id="experience"
              value={formData.persona.experience}
              onChange={(e) => updatePersona('experience', e.target.value)}
              className="w-full px-4 py-2 border border-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-900"
              disabled={isLoading}
            >
              <option value="">Select experience level</option>
              <option value="junior">Junior (0-2 years)</option>
              <option value="mid">Mid-level (2-5 years)</option>
              <option value="senior">Senior (5-10 years)</option>
              <option value="lead">Lead/Principal (10+ years)</option>
            </select>
          </div>
        </div>
      </div>

      {/* Model Configuration */}
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          AI Model Configuration
        </h3>
        <div className="space-y-4">
          <div>
            <label htmlFor="model" className="block text-sm font-medium text-gray-700 mb-2">
              Model
            </label>
            <select
              id="model"
              value={formData.model_config.model}
              onChange={(e) => updateModelConfig('model', e.target.value)}
              className="w-full px-4 py-2 border border-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-900"
              disabled={isLoading}
            >
              <option value="gpt-4">GPT-4</option>
              <option value="gpt-3.5-turbo">GPT-3.5 Turbo</option>
              <option value="claude-3">Claude 3</option>
            </select>
          </div>

          <div>
            <label htmlFor="temperature" className="block text-sm font-medium text-gray-700 mb-2">
              Temperature: {formData.model_config.temperature}
            </label>
            <input
              type="range"
              id="temperature"
              min="0"
              max="1"
              step="0.1"
              value={formData.model_config.temperature}
              onChange={(e) => updateModelConfig('temperature', parseFloat(e.target.value))}
              className="w-full"
              disabled={isLoading}
            />
            <p className="text-xs text-gray-500 mt-1">
              Lower values make output more focused, higher values more creative
            </p>
          </div>

          <div>
            <label htmlFor="max_tokens" className="block text-sm font-medium text-gray-700 mb-2">
              Max Tokens
            </label>
            <input
              type="number"
              id="max_tokens"
              min="100"
              max="4000"
              step="100"
              value={formData.model_config.max_tokens}
              onChange={(e) => updateModelConfig('max_tokens', parseInt(e.target.value))}
              className="w-full px-4 py-2 border border-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-900"
              disabled={isLoading}
            />
          </div>
        </div>
      </div>

      {/* Notifications */}
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          Notifications
        </h3>
        <div className="space-y-3">
          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={formData.notifications.email}
              onChange={(e) => updateNotifications('email', e.target.checked)}
              className="w-4 h-4"
              disabled={isLoading}
            />
            <span className="text-sm text-gray-700">Email notifications</span>
          </label>

          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={formData.notifications.realtime}
              onChange={(e) => updateNotifications('realtime', e.target.checked)}
              className="w-4 h-4"
              disabled={isLoading}
            />
            <span className="text-sm text-gray-700">Real-time updates</span>
          </label>
        </div>
      </div>

      <div className="pt-4">
        <button
          type="submit"
          disabled={isLoading}
          className="relative group overflow-hidden bg-gray-900 text-white px-6 py-3 text-sm font-medium transition-all hover:shadow-lg disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <span className="relative z-10">{isLoading ? 'Saving...' : 'Save Settings'}</span>
          <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
        </button>
      </div>
    </form>
  );
}
