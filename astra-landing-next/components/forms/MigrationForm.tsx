'use client';

import { useState } from 'react';

interface MigrationFormProps {
  onSubmit: (data: MigrationFormData) => Promise<void>;
  onCancel?: () => void;
  projectId?: number;
  isLoading?: boolean;
}

export interface MigrationFormData {
  project_id: number;
  source_language: string;
  target_language: string;
}

const SUPPORTED_LANGUAGES = [
  'JavaScript',
  'TypeScript',
  'Python',
  'Rust',
  'Go',
  'Java',
  'C++',
  'C#',
  'Ruby',
  'PHP',
];

export default function MigrationForm({ onSubmit, onCancel, projectId, isLoading }: MigrationFormProps) {
  const [formData, setFormData] = useState<MigrationFormData>({
    project_id: projectId || 0,
    source_language: '',
    target_language: '',
  });
  const [errors, setErrors] = useState<Partial<Record<keyof MigrationFormData, string>>>({});

  const validate = (): boolean => {
    const newErrors: Partial<Record<keyof MigrationFormData, string>> = {};

    if (!formData.project_id) {
      newErrors.project_id = 'Project is required';
    }

    if (!formData.source_language) {
      newErrors.source_language = 'Source language is required';
    }

    if (!formData.target_language) {
      newErrors.target_language = 'Target language is required';
    }

    if (formData.source_language === formData.target_language) {
      newErrors.target_language = 'Target language must be different from source language';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) return;

    try {
      await onSubmit(formData);
    } catch (error) {
      console.error('Form submission error:', error);
    }
  };

  const handleChange = (field: keyof MigrationFormData, value: string | number) => {
    setFormData(prev => ({ ...prev, [field]: value }));
    if (errors[field]) {
      setErrors(prev => ({ ...prev, [field]: undefined }));
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      <div>
        <label htmlFor="source_language" className="block text-sm font-medium text-gray-700 mb-2">
          Source Language *
        </label>
        <select
          id="source_language"
          value={formData.source_language}
          onChange={(e) => handleChange('source_language', e.target.value)}
          className={`w-full px-4 py-2 border ${errors.source_language ? 'border-red-500' : 'border-gray-300'} focus:outline-none focus:ring-2 focus:ring-gray-900`}
          disabled={isLoading}
        >
          <option value="">Select source language</option>
          {SUPPORTED_LANGUAGES.map(lang => (
            <option key={lang} value={lang}>{lang}</option>
          ))}
        </select>
        {errors.source_language && <p className="mt-1 text-sm text-red-600">{errors.source_language}</p>}
      </div>

      <div>
        <label htmlFor="target_language" className="block text-sm font-medium text-gray-700 mb-2">
          Target Language *
        </label>
        <select
          id="target_language"
          value={formData.target_language}
          onChange={(e) => handleChange('target_language', e.target.value)}
          className={`w-full px-4 py-2 border ${errors.target_language ? 'border-red-500' : 'border-gray-300'} focus:outline-none focus:ring-2 focus:ring-gray-900`}
          disabled={isLoading}
        >
          <option value="">Select target language</option>
          {SUPPORTED_LANGUAGES.map(lang => (
            <option key={lang} value={lang}>{lang}</option>
          ))}
        </select>
        {errors.target_language && <p className="mt-1 text-sm text-red-600">{errors.target_language}</p>}
      </div>

      <div className="bg-gray-50 border border-gray-200 p-4">
        <p className="text-sm text-gray-600">
          Note: The migration will be performed by the Astra CLI. This form creates a migration record that the CLI will process.
        </p>
      </div>

      <div className="flex gap-4 pt-4">
        <button
          type="submit"
          disabled={isLoading}
          className="relative group overflow-hidden bg-gray-900 text-white px-6 py-3 text-sm font-medium transition-all hover:shadow-lg disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <span className="relative z-10">{isLoading ? 'Creating...' : 'Create Migration'}</span>
          <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
        </button>
        {onCancel && (
          <button
            type="button"
            onClick={onCancel}
            disabled={isLoading}
            className="relative group overflow-hidden border border-gray-900 text-gray-900 px-6 py-3 text-sm font-medium transition-all hover:text-white disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <span className="relative z-10">Cancel</span>
            <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
          </button>
        )}
      </div>
    </form>
  );
}
